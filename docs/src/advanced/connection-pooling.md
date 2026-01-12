# Connection Pooling

php-sqlx includes a built-in connection pool that manages database connections efficiently.

## How Pooling Works

Instead of opening a new connection for each query:

1. Connections are created and kept in a pool
2. Queries acquire a connection from the pool
3. After the query, the connection returns to the pool
4. Connections are reused for subsequent queries

This reduces connection overhead and improves performance.

## Default Configuration

By default, php-sqlx creates a minimal pool:

- **Max connections**: 2
- **Min connections**: 0
- **Idle timeout**: None
- **Max lifetime**: None

## Configuring the Pool

```php
use Sqlx\DriverFactory;
use Sqlx\DriverOptions;

$driver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://localhost/mydb",

    // Pool size
    DriverOptions::OPT_MAX_CONNECTIONS => 10,  // Maximum connections
    DriverOptions::OPT_MIN_CONNECTIONS => 2,   // Minimum idle connections

    // Timeouts
    DriverOptions::OPT_IDLE_TIMEOUT => "5m",      // Close idle connections after 5 minutes
    DriverOptions::OPT_MAX_LIFETIME => "30m",     // Replace connections after 30 minutes
    DriverOptions::OPT_ACQUIRE_TIMEOUT => "30s",  // Wait max 30s to acquire connection

    // Health checks
    DriverOptions::OPT_TEST_BEFORE_ACQUIRE => true,  // Validate connections before use
]);
```

## Pool Size Guidelines

### Web Applications (PHP-FPM)

Each PHP-FPM worker is independent, so pool sizes should be small:

```php
DriverOptions::OPT_MAX_CONNECTIONS => 2,
DriverOptions::OPT_MIN_CONNECTIONS => 1,
```

With 50 PHP-FPM workers: 50 × 2 = 100 max connections

### Long-Running Workers (Swoole, RoadRunner)

Workers persist and can share pools:

```php
DriverOptions::OPT_MAX_CONNECTIONS => 10,
DriverOptions::OPT_MIN_CONNECTIONS => 2,
```

### CLI Scripts

Usually need only one connection:

```php
DriverOptions::OPT_MAX_CONNECTIONS => 1,
DriverOptions::OPT_MIN_CONNECTIONS => 0,
```

## Timeouts

### Idle Timeout

Close connections that have been idle too long:

```php
DriverOptions::OPT_IDLE_TIMEOUT => "5m",  // 5 minutes
```

Accepts duration strings: `"30s"`, `"5m"`, `"1h"`, or seconds as integer.

### Max Lifetime

Replace connections after a maximum age (prevents stale connections):

```php
DriverOptions::OPT_MAX_LIFETIME => "30m",  // 30 minutes
```

### Acquire Timeout

Maximum time to wait for a connection from the pool:

```php
DriverOptions::OPT_ACQUIRE_TIMEOUT => "30s",
```

If the timeout is exceeded, a `PoolExhaustedException` is thrown.

## Health Checks

### Test Before Acquire

Validate connections before returning from the pool:

```php
DriverOptions::OPT_TEST_BEFORE_ACQUIRE => true,
```

This adds a small ping query before each use, but ensures connections are valid.

## Persistent Pools

For PHP-FPM, you can share pools across requests using a persistent name:

```php
DriverOptions::OPT_PERSISTENT_NAME => "myapp_db",
```

Connections persist across requests, reducing connection overhead.

## Connection States

Connections cycle through states:

1. **Idle**: In pool, available for use
2. **In Use**: Acquired by a query
3. **Closed**: Removed from pool (error, timeout, max lifetime)

## Pool Exhaustion

When all connections are in use and the pool is at max size:

```php
use Sqlx\Exceptions\PoolExhaustedException;

try {
    $result = $driver->queryAll("SELECT * FROM large_table");
} catch (PoolExhaustedException $e) {
    // All connections busy, acquire timeout exceeded
    echo "Database pool exhausted";
}
```

### Preventing Exhaustion

1. **Increase pool size** (if database can handle more connections)
2. **Increase acquire timeout** (if queries are legitimately slow)
3. **Optimize slow queries** (reduce connection hold time)
4. **Use read replicas** (distribute load)

## Monitoring Pool Status

Currently, pool statistics are not exposed to PHP. Monitor at the database level:

```sql
-- PostgreSQL: Active connections
SELECT count(*) FROM pg_stat_activity WHERE datname = 'mydb';

-- MySQL: Active connections
SHOW STATUS LIKE 'Threads_connected';

-- MSSQL: Active connections
SELECT COUNT(*) FROM sys.dm_exec_connections;
```

## Best Practices

### Right-Size Your Pool

Too small: Connections wait, requests slow down
Too large: Wastes database resources, may hit database limits

```php
// Start conservative
DriverOptions::OPT_MAX_CONNECTIONS => 5,

// Monitor and adjust based on:
// - Average query duration
// - Concurrent request count
// - Database connection limits
```

### Use Connection Lifetimes

Prevent stale connections and connection leaks:

```php
DriverOptions::OPT_MAX_LIFETIME => "30m",
DriverOptions::OPT_IDLE_TIMEOUT => "5m",
```

### Enable Health Checks for Critical Apps

```php
DriverOptions::OPT_TEST_BEFORE_ACQUIRE => true,
```

Small overhead, but catches dead connections before they cause query failures.

### Don't Hold Connections

Release connections as soon as possible:

```php
// BAD - Holds connection during sleep
$result = $driver->queryAll("SELECT * FROM data");
sleep(10);  // Connection still held
processData($result);

// GOOD - Connection released before sleep
$result = $driver->queryAll("SELECT * FROM data");
// Connection returned to pool
sleep(10);
processData($result);
```
