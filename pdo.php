<?php
namespace PdoEmulator;
/**
 * Базовый эмулятор PDO, использующий библиотеку Sqlx.
 * Предназначен для облегчения перехода существующего кода.
 */
class PDO
{
    /** Константы режимов ошибок */
    const ERRMODE_SILENT = 0;
    const ERRMODE_WARNING = 1;
    const ERRMODE_EXCEPTION = 2;

    /** Константы выбора результата */
    const FETCH_BOTH = 0; // Не поддерживается напрямую, будет FETCH_ASSOC
    const FETCH_ASSOC = 2;
    const FETCH_NUM = 3; // Не поддерживается напрямую, будет FETCH_ASSOC
    const FETCH_OBJ = 5;

    /** Константы привязки параметров */
    const PARAM_INT = 1;
    const PARAM_STR = 2;
    const PARAM_BOOL = 5;
    const PARAM_NULL = 0;
    // Другие типы (LOB, DECIMAL и т.д.) не поддерживаются в этой обертке

    /** @var \Sqlx\MySqlDriver|\Sqlx\PgDriver|\Sqlx\MssqlDriver $driver */
    private $driver;

    /** @var int $errorMode Текущий режим обработки ошибок */
    private $errorMode = self::ERRMODE_SILENT;

    /** @var array $errorCode Последний код ошибки */
    private $errorCode = [0, '', ''];

    /** @var bool $inTransaction Флаг активной транзакции */
    private $inTransaction = false;

    /**
     * Создает экземпляр PDO, подключаясь к базе данных.
     *
     * @param string $dsn Строка подключения (например, 'mysql:host=localhost;dbname=test')
     * @param string $username Имя пользователя
     * @param string $password Пароль
     * @param array $options Опции драйвера
     * @throws PDOException Если подключение не удалось
     */
    public function __construct($dsn, $username = "", $password = "", $options = [])
    {
        // Преобразуем DSN PDO в формат URL, понятный Sqlx\DriverFactory
        $url = $this->convertDsnToUrl($dsn, $username, $password);

        // Добавляем опции, если они есть
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
     * Преобразует DSN PDO в URL для Sqlx.
     * Поддерживает основные схемы: mysql, pgsql, sqlsrv.
     *
     * @param string $dsn
     * @param string $username
     * @param string $password
     * @return string
     */
    private function convertDsnToUrl($dsn, $username, $password)
    {
        // Простой парсинг DSN (можно улучшить)
        if (strpos($dsn, ':') === false) {
            throw new \InvalidArgumentException("Invalid DSN format");
        }

        list($scheme, $paramsStr) = explode(':', $dsn, 2);

        // Отображение схем PDO на схемы Sqlx
        $schemeMap = [
            'mysql' => 'mysql',
            'pgsql' => 'postgres',
            'sqlsrv' => 'mssql',
        ];

        if (!isset($schemeMap[$scheme])) {
            throw new \InvalidArgumentException("Unsupported DSN scheme: $scheme");
        }

        $sqlxScheme = $schemeMap[$scheme];

        // Разбор параметров DSN (host, dbname, port и т.д.)
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

        // Формируем URL
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
     * Выполняет SQL-запрос и возвращает объект PDOStatement.
     *
     * @param string $statement SQL-запрос
     * @param array $driver_options Игнорируется в этой обертке
     * @return PDOStatement
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
     * Выполняет SQL-запрос и возвращает количество затронутых строк.
     *
     * @param string $statement SQL-запрос
     * @param array $input_parameters Параметры для привязки
     * @return int|false Количество строк или false в случае ошибки
     */
    public function exec($statement, $input_parameters = [])
    {
        try {
            $result = $this->driver->execute($statement, $input_parameters);
            $this->setError('00000'); // Успешно
            return $result;
        } catch (\Exception $e) {
            $this->setError('HY000', $e->getMessage());
            return false;
        }
    }

    /**
     * Выполняет SQL-запрос и возвращает первую строку результата.
     *
     * @param string $statement SQL-запрос
     * @param int $mode Режим выборки (FETCH_ASSOC, FETCH_OBJ)
     * @param mixed $arg1 Игнорируется
     * @param array $ctor_args Игнорируется
     * @return mixed|false Массив/объект или false
     */
    public function query($statement, $mode = self::FETCH_BOTH, $arg1 = null, $ctor_args = [])
    {
        try {
            // Определяем, какой метод вызывать в зависимости от режима
            if ($mode === self::FETCH_OBJ) {
                $rows = $this->driver->queryAllObj($statement);
            } else {
                // FETCH_BOTH и FETCH_NUM преобразуются в FETCH_ASSOC
                $rows = $this->driver->queryAllAssoc($statement);
            }

            if (empty($rows)) {
                $this->setError('00000'); // Успешно, но строк нет
                return false;
            }

            $this->setError('00000'); // Успешно
            return new PDOStatementFake($rows, $mode);
        } catch (\Exception $e) {
            $this->setError('HY000', $e->getMessage());
            return false;
        }
    }

    /**
     * Возвращает последний вставленный ID.
     * Реализация зависит от СУБД. Здесь используется общая логика.
     *
     * @param string $name Имя последовательности (для PostgreSQL)
     * @return string
     */
    public function lastInsertId($name = null)
    {
        try {
            if ($name) {
                // Предполагаем, что это PostgreSQL
                $result = $this->driver->queryValue("SELECT CURRVAL('$name')");
            } else {
                // Предполагаем MySQL или MSSQL
                $result = $this->driver->queryValue("SELECT LAST_INSERT_ID()");
                if ($result === null) {
                    $result = $this->driver->queryValue("SELECT @@IDENTITY"); // Для MSSQL
                }
            }
            return (string)$result;
        } catch (\Exception $e) {
            $this->setError('HY000', $e->getMessage());
            return '0';
        }
    }

    /**
     * Начинает транзакцию.
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
            // Используем callable-подход драйвера для начала транзакции
            $this->driver->begin(function ($driver) {
                // Просто помечаем, что транзакция активна.
                // Реальная логика commit/rollback будет в методах ниже.
                $this->inTransaction = true;
                // Возвращаем true, чтобы транзакция не откатилась автоматически.
                return true;
            });
            $this->setError('00000');
            return true;
        } catch (\Exception $e) {
            $this->setError('HY000', $e->getMessage());
            return false;
        }
    }

    /**
     * Фиксирует транзакцию.
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
            // В Sqlx нет прямого аналога commit, но транзакция фиксируется при успешном выходе из callable.
            // Для эмуляции, мы просто помечаем транзакцию как завершенную.
            // Это НЕ идеальное решение, но в рамках callable-подхода Sqlx это сложно сделать иначе.
            $this->inTransaction = false;
            $this->setError('00000');
            return true;
        } catch (\Exception $e) {
            $this->setError('HY000', $e->getMessage());
            return false;
        }
    }

    /**
     * Откатывает транзакцию.
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
            // Аналогично commit, мы просто помечаем транзакцию как завершенную.
            // Настоящий откат должен был произойти внутри callable, если вернуть false.
            $this->inTransaction = false;
            $this->setError('00000');
            return true;
        } catch (\Exception $e) {
            $this->setError('HY000', $e->getMessage());
            return false;
        }
    }

    /**
     * Проверяет, находится ли соединение в активной транзакции.
     *
     * @return bool
     */
    public function inTransaction()
    {
        return $this->inTransaction;
    }

    /**
     * Устанавливает атрибут.
     *
     * @param int $attribute
     * @param mixed $value
     * @return bool
     */
    public function setAttribute($attribute, $value)
    {
        switch ($attribute) {
            case \PDO::ATTR_ERRMODE:
                if (in_array($value, [self::ERRMODE_SILENT, self::ERRMODE_WARNING, self::ERRMODE_EXCEPTION])) {
                    $this->errorMode = $value;
                    return true;
                }
                break;
            // Другие атрибуты (PDO::ATTR_DEFAULT_FETCH_MODE и т.д.) можно добавить по мере необходимости
        }
        $this->setError('HY000', 'Unsupported attribute or value');
        return false;
    }

    /**
     * Возвращает значение атрибута.
     *
     * @param int $attribute
     * @return mixed
     */
    public function getAttribute($attribute)
    {
        switch ($attribute) {
            case \PDO::ATTR_ERRMODE:
                return $this->errorMode;
            case \PDO::ATTR_DRIVER_NAME:
                // Определяем тип драйвера по классу
                $driverClass = get_class($this->driver);
                if (strpos($driverClass, 'MySql') !== false) {
                    return 'mysql';
                } elseif (strpos($driverClass, 'Pg') !== false) {
                    return 'pgsql';
                } elseif (strpos($driverClass, 'Mssql') !== false) {
                    return 'sqlsrv';
                }
                return 'unknown';
            // Другие атрибуты можно добавить по мере необходимости
        }
        $this->setError('HY000', 'Unsupported attribute');
        return null;
    }

    /**
     * Возвращает информацию о последней ошибке.
     *
     * @return array
     */
    public function errorInfo()
    {
        return $this->errorCode;
    }

    /**
     * Устанавливает внутренний код ошибки.
     *
     * @param string $sqlState
     * @param string $message
     */
    private function setError($sqlState, $message = '')
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
 * Исключение PDO.
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
 * Класс для эмуляции PDOStatement, возвращаемого методом PDO::query.
 * Хранит уже выбранные данные.
 */
class PDOStatementFake
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

        // PDO::query не поддерживает FETCH_NUM напрямую, но мы эмулируем
        if ($fetch_style === PDO::FETCH_NUM && is_array($row)) {
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

    // Другие методы (rowCount, columnCount и т.д.) можно реализовать по мере необходимости
    public function rowCount()
    {
        return count($this->rows);
    }
}

/**
 * Основной класс для подготовленных выражений.
 */
class PDOStatement
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
     * Привязывает параметр к переменной.
     *
     * @param mixed $param Имя параметра (:name) или номер (?)
     * @param mixed $var Переменная для привязки
     * @param int $type Тип данных (игнорируется в этой обертке)
     * @param int $maxLength Игнорируется
     * @param mixed $driverOptions Игнорируется
     * @return bool
     */
    public function bindParam($param, &$var, $type = PDO::PARAM_STR, $maxLength = null, $driverOptions = null)
    {
        // Сохраняем ссылку на переменную
        $this->boundParams[$param] =& $var;
        return true;
    }

    /**
     * Привязывает значение к параметру.
     *
     * @param mixed $param Имя параметра (:name) или номер (?)
     * @param mixed $value Значение
     * @param int $type Тип данных (игнорируется)
     * @return bool
     */
    public function bindValue($param, $value, $type = PDO::PARAM_STR)
    {
        $this->boundParams[$param] = $value;
        return true;
    }

    /**
     * Выполняет подготовленное выражение.
     *
     * @param array $input_parameters Ассоциативный массив параметров
     * @return bool
     */
    public function execute($input_parameters = null)
    {
        $params = $input_parameters !== null ? $input_parameters : $this->boundParams;

        // Преобразуем именованные параметры :name в формат, который ожидает Sqlx (обычно :name или $name)
        // Sqlx, судя по стабам, принимает ассоциативные массивы, так что оставляем как есть.
        // Если ваш драйвер требует другого формата (например, $1, $2), нужна дополнительная конвертация.

        try {
            // Пытаемся выполнить запрос как запрос на изменение данных (INSERT, UPDATE, DELETE)
            $result = $this->preparedQuery->execute($params);
            $this->executed = true;
            $this->pdo->setError('00000');
            return true;
        } catch (\Exception $e) {
            // Если execute не сработал, возможно, это SELECT. Пытаемся получить все строки.
            try {
                $this->fetchedRows = $this->preparedQuery->queryAll($params);
                $this->fetchPointer = 0;
                $this->executed = true;
                $this->pdo->setError('00000');
                return true;
            } catch (\Exception $e2) {
                $this->pdo->setError('HY000', $e2->getMessage());
                return false;
            }
        }
    }

    /**
     * Выбирает следующую строку из результирующего набора.
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

        // Определяем режим выборки
        if ($fetch_style === null) {
            $fetch_style = $this->pdo->getAttribute(PDO::ATTR_DEFAULT_FETCH_MODE) ?? PDO::FETCH_BOTH;
        }

        if ($fetch_style === PDO::FETCH_OBJ) {
            if (!isset($this->fetchedRows[$this->fetchPointer])) {
                return false;
            }
            $row = $this->fetchedRows[$this->fetchPointer];
            $this->fetchPointer++;
            return (object)$row;
        } else {
            // PDO::FETCH_BOTH и PDO::FETCH_NUM преобразуются в FETCH_ASSOC
            if (!isset($this->fetchedRows[$this->fetchPointer])) {
                return false;
            }
            $row = $this->fetchedRows[$this->fetchPointer];
            $this->fetchPointer++;

            if ($fetch_style === PDO::FETCH_NUM) {
                return array_values($row);
            }

            return $row; // PDO::FETCH_ASSOC
        }
    }

    /**
     * Выбирает все строки из результирующего набора.
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
     * Возвращает количество строк, затронутых последним SQL-запросом.
     *
     * @return int
     */
    public function rowCount()
    {
        // Для SELECT rowCount не надежен в PDO, и здесь он возвращает количество выбранных строк.
        if ($this->executed && !empty($this->fetchedRows)) {
            return count($this->fetchedRows);
        }
        // Для INSERT/UPDATE/DELETE rowCount должен возвращать количество затронутых строк,
        // но в текущей реализации execute мы не сохраняем этот результат.
        // Это ограничение данной обертки.
        return 0;
    }

    /**
     * Возвращает информацию о столбце.
     *
     * @param int $column
     * @return array|false
     */
    public function getColumnMeta($column)
    {
        // Эта функция не реализована, так как Sqlx не предоставляет метаданные столбцов
        // в стабах. Возвращает false, как и PDO, если метаданные недоступны.
        return false;
    }

    /**
     * Закрывает курсор, освобождая ресурсы.
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
     * Возвращает количество столбцов в результирующем наборе.
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
     * Возвращает информацию об ошибке для этого statement.
     *
     * @return array
     */
    public function errorInfo()
    {
        return $this->pdo->errorInfo();
    }
}
