# Exception Types

php-sqlx uses a hierarchy of exceptions to provide detailed error information.

## Exception Hierarchy

```
SqlxException (base)
├── ConnectionException
├── QueryException
├── TransactionException
├── ParseException
├── ParameterException
├── ConfigurationException
├── ValidationException
├── NotPermittedException
├── TimeoutException
└── PoolExhaustedException
```

All exceptions extend `Sqlx\Exceptions\SqlxException`.

## SqlxException

The base exception class for all php-sqlx errors.

```php
use Sqlx\Exceptions\SqlxException;

try {
    $driver->queryRow("SELECT * FROM users");
} catch (SqlxException $e) {
    echo "Error: " . $e->getMessage();
    echo "Code: " . $e->getCode();

    if ($e->isTransient()) {
        // Can retry this operation
    }
}
```

### Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `getMessage()` | string | Error message |
| `getCode()` | int | Error code constant |
| `isTransient()` | bool | Whether error is temporary/retriable |
| `getSql()` | ?string | The SQL that caused the error (if applicable) |

## ConnectionException

Thrown when connection to the database fails.

```php
use Sqlx\Exceptions\ConnectionException;

try {
    $driver = DriverFactory::make("postgres://invalid-host/mydb");
} catch (ConnectionException $e) {
    echo "Could not connect: " . $e->getMessage();
}
```

**Common causes:**
- Invalid hostname or port
- Authentication failure
- Network issues
- Database server down
- SSL/TLS configuration errors

## QueryException

Thrown when a query fails to execute.

```php
use Sqlx\Exceptions\QueryException;

try {
    $driver->queryRow("SELECT * FROM nonexistent_table");
} catch (QueryException $e) {
    echo "Query failed: " . $e->getMessage();
    echo "SQL: " . $e->getSql();
}
```

**Common causes:**
- Table or column doesn't exist
- Syntax errors
- Constraint violations
- Permission denied
- Lock timeouts

## TransactionException

Thrown for transaction-related errors.

```php
use Sqlx\Exceptions\TransactionException;

try {
    $driver->begin(function($driver) {
        // ... operations that cause deadlock
        return true;
    });
} catch (TransactionException $e) {
    if ($e->isTransient()) {
        // Deadlock or serialization failure - can retry
    }
}
```

**Common causes:**
- Deadlocks
- Serialization failures
- Lock wait timeouts
- Transaction already in progress

## ParseException

Thrown when SQL parsing fails (before sending to database).

```php
use Sqlx\Exceptions\ParseException;

try {
    $driver->queryAll("SELECT * FROM users {{ AND status = }}");  // Incomplete block
} catch (ParseException $e) {
    echo "Parse error: " . $e->getMessage();
}
```

**Common causes:**
- Malformed conditional blocks `{{ }}`
- Invalid placeholder syntax
- Unmatched quotes or brackets

## ParameterException

Thrown when parameter validation fails.

```php
use Sqlx\Exceptions\ParameterException;

try {
    $driver->queryAll("SELECT * FROM users WHERE age = ?u", [-5]);  // Negative unsigned
} catch (ParameterException $e) {
    echo "Invalid parameter: " . $e->getMessage();
}
```

**Common causes:**
- Type mismatch with type-safe placeholders
- Missing required parameters
- Wrong number of parameters
- Invalid parameter format

## ConfigurationException

Thrown for configuration errors.

```php
use Sqlx\Exceptions\ConfigurationException;

try {
    $driver = DriverFactory::make([
        // Missing OPT_URL
        DriverOptions::OPT_MAX_CONNECTIONS => 10,
    ]);
} catch (ConfigurationException $e) {
    echo "Config error: " . $e->getMessage();
}
```

**Common causes:**
- Missing required options
- Invalid option values
- Conflicting options

## ValidationException

Thrown when input validation fails.

```php
use Sqlx\Exceptions\ValidationException;

try {
    $driver->insert('users', [
        'email' => 'not-an-email',  // Invalid format
    ]);
} catch (ValidationException $e) {
    echo "Validation failed: " . $e->getMessage();
}
```

## NotPermittedException

Thrown when an operation is not allowed.

```php
use Sqlx\Exceptions\NotPermittedException;

$readOnlyDriver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://localhost/mydb",
    DriverOptions::OPT_READONLY => true,
]);

try {
    $readOnlyDriver->execute("INSERT INTO users ...", [...]);
} catch (NotPermittedException $e) {
    echo "Write not allowed on read-only connection";
}
```

## TimeoutException

Thrown when an operation times out.

```php
use Sqlx\Exceptions\TimeoutException;

try {
    $driver->queryAll("SELECT * FROM huge_table");  // Takes too long
} catch (TimeoutException $e) {
    echo "Query timed out: " . $e->getMessage();
}
```

## PoolExhaustedException

Thrown when no connection is available from the pool.

```php
use Sqlx\Exceptions\PoolExhaustedException;

try {
    $result = $driver->queryAll("SELECT ...");
} catch (PoolExhaustedException $e) {
    echo "All connections busy, try again later";
}
```

**Common causes:**
- All connections in use
- Acquire timeout exceeded
- Slow queries holding connections

## Catching Multiple Types

```php
try {
    $user = $driver->queryRow("SELECT * FROM users WHERE id = ?", [$id]);
} catch (ConnectionException $e) {
    // Handle connection issues
    log_error("Database unavailable: " . $e->getMessage());
    show_maintenance_page();
} catch (QueryException $e) {
    // Handle query issues
    log_error("Query failed: " . $e->getSql());
    show_error_page();
} catch (SqlxException $e) {
    // Catch-all for other php-sqlx errors
    log_error("Database error: " . $e->getMessage());
    show_error_page();
}
```

## Best Practices

### Log Full Details

```php
try {
    $driver->execute($sql, $params);
} catch (SqlxException $e) {
    $logger->error('Database error', [
        'message' => $e->getMessage(),
        'code' => $e->getCode(),
        'sql' => $e->getSql(),
        'transient' => $e->isTransient(),
        'trace' => $e->getTraceAsString(),
    ]);
    throw $e;
}
```

### Retry Transient Errors

```php
try {
    $result = $driver->queryAll($sql, $params);
} catch (SqlxException $e) {
    if ($e->isTransient()) {
        // Retry logic
        usleep(100000);
        $result = $driver->queryAll($sql, $params);
    } else {
        throw $e;
    }
}
```

### Convert to Domain Exceptions

```php
try {
    return $this->driver->queryRow(
        "SELECT * FROM users WHERE id = ?",
        [$id]
    );
} catch (QueryException $e) {
    throw new UserNotFoundException("User $id not found", 0, $e);
}
```
