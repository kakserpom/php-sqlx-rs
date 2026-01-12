# Retry Policy

php-sqlx can automatically retry failed queries for transient errors like connection drops or deadlocks.

## Enabling Retries

```php
use Sqlx\DriverFactory;
use Sqlx\DriverOptions;

$driver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://localhost/mydb",
    DriverOptions::OPT_RETRY_MAX_ATTEMPTS => 3,
]);
```

## Configuration Options

### OPT_RETRY_MAX_ATTEMPTS

Maximum number of retry attempts.

```php
DriverOptions::OPT_RETRY_MAX_ATTEMPTS => 3  // Default: 0 (disabled)
```

Total attempts = 1 (original) + max_attempts (retries)

### OPT_RETRY_INITIAL_BACKOFF

Delay before the first retry.

```php
DriverOptions::OPT_RETRY_INITIAL_BACKOFF => "100ms"  // Default: "100ms"
```

### OPT_RETRY_MAX_BACKOFF

Maximum delay between retries.

```php
DriverOptions::OPT_RETRY_MAX_BACKOFF => "10s"  // Default: "10s"
```

### OPT_RETRY_MULTIPLIER

Multiplier for exponential backoff.

```php
DriverOptions::OPT_RETRY_MULTIPLIER => 2.0  // Default: 2.0
```

## Exponential Backoff

Delays increase exponentially between retries:

```
Attempt 1: immediate
Retry 1: 100ms wait
Retry 2: 200ms wait (100ms × 2)
Retry 3: 400ms wait (200ms × 2)
...capped at max_backoff
```

With default settings (initial=100ms, multiplier=2.0, max=10s):
- Retry 1: 100ms
- Retry 2: 200ms
- Retry 3: 400ms
- Retry 4: 800ms
- Retry 5: 1.6s
- Retry 6: 3.2s
- Retry 7: 6.4s
- Retry 8+: 10s (capped)

## Transient Errors

Only transient errors trigger retries:

| Error Type | Retried | Examples |
|------------|---------|----------|
| Connection lost | Yes | Network timeout, server restart |
| Deadlock | Yes | Concurrent transaction conflict |
| Lock timeout | Yes | Table/row lock wait exceeded |
| Serialization failure | Yes | SERIALIZABLE transaction conflict |
| Syntax error | No | Invalid SQL |
| Constraint violation | No | Unique key violation |
| Permission denied | No | Insufficient privileges |

## Idempotency Warning

**Important**: Retries are only safe for idempotent queries!

**Safe to retry:**
```php
// SELECT queries
$user = $driver->queryRow("SELECT * FROM users WHERE id = ?", [1]);

// Idempotent updates
$driver->execute("UPDATE users SET name = ? WHERE id = ?", ['Alice', 1]);
```

**Dangerous to retry:**
```php
// Non-idempotent insert - could create duplicates
$driver->execute("INSERT INTO logs (message) VALUES (?)", ['event']);

// Counter increment - could increment multiple times
$driver->execute("UPDATE counters SET value = value + 1 WHERE id = ?", [1]);
```

## Safe Retry Patterns

### Use Transactions for Writes

```php
// Retries are disabled within transactions by default
$driver->begin(function($driver) {
    $driver->execute("INSERT INTO orders ...", [...]);
    $driver->execute("UPDATE inventory ...", [...]);
    return true;
});
```

### Use Upserts

```php
// Idempotent - safe to retry
$driver->upsert('users',
    ['email' => 'alice@example.com', 'name' => 'Alice'],
    ['email'],  // conflict key
    ['name']    // update on conflict
);
```

### Use INSERT ... ON CONFLICT DO NOTHING

```php
// Idempotent - safe to retry
$driver->execute(
    "INSERT INTO events (id, data) VALUES (?, ?) ON CONFLICT (id) DO NOTHING",
    [$eventId, $data]
);
```

## Configuration Examples

### Conservative (Web Application)

```php
$driver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://localhost/mydb",
    DriverOptions::OPT_RETRY_MAX_ATTEMPTS => 2,
    DriverOptions::OPT_RETRY_INITIAL_BACKOFF => "50ms",
    DriverOptions::OPT_RETRY_MAX_BACKOFF => "500ms",
]);
```

### Aggressive (Background Worker)

```php
$driver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://localhost/mydb",
    DriverOptions::OPT_RETRY_MAX_ATTEMPTS => 5,
    DriverOptions::OPT_RETRY_INITIAL_BACKOFF => "100ms",
    DriverOptions::OPT_RETRY_MAX_BACKOFF => "30s",
    DriverOptions::OPT_RETRY_MULTIPLIER => 2.0,
]);
```

### Disabled (Strict Mode)

```php
$driver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://localhost/mydb",
    DriverOptions::OPT_RETRY_MAX_ATTEMPTS => 0,  // No retries
]);
```

## Manual Retry Logic

For more control, implement your own retry logic:

```php
function withRetry(callable $operation, int $maxAttempts = 3): mixed
{
    $attempt = 0;
    $lastException = null;

    while ($attempt < $maxAttempts) {
        try {
            return $operation();
        } catch (SqlxException $e) {
            if (!$e->isTransient()) {
                throw $e;
            }
            $lastException = $e;
            $attempt++;
            if ($attempt < $maxAttempts) {
                usleep(100000 * pow(2, $attempt - 1));  // Exponential backoff
            }
        }
    }

    throw $lastException;
}

// Usage
$result = withRetry(function() use ($driver) {
    return $driver->queryRow("SELECT * FROM users WHERE id = ?", [1]);
});
```

## Monitoring Retries

Currently, retry attempts are not exposed to PHP. Consider using query hooks for visibility:

```php
$retryCount = 0;

$driver->onQuery(function($sql, $params, $duration) use (&$retryCount) {
    // Note: This counts total queries, not retries specifically
    // You'd need to track unique query executions for accurate retry counts
});
```
