# PDO Emulator for Sqlx

A PDO-compatible wrapper for the Sqlx PHP extension, designed to ease migration of existing PDO-based code.

## Usage Example

```php
<?php
require_once 'pdo.php';

// Create connection
$pdo = new PdoEmulator\PDO('pgsql:host=localhost;dbname=mydb', 'username', 'password');

// Set attributes
$pdo->setAttribute(PDO::ATTR_ERRMODE, PdoEmulator\PDO::ERRMODE_EXCEPTION);
$pdo->setAttribute(PDO::ATTR_DEFAULT_FETCH_MODE, PdoEmulator\PDO::FETCH_ASSOC);

// Simple query
$stmt = $pdo->query("SELECT * FROM users WHERE active = true");
foreach ($stmt as $row) {
    echo $row['name'] . "\n";
}

// Prepared statement with positional parameters
$stmt = $pdo->prepare("SELECT * FROM users WHERE id = ?");
$stmt->execute([123]);
$user = $stmt->fetch();

// Prepared statement with named parameters
$stmt = $pdo->prepare("SELECT * FROM posts WHERE author_id = :author_id AND status = :status");
$stmt->execute(['author_id' => 456, 'status' => 'published']);
$posts = $stmt->fetchAll();

// Insert and get last ID
$stmt = $pdo->prepare("INSERT INTO users (name, email) VALUES (?, ?)");
$stmt->execute(['John Doe', 'john@example.com']);
$lastId = $pdo->lastInsertId();
```

## Transaction Handling - IMPORTANT

### Limitation

PDO-style transactions (`beginTransaction()`, `commit()`, `rollBack()`) have **limited functionality** due to architectural differences between PDO and Sqlx:

- **PDO** uses explicit begin/commit/rollback calls
- **Sqlx** uses a callback-based transaction API

The current implementation only tracks transaction state internally but **does not create real database transactions**.

### Recommended Approach

For real transaction support, use the native Sqlx API:

```php
<?php
$driver = \Sqlx\DriverFactory::make('postgres://user:pass@localhost/db');

$driver->begin(function($driver) {
    // All queries here run in a transaction
    $driver->execute('INSERT INTO users (name) VALUES (?)', ['John']);
    $driver->execute('UPDATE accounts SET balance = balance - 100 WHERE user_id = ?)', [1]);

    // Return true to commit, false to rollback
    return true;
});
```

If an exception is thrown inside the callback, the transaction is automatically rolled back.

### Savepoints

For nested transaction-like behavior, use savepoints:

```php
<?php
$driver->begin(function($driver) {
    $driver->execute('INSERT INTO users (name) VALUES (?)', ['John']);

    // Create savepoint
    $driver->savepoint('sp1');

    try {
        $driver->execute('INSERT INTO logs (message) VALUES (?)', ['Action logged']);
    } catch (\Exception $e) {
        // Rollback to savepoint
        $driver->rollbackToSavepoint('sp1');
    }

    // Release savepoint if no longer needed
    $driver->releaseSavepoint('sp1');

    return true;
});
```

## Supported Features

### Fetch Modes
- `FETCH_ASSOC` - Associative array
- `FETCH_OBJ` - Object (stdClass)
- `FETCH_BOTH` - Converted to FETCH_ASSOC (not true both)
- `FETCH_NUM` - Numeric array (via array_values)

### Error Modes
- `ERRMODE_SILENT` - No errors reported
- `ERRMODE_WARNING` - PHP warnings
- `ERRMODE_EXCEPTION` - Throws PDOException

### Attributes
- `ATTR_ERRMODE` - Error reporting mode
- `ATTR_DEFAULT_FETCH_MODE` - Default fetch mode for statements
- `ATTR_DRIVER_NAME` - Database driver name (mysql, pgsql, sqlsrv)

### Methods

#### PDO Class
- `__construct($dsn, $username, $password, $options)` - Create connection
- `prepare($statement)` - Prepare a statement
- `query($statement, $mode)` - Execute query and return result set
- `exec($statement)` - Execute statement and return affected rows
- `lastInsertId($name)` - Get last inserted ID
- `beginTransaction()` - Begin transaction (⚠️ limited)
- `commit()` - Commit transaction (⚠️ limited)
- `rollBack()` - Rollback transaction (⚠️ limited)
- `inTransaction()` - Check if in transaction
- `setAttribute($attribute, $value)` - Set attribute
- `getAttribute($attribute)` - Get attribute
- `errorInfo()` - Get error information

#### PDOStatement Class
- `bindParam($param, &$var, $type)` - Bind parameter by reference
- `bindValue($param, $value, $type)` - Bind parameter by value
- `execute($params)` - Execute prepared statement
- `fetch($fetch_style)` - Fetch next row
- `fetchAll($fetch_style)` - Fetch all rows
- `rowCount()` - Get row count
- `columnCount()` - Get column count
- `closeCursor()` - Close cursor
- `errorInfo()` - Get error information
- Iterator methods: `current()`, `key()`, `next()`, `rewind()`, `valid()`

## Limitations

1. **Transactions** - PDO-style transaction methods do not create real database transactions
2. **FETCH_BOTH** - Not truly "both", converts to FETCH_ASSOC
3. **FETCH_NUM** - Implemented via array_values() on FETCH_ASSOC
4. **Parameter Types** - Type hints in `bindParam()` are ignored
5. **Column Metadata** - `getColumnMeta()` not implemented (returns false)
6. **Some PDO Attributes** - Not all PDO attributes are supported
7. **Statement Options** - `prepare()` driver options are ignored

## DSN Format

The emulator converts PDO DSN format to Sqlx URL format:

```php
// MySQL
'mysql:host=localhost;port=3306;dbname=mydb'
→ 'mysql://user:pass@localhost:3306/mydb'

// PostgreSQL
'pgsql:host=localhost;port=5432;dbname=mydb'
→ 'postgres://user:pass@localhost:5432/mydb'

// SQL Server
'sqlsrv:host=localhost;port=1433;dbname=mydb'
→ 'mssql://user:pass@localhost:1433/mydb'
```

## Testing

Run the comprehensive test suite:

```bash
php pdo-test.php
```

Or use the simple example:

```bash
php pdo.example.php
```

## Migration Guide

### From PDO to Sqlx Native API

If you want full Sqlx features, consider migrating:

```php
// PDO style
$stmt = $pdo->prepare("SELECT * FROM users WHERE id = ?");
$stmt->execute([123]);
$user = $stmt->fetch(PDO::FETCH_ASSOC);

// Sqlx native style
$driver = \Sqlx\DriverFactory::make('postgres://...');
$user = $driver->queryRowAssoc("SELECT * FROM users WHERE id = ?", [123]);
```

### Benefits of Native Sqlx API

- True transaction support with callbacks
- Better performance (no emulation overhead)
- Advanced features like query builders
- Conditional SQL blocks `{{ AND field = :param }}`
- Safe `IN (?)` clause expansion
- Native JSON support
- Connection pooling

See the main [README.md](README.md) for full Sqlx documentation.

## Contributing

When improving this emulator:

1. Maintain backward compatibility where possible
2. Document any limitations clearly
3. Add tests for new features
4. Update this README with changes

## License

MIT - Same as the main php-sqlx project