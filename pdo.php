<?php
namespace PdoEmulator;

/**
 * PDO Emulator using the Sqlx library.
 * Designed to ease migration of existing PDO-based code.
 *
 * VERSION 3.0 IMPROVEMENTS:
 * - Fixed setError() visibility (private -> public for PDOStatement access)
 * - Added ATTR_DEFAULT_FETCH_MODE support
 * - Fixed SQL injection vulnerability in lastInsertId()
 * - Improved execute() logic with query type detection
 * - Added Iterator support for foreach loops
 * - Full transaction support! (beginTransaction/commit/rollback work correctly)
 * - Uses new Sqlx imperative API with optional callback
 *
 * NEW IN VERSION 3.0:
 * PDO-style transactions are now fully functional thanks to Rust API improvements!
 * Transactions are created in the database and correctly committed/rolled back.
 *
 * LIMITATIONS:
 * - Some PDO attributes are not supported
 * - FETCH_BOTH and FETCH_NUM are converted to FETCH_ASSOC or use array_values()
 */
class PDO
{
    /** Error mode constants */
    const ERRMODE_SILENT = 0;
    const ERRMODE_WARNING = 1;
    const ERRMODE_EXCEPTION = 2;

    /** Fetch mode constants */
    const FETCH_BOTH = 0; // Not directly supported, will use FETCH_ASSOC
    const FETCH_ASSOC = 2;
    const FETCH_NUM = 3; // Not directly supported, uses array_values() on FETCH_ASSOC
    const FETCH_OBJ = 5;

    /** Parameter type constants */
    const PARAM_INT = 1;
    const PARAM_STR = 2;
    const PARAM_BOOL = 5;
    const PARAM_NULL = 0;
    // Other types (LOB, DECIMAL, etc.) are not supported in this wrapper

    /** Attribute constants */
    const ATTR_ERRMODE = 3;
    const ATTR_DEFAULT_FETCH_MODE = 19;
    const ATTR_DRIVER_NAME = 16;

    /** @var \Sqlx\MySqlDriver|\Sqlx\PgDriver|\Sqlx\MssqlDriver $driver */
    private $driver;

    /** @var int $errorMode Current error handling mode */
    private $errorMode = self::ERRMODE_SILENT;

    /** @var int $defaultFetchMode Default fetch mode */
    private $defaultFetchMode = self::FETCH_BOTH;

    /** @var array $errorCode Last error code */
    private $errorCode = [0, '', ''];

    /** @var bool $inTransaction Active transaction flag */
    private $inTransaction = false;

    /**
     * Creates a PDO instance and connects to the database.
     *
     * @param string $dsn Connection string (e.g., 'mysql:host=localhost;dbname=test')
     * @param string $username Username
     * @param string $password Password
     * @param array $options Driver options
     * @throws PDOException If connection fails
     */
    public function __construct($dsn, $username = "", $password = "", $options = [])
    {
        // Convert PDO DSN to URL format understood by Sqlx\DriverFactory
        $url = $this->convertDsnToUrl($dsn, $username, $password);

        // Add options if present
        if (!empty($options)) {
            $url .= '?' . http_build_query($options);
        }

        try {
            $this->driver = \Sqlx\DriverFactory::make($url);
        } catch (\Exception $e) {
            $this->setError('HY000', $e->getMessage());
            throw new PDOException($e->getMessage(), 'HY000');
        }
    }

    /**
     * Converts PDO DSN to Sqlx URL format.
     * Supports main schemes: mysql, pgsql, sqlsrv.
     *
     * @param string $dsn
     * @param string $username
     * @param string $password
     * @return string
     */
    private function convertDsnToUrl($dsn, $username, $password)
    {
        // Simple DSN parsing (can be improved)
        if (strpos($dsn, ':') === false) {
            throw new \InvalidArgumentException("Invalid DSN format");
        }

        list($scheme, $paramsStr) = explode(':', $dsn, 2);

        // PDO scheme to Sqlx scheme mapping
        $schemeMap = [
            'mysql' => 'mysql',
            'pgsql' => 'postgres',
            'sqlsrv' => 'mssql',
        ];

        if (!isset($schemeMap[$scheme])) {
            throw new \InvalidArgumentException("Unsupported DSN scheme: $scheme");
        }

        $sqlxScheme = $schemeMap[$scheme];

        // Parse DSN parameters (host, dbname, port, etc.)
        $params = [];
        foreach (explode(';', $paramsStr) as $pair) {
            if (empty(trim($pair))) continue;
            if (strpos($pair, '=') !== false) {
                list($key, $value) = explode('=', $pair, 2);
                $params[trim($key)] = trim($value);
            }
        }

        $host = $params['host'] ?? 'localhost';
        $port = $params['port'] ?? ($scheme === 'pgsql' ? 5432 : ($scheme === 'mysql' ? 3306 : 1433));
        $dbname = $params['dbname'] ?? '';

        // Build URL
        $userPass = '';
        if ($username !== '') {
            $userPass = rawurlencode($username);
            if ($password !== '') {
                $userPass .= ':' . rawurlencode($password);
            }
            $userPass .= '@';
        }

        $path = $dbname ? '/' . ltrim($dbname, '/') : '';

        return "$sqlxScheme://$userPass$host:$port$path";
    }

    /**
     * Prepares an SQL statement and returns a PDOStatement object.
     *
     * @param string $statement SQL query
     * @param array $driver_options Ignored in this wrapper
     * @return PDOStatement|false
     */
    public function prepare($statement, $driver_options = [])
    {
        try {
            $preparedQuery = $this->driver->prepare($statement);
            return new PDOStatement($this, $preparedQuery, $statement);
        } catch (\Exception $e) {
            $this->setError('HY000', $e->getMessage());
            return false;
        }
    }

    /**
     * Executes an SQL statement and returns the number of affected rows.
     *
     * @param string $statement SQL query
     * @param array $input_parameters Parameters to bind
     * @return int|false Number of rows or false on error
     */
    public function exec($statement, $input_parameters = [])
    {
        try {
            $result = $this->driver->execute($statement, $input_parameters);
            $this->setError('00000'); // Success
            return $result;
        } catch (\Exception $e) {
            $this->setError('HY000', $e->getMessage());
            return false;
        }
    }

    /**
     * Executes an SQL query and returns the result set as a PDOStatement object.
     *
     * @param string $statement SQL query
     * @param int $mode Fetch mode (FETCH_ASSOC, FETCH_OBJ)
     * @param mixed $arg1 Ignored
     * @param array $ctor_args Ignored
     * @return PDOStatementFake|false Array/object or false
     */
    public function query($statement, $mode = self::FETCH_BOTH, $arg1 = null, $ctor_args = [])
    {
        try {
            // Determine which method to call based on mode
            if ($mode === self::FETCH_OBJ) {
                $rows = $this->driver->queryAllObj($statement);
            } else {
                // FETCH_BOTH and FETCH_NUM are converted to FETCH_ASSOC
                $rows = $this->driver->queryAllAssoc($statement);
            }

            if (empty($rows)) {
                $this->setError('00000'); // Success, but no rows
                return false;
            }

            $this->setError('00000'); // Success
            return new PDOStatementFake($rows, $mode);
        } catch (\Exception $e) {
            $this->setError('HY000', $e->getMessage());
            return false;
        }
    }

    /**
     * Returns the last inserted ID.
     * Implementation depends on the DBMS. Uses generic logic here.
     *
     * @param string $name Sequence name (for PostgreSQL)
     * @return string
     */
    public function lastInsertId($name = null)
    {
        try {
            if ($name) {
                // Assume PostgreSQL - use parameterized query
                $result = $this->driver->queryValue("SELECT CURRVAL(?)", [$name]);
            } else {
                // Assume MySQL or MSSQL
                $result = $this->driver->queryValue("SELECT LAST_INSERT_ID()");
                if ($result === null) {
                    $result = $this->driver->queryValue("SELECT @@IDENTITY"); // For MSSQL
                }
            }
            return (string)$result;
        } catch (\Exception $e) {
            $this->setError('HY000', $e->getMessage());
            return '0';
        }
    }

    /**
     * Begins a transaction.
     *
     * Uses the new Sqlx imperative API with optional callback.
     * Creates a real database transaction that can be committed via commit()
     * or rolled back via rollback().
     *
     * @return bool
     */
    public function beginTransaction()
    {
        if ($this->inTransaction) {
            $this->setError('HY000', 'There is already an active transaction');
            return false;
        }

        try {
            // Call begin() without callback for imperative-style transactions
            $this->driver->begin();
            $this->inTransaction = true;
            $this->setError('00000');
            return true;
        } catch (\Exception $e) {
            $this->setError('HY000', $e->getMessage());
            return false;
        }
    }

    /**
     * Commits a transaction.
     *
     * Uses the native commit() method of the Sqlx driver.
     *
     * @return bool
     */
    public function commit()
    {
        if (!$this->inTransaction) {
            $this->setError('HY000', 'There is no active transaction');
            return false;
        }

        try {
            $this->driver->commit();
            $this->inTransaction = false;
            $this->setError('00000');
            return true;
        } catch (\Exception $e) {
            $this->setError('HY000', $e->getMessage());
            return false;
        }
    }

    /**
     * Rolls back a transaction.
     *
     * Uses the native rollback() method of the Sqlx driver.
     *
     * @return bool
     */
    public function rollBack()
    {
        if (!$this->inTransaction) {
            $this->setError('HY000', 'There is no active transaction');
            return false;
        }

        try {
            $this->driver->rollback();
            $this->inTransaction = false;
            $this->setError('00000');
            return true;
        } catch (\Exception $e) {
            $this->setError('HY000', $e->getMessage());
            return false;
        }
    }

    /**
     * Checks if inside a transaction.
     *
     * @return bool
     */
    public function inTransaction()
    {
        return $this->inTransaction;
    }

    /**
     * Sets an attribute.
     *
     * @param int $attribute
     * @param mixed $value
     * @return bool
     */
    public function setAttribute($attribute, $value)
    {
        switch ($attribute) {
            case self::ATTR_ERRMODE:
                if (in_array($value, [self::ERRMODE_SILENT, self::ERRMODE_WARNING, self::ERRMODE_EXCEPTION])) {
                    $this->errorMode = $value;
                    return true;
                }
                break;
            case self::ATTR_DEFAULT_FETCH_MODE:
                if (in_array($value, [self::FETCH_BOTH, self::FETCH_ASSOC, self::FETCH_NUM, self::FETCH_OBJ])) {
                    $this->defaultFetchMode = $value;
                    return true;
                }
                break;
            // Other attributes can be added as needed
        }
        $this->setError('HY000', 'Unsupported attribute or value');
        return false;
    }

    /**
     * Gets an attribute value.
     *
     * @param int $attribute
     * @return mixed
     */
    public function getAttribute($attribute)
    {
        switch ($attribute) {
            case self::ATTR_ERRMODE:
                return $this->errorMode;
            case self::ATTR_DEFAULT_FETCH_MODE:
                return $this->defaultFetchMode;
            case self::ATTR_DRIVER_NAME:
                // Determine driver type by class name
                $driverClass = get_class($this->driver);
                if (strpos($driverClass, 'MySql') !== false) {
                    return 'mysql';
                } elseif (strpos($driverClass, 'Pg') !== false) {
                    return 'pgsql';
                } elseif (strpos($driverClass, 'Mssql') !== false) {
                    return 'sqlsrv';
                }
                return 'unknown';
            // Other attributes can be added as needed
        }
        $this->setError('HY000', 'Unsupported attribute');
        return null;
    }

    /**
     * Returns error information about the last operation.
     *
     * @return array
     */
    public function errorInfo()
    {
        return $this->errorCode;
    }

    /**
     * Sets the internal error code.
     * Public for access from PDOStatement.
     *
     * @param string $sqlState
     * @param string $message
     * @internal
     */
    public function setError($sqlState, $message = '')
    {
        $this->errorCode = [$sqlState, '', $message];

        if ($this->errorMode === self::ERRMODE_WARNING && $sqlState !== '00000') {
            trigger_error($message, E_USER_WARNING);
        } elseif ($this->errorMode === self::ERRMODE_EXCEPTION && $sqlState !== '00000') {
            throw new PDOException($message, $sqlState);
        }
    }
}

/**
 * PDO Exception class.
 */
class PDOException extends \Exception
{
    public $errorInfo = [];

    public function __construct($message, $code, ?\Exception $previous = null)
    {
        parent::__construct($message, 0, $previous);
        $this->errorInfo = [$code, '', $message];
        $this->code = $code;
    }
}

/**
 * Fake PDOStatement class for emulating PDO::query() results.
 * Stores already fetched data.
 * Implements Iterator for foreach support.
 */
class PDOStatementFake implements \Iterator
{
    private $rows;
    private $mode;
    private $pointer = 0;

    public function __construct($rows, $mode)
    {
        $this->rows = $rows;
        $this->mode = $mode;
        $this->pointer = 0;
    }

    public function fetch($fetch_style = null, $cursor_orientation = null, $cursor_offset = null)
    {
        if (!isset($this->rows[$this->pointer])) {
            return false;
        }

        $row = $this->rows[$this->pointer];
        $this->pointer++;

        // Emulate FETCH_NUM
        if ($fetch_style === \PdoEmulator\PDO::FETCH_NUM && is_array($row)) {
            return array_values($row);
        }

        return $row;
    }

    public function fetchAll($how = null, $class_name = null, $ctor_args = [])
    {
        $result = [];
        $originalPointer = $this->pointer;
        $this->pointer = 0;

        while (($row = $this->fetch($how)) !== false) {
            $result[] = $row;
        }

        $this->pointer = $originalPointer;
        return $result;
    }

    /**
     * Returns the number of rows.
     *
     * @return int
     */
    public function rowCount()
    {
        return count($this->rows);
    }

    // Iterator interface implementation

    /**
     * Returns the current element.
     * @return mixed
     */
    public function current(): mixed
    {
        return $this->rows[$this->pointer] ?? false;
    }

    /**
     * Returns the key of the current element.
     * @return int
     */
    public function key(): mixed
    {
        return $this->pointer;
    }

    /**
     * Moves to the next element.
     * @return void
     */
    public function next(): void
    {
        $this->pointer++;
    }

    /**
     * Rewinds the iterator to the first element.
     * @return void
     */
    public function rewind(): void
    {
        $this->pointer = 0;
    }

    /**
     * Checks if the current position is valid.
     * @return bool
     */
    public function valid(): bool
    {
        return isset($this->rows[$this->pointer]);
    }
}

/**
 * Main class for prepared statements.
 * Implements Iterator for foreach support.
 */
class PDOStatement implements \Iterator
{
    /** @var PDO $pdo */
    private $pdo;
    /** @var \Sqlx\MySqlPreparedQuery|\Sqlx\PgPreparedQuery|\Sqlx\Driver\MssqlPreparedQuery $preparedQuery */
    private $preparedQuery;
    private $queryString;
    private $boundParams = [];
    private $fetchedRows = [];
    private $fetchPointer = 0;
    private $executed = false;

    public function __construct(PDO $pdo, $preparedQuery, $queryString)
    {
        $this->pdo = $pdo;
        $this->preparedQuery = $preparedQuery;
        $this->queryString = $queryString;
    }

    /**
     * Binds a parameter to a variable reference.
     *
     * @param mixed $param Parameter name (:name) or number (?)
     * @param mixed $var Variable to bind
     * @param int $type Data type (ignored in this wrapper)
     * @param int $maxLength Ignored
     * @param mixed $driverOptions Ignored
     * @return bool
     */
    public function bindParam($param, &$var, $type = \PdoEmulator\PDO::PARAM_STR, $maxLength = null, $driverOptions = null)
    {
        // Store reference to variable
        $this->boundParams[$param] =& $var;
        return true;
    }

    /**
     * Binds a value to a parameter.
     *
     * @param mixed $param Parameter name (:name) or number (?)
     * @param mixed $value Value
     * @param int $type Data type (ignored)
     * @return bool
     */
    public function bindValue($param, $value, $type = \PdoEmulator\PDO::PARAM_STR)
    {
        $this->boundParams[$param] = $value;
        return true;
    }

    /**
     * Executes the prepared statement.
     *
     * @param array $input_parameters Associative array of parameters
     * @return bool
     */
    public function execute($input_parameters = null)
    {
        $params = $input_parameters !== null ? $input_parameters : $this->boundParams;

        // Determine query type by first keyword
        $queryType = strtoupper(trim(preg_replace('/\s+/', ' ', $this->queryString)));
        $firstWord = explode(' ', $queryType)[0];

        try {
            if (in_array($firstWord, ['SELECT', 'SHOW', 'DESCRIBE', 'EXPLAIN', 'WITH'])) {
                // This is a SELECT query
                $this->fetchedRows = $this->preparedQuery->queryAll($params);
                $this->fetchPointer = 0;
                $this->executed = true;
                $this->pdo->setError('00000');
                return true;
            } else {
                // This is a data modification query (INSERT, UPDATE, DELETE, etc.)
                $this->preparedQuery->execute($params);
                $this->fetchedRows = [];
                $this->fetchPointer = 0;
                $this->executed = true;
                $this->pdo->setError('00000');
                return true;
            }
        } catch (\Exception $e) {
            $this->pdo->setError('HY000', $e->getMessage());
            return false;
        }
    }

    /**
     * Fetches the next row from the result set.
     *
     * @param int $fetch_style
     * @param int $cursor_orientation
     * @param int $cursor_offset
     * @return mixed|false
     */
    public function fetch($fetch_style = null, $cursor_orientation = null, $cursor_offset = null)
    {
        if (!$this->executed) {
            $this->pdo->setError('HY000', 'Statement not executed');
            return false;
        }

        // Determine fetch mode
        if ($fetch_style === null) {
            $fetch_style = $this->pdo->getAttribute(\PdoEmulator\PDO::ATTR_DEFAULT_FETCH_MODE) ?? \PdoEmulator\PDO::FETCH_BOTH;
        }

        if ($fetch_style === \PdoEmulator\PDO::FETCH_OBJ) {
            if (!isset($this->fetchedRows[$this->fetchPointer])) {
                return false;
            }
            $row = $this->fetchedRows[$this->fetchPointer];
            $this->fetchPointer++;
            return (object)$row;
        } else {
            // FETCH_BOTH and FETCH_NUM are converted to FETCH_ASSOC
            if (!isset($this->fetchedRows[$this->fetchPointer])) {
                return false;
            }
            $row = $this->fetchedRows[$this->fetchPointer];
            $this->fetchPointer++;

            if ($fetch_style === \PdoEmulator\PDO::FETCH_NUM) {
                return array_values($row);
            }

            return $row; // FETCH_ASSOC
        }
    }

    /**
     * Fetches all rows from the result set.
     *
     * @param int $how
     * @param mixed $class_name
     * @param array $ctor_args
     * @return array
     */
    public function fetchAll($how = null, $class_name = null, $ctor_args = [])
    {
        if (!$this->executed) {
            $this->pdo->setError('HY000', 'Statement not executed');
            return [];
        }

        $result = [];
        $originalPointer = $this->fetchPointer;
        $this->fetchPointer = 0;

        while (($row = $this->fetch($how)) !== false) {
            $result[] = $row;
        }

        $this->fetchPointer = $originalPointer;
        return $result;
    }

    /**
     * Returns the number of rows affected by the last SQL statement.
     *
     * @return int
     */
    public function rowCount()
    {
        // For SELECT, rowCount is unreliable in PDO; here it returns fetched row count.
        if ($this->executed && !empty($this->fetchedRows)) {
            return count($this->fetchedRows);
        }
        // For INSERT/UPDATE/DELETE, rowCount should return affected rows,
        // but in current implementation we don't store that result.
        // This is a limitation of this wrapper.
        return 0;
    }

    /**
     * Returns column metadata.
     *
     * @param int $column
     * @return array|false
     */
    public function getColumnMeta($column)
    {
        // This function is not implemented as Sqlx doesn't provide column metadata
        // in stubs. Returns false, same as PDO when metadata is unavailable.
        return false;
    }

    /**
     * Closes the cursor, freeing resources.
     *
     * @return bool
     */
    public function closeCursor()
    {
        $this->fetchedRows = [];
        $this->fetchPointer = 0;
        $this->executed = false;
        return true;
    }

    /**
     * Returns the number of columns in the result set.
     *
     * @return int
     */
    public function columnCount()
    {
        if (!$this->executed || empty($this->fetchedRows)) {
            return 0;
        }
        return count($this->fetchedRows[0]);
    }

    /**
     * Returns error information for this statement.
     *
     * @return array
     */
    public function errorInfo()
    {
        return $this->pdo->errorInfo();
    }

    // Iterator interface implementation

    /**
     * Returns the current element.
     * @return mixed
     */
    public function current(): mixed
    {
        return $this->fetchedRows[$this->fetchPointer] ?? false;
    }

    /**
     * Returns the key of the current element.
     * @return int
     */
    public function key(): mixed
    {
        return $this->fetchPointer;
    }

    /**
     * Moves to the next element.
     * @return void
     */
    public function next(): void
    {
        $this->fetchPointer++;
    }

    /**
     * Rewinds the iterator to the first element.
     * @return void
     */
    public function rewind(): void
    {
        $this->fetchPointer = 0;
    }

    /**
     * Checks if the current position is valid.
     * @return bool
     */
    public function valid(): bool
    {
        return isset($this->fetchedRows[$this->fetchPointer]);
    }
}
