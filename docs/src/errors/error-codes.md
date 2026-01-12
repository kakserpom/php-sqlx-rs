# Error Codes

php-sqlx exceptions include error codes that help identify the type of error.

## Error Code Constants

Error codes are defined as constants on `SqlxException`:

```php
use Sqlx\Exceptions\SqlxException;

SqlxException::GENERAL          // 0 - General/unknown error
SqlxException::CONNECTION       // 1 - Connection failed
SqlxException::QUERY            // 2 - Query execution failed
SqlxException::TRANSACTION      // 3 - Transaction error
SqlxException::PARSE            // 4 - SQL parsing error
SqlxException::PARAMETER        // 5 - Parameter binding error
SqlxException::CONFIGURATION    // 6 - Configuration error
SqlxException::VALIDATION       // 7 - Validation error
SqlxException::NOT_PERMITTED    // 8 - Operation not allowed
SqlxException::TIMEOUT          // 9 - Operation timed out
SqlxException::POOL_EXHAUSTED   // 10 - Connection pool exhausted
```

## Using Error Codes

```php
use Sqlx\Exceptions\SqlxException;

try {
    $driver->queryRow($sql, $params);
} catch (SqlxException $e) {
    switch ($e->getCode()) {
        case SqlxException::CONNECTION:
            // Database unavailable
            break;
        case SqlxException::QUERY:
            // Query failed
            break;
        case SqlxException::TIMEOUT:
            // Operation too slow
            break;
        default:
            // Other error
    }
}
```

## Code Reference

### GENERAL (0)

Unspecified error. Check the message for details.

```php
// Rare - most errors have specific codes
```

### CONNECTION (1)

Connection to database failed.

```php
// Network error
// "Connection error: Failed to connect to server"

// Authentication failure
// "Connection error: password authentication failed for user 'myuser'"

// SSL error
// "Connection error: SSL connection required"
```

### QUERY (2)

Query execution failed.

```php
// Table doesn't exist
// "Query error: relation \"nonexistent\" does not exist"

// Column doesn't exist
// "Query error: column \"invalid_col\" does not exist"

// Constraint violation
// "Query error: duplicate key value violates unique constraint"

// Permission denied
// "Query error: permission denied for table users"
```

### TRANSACTION (3)

Transaction operation failed.

```php
// Deadlock
// "Transaction error: deadlock detected"

// Serialization failure
// "Transaction error: could not serialize access"

// Already in transaction
// "Transaction error: there is already a transaction in progress"
```

### PARSE (4)

SQL parsing failed (before sending to database).

```php
// Invalid conditional block
// "Parse error: unclosed conditional block"

// Invalid placeholder
// "Parse error: unknown placeholder type '?x'"
```

### PARAMETER (5)

Parameter binding failed.

```php
// Type mismatch
// "Parameter error: expected integer, got string 'abc'"

// Wrong count
// "Parameter error: expected 3 parameters, got 2"

// Missing named parameter
// "Parameter error: missing required parameter 'user_id'"
```

### CONFIGURATION (6)

Configuration error.

```php
// Missing URL
// "Configuration error: OPT_URL is required"

// Invalid option
// "Configuration error: invalid value for OPT_MAX_CONNECTIONS"
```

### VALIDATION (7)

Input validation failed.

```php
// Invalid identifier
// "Validation error: invalid table name"
```

### NOT_PERMITTED (8)

Operation not allowed.

```php
// Write on read-only
// "Not permitted: write operations disabled on read-only connection"
```

### TIMEOUT (9)

Operation timed out.

```php
// Query timeout
// "Timeout: query exceeded maximum execution time"

// Statement timeout
// "Timeout: canceling statement due to statement timeout"
```

### POOL_EXHAUSTED (10)

Connection pool is full.

```php
// All connections busy
// "Pool exhausted: timed out waiting for connection"
```

## Database-Specific Error Information

The original database error is often included in the message:

```php
try {
    $driver->execute("INSERT INTO users (email) VALUES (?)", ['duplicate@example.com']);
} catch (QueryException $e) {
    // PostgreSQL
    // "Query error: duplicate key value violates unique constraint \"users_email_key\""

    // MySQL
    // "Query error: Duplicate entry 'duplicate@example.com' for key 'users.email'"

    // MSSQL
    // "Query error: Violation of UNIQUE KEY constraint 'UQ_users_email'"
}
```

## Programmatic Error Handling

### By Exception Type

```php
use Sqlx\Exceptions\{
    ConnectionException,
    QueryException,
    TimeoutException,
    PoolExhaustedException
};

try {
    $result = $driver->queryAll($sql, $params);
} catch (ConnectionException $e) {
    // Retry with backoff or fail
} catch (TimeoutException $e) {
    // Cancel or increase timeout
} catch (PoolExhaustedException $e) {
    // Wait and retry
} catch (QueryException $e) {
    // Log and report error
}
```

### By Error Code

```php
try {
    $result = $driver->queryAll($sql, $params);
} catch (SqlxException $e) {
    if (in_array($e->getCode(), [
        SqlxException::CONNECTION,
        SqlxException::TIMEOUT,
        SqlxException::POOL_EXHAUSTED,
    ])) {
        // Infrastructure issue - retry or fail gracefully
        return $this->handleInfrastructureError($e);
    }

    if ($e->getCode() === SqlxException::QUERY) {
        // Application bug - log details
        return $this->handleQueryError($e);
    }

    throw $e;
}
```

### By Transient Flag

```php
try {
    $result = $driver->queryAll($sql, $params);
} catch (SqlxException $e) {
    if ($e->isTransient()) {
        // Temporary issue - can retry
        // (deadlock, connection lost, timeout)
    } else {
        // Permanent issue - don't retry
        // (syntax error, permission denied)
    }
}
```

## Logging Recommendations

Include relevant context in error logs:

```php
try {
    $result = $driver->execute($sql, $params);
} catch (SqlxException $e) {
    $this->logger->error('Database operation failed', [
        'error_code' => $e->getCode(),
        'error_name' => $this->getErrorName($e->getCode()),
        'message' => $e->getMessage(),
        'sql' => $e->getSql(),
        'is_transient' => $e->isTransient(),
        'context' => [
            'user_id' => $currentUserId,
            'request_id' => $requestId,
        ],
    ]);
}

private function getErrorName(int $code): string
{
    return match($code) {
        SqlxException::GENERAL => 'GENERAL',
        SqlxException::CONNECTION => 'CONNECTION',
        SqlxException::QUERY => 'QUERY',
        SqlxException::TRANSACTION => 'TRANSACTION',
        SqlxException::PARSE => 'PARSE',
        SqlxException::PARAMETER => 'PARAMETER',
        SqlxException::CONFIGURATION => 'CONFIGURATION',
        SqlxException::VALIDATION => 'VALIDATION',
        SqlxException::NOT_PERMITTED => 'NOT_PERMITTED',
        SqlxException::TIMEOUT => 'TIMEOUT',
        SqlxException::POOL_EXHAUSTED => 'POOL_EXHAUSTED',
        default => 'UNKNOWN',
    };
}
```
