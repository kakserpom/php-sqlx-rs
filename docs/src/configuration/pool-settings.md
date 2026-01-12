# Connection Pool Settings

Configure the connection pool for optimal performance and resource usage.

## Pool Size

### OPT_MAX_CONNECTIONS

Maximum number of connections in the pool.

```php
DriverOptions::OPT_MAX_CONNECTIONS => 10  // Default: 2
```

**Guidelines:**
- PHP-FPM: Keep low (1-3) since each worker has its own pool
- Long-running workers: Can be higher (5-20)
- CLI scripts: Usually 1 is sufficient

**Calculate total connections:**
```
Total = workers × max_connections
Example: 50 PHP-FPM workers × 2 = 100 connections
```

### OPT_MIN_CONNECTIONS

Minimum idle connections to maintain.

```php
DriverOptions::OPT_MIN_CONNECTIONS => 2  // Default: 0
```

Benefits:
- Faster first query (no connection time)
- Stable connection count

Trade-offs:
- Uses database resources even when idle
- May not be useful for short-lived processes

## Timeouts

### OPT_IDLE_TIMEOUT

Close connections that have been idle longer than this duration.

```php
DriverOptions::OPT_IDLE_TIMEOUT => "5m"  // Default: null (no timeout)
```

When to use:
- Cloud databases with connection limits
- Long-running workers with variable load
- Resource conservation

### OPT_MAX_LIFETIME

Close and replace connections after this age, regardless of activity.

```php
DriverOptions::OPT_MAX_LIFETIME => "30m"  // Default: null (no limit)
```

Prevents:
- Stale connections
- Memory leaks in long-running processes
- Issues with database failovers

### OPT_ACQUIRE_TIMEOUT

Maximum time to wait for a connection from the pool.

```php
DriverOptions::OPT_ACQUIRE_TIMEOUT => "30s"  // Default: null (wait indefinitely)
```

If exceeded, throws `PoolExhaustedException`.

## Health Checks

### OPT_TEST_BEFORE_ACQUIRE

Validate connections before returning them from the pool.

```php
DriverOptions::OPT_TEST_BEFORE_ACQUIRE => true  // Default: false
```

Adds a small ping query (e.g., `SELECT 1`) before each acquisition.

**Pros:**
- Catches dead connections before they cause query failures
- Handles network timeouts, database restarts

**Cons:**
- Small overhead per query
- May not be needed if idle_timeout/max_lifetime are set

## Configuration Examples

### Web Application (PHP-FPM)

```php
$driver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://localhost/myapp",
    DriverOptions::OPT_MAX_CONNECTIONS => 2,
    DriverOptions::OPT_MIN_CONNECTIONS => 1,
    DriverOptions::OPT_IDLE_TIMEOUT => "1m",
    DriverOptions::OPT_ACQUIRE_TIMEOUT => "5s",
]);
```

### Long-Running Worker

```php
$driver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://localhost/myapp",
    DriverOptions::OPT_MAX_CONNECTIONS => 10,
    DriverOptions::OPT_MIN_CONNECTIONS => 2,
    DriverOptions::OPT_IDLE_TIMEOUT => "5m",
    DriverOptions::OPT_MAX_LIFETIME => "30m",
    DriverOptions::OPT_TEST_BEFORE_ACQUIRE => true,
]);
```

### CLI Script

```php
$driver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://localhost/myapp",
    DriverOptions::OPT_MAX_CONNECTIONS => 1,
    DriverOptions::OPT_MIN_CONNECTIONS => 0,
]);
```

### Cloud Database (Limited Connections)

```php
$driver = DriverFactory::make([
    DriverOptions::OPT_URL => getenv('DATABASE_URL'),
    DriverOptions::OPT_MAX_CONNECTIONS => 5,
    DriverOptions::OPT_MIN_CONNECTIONS => 0,
    DriverOptions::OPT_IDLE_TIMEOUT => "2m",
    DriverOptions::OPT_MAX_LIFETIME => "15m",
    DriverOptions::OPT_ACQUIRE_TIMEOUT => "10s",
    DriverOptions::OPT_TEST_BEFORE_ACQUIRE => true,
]);
```

### High-Availability Setup

```php
$driver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://primary.db/myapp",
    DriverOptions::OPT_MAX_CONNECTIONS => 5,
    DriverOptions::OPT_MIN_CONNECTIONS => 2,
    DriverOptions::OPT_MAX_LIFETIME => "10m",
    DriverOptions::OPT_TEST_BEFORE_ACQUIRE => true,
    DriverOptions::OPT_READ_REPLICAS => [
        "postgres://replica1.db/myapp",
        "postgres://replica2.db/myapp",
    ],
]);
```

## Monitoring

While php-sqlx doesn't expose pool metrics directly, monitor at the database level:

### PostgreSQL

```sql
-- Current connections by application
SELECT application_name, count(*)
FROM pg_stat_activity
WHERE datname = 'myapp'
GROUP BY application_name;

-- Connection states
SELECT state, count(*)
FROM pg_stat_activity
WHERE datname = 'myapp'
GROUP BY state;
```

### MySQL

```sql
-- Current connections
SHOW STATUS LIKE 'Threads_connected';

-- Max connections reached
SHOW STATUS LIKE 'Connection_errors_max_connections';
```

## Troubleshooting

### Pool Exhaustion

**Symptoms:** `PoolExhaustedException` thrown

**Solutions:**
1. Increase `OPT_MAX_CONNECTIONS`
2. Increase `OPT_ACQUIRE_TIMEOUT`
3. Optimize slow queries
4. Add read replicas for read-heavy workloads

### Connection Churn

**Symptoms:** High connection creation rate in database logs

**Solutions:**
1. Increase `OPT_MIN_CONNECTIONS`
2. Increase `OPT_IDLE_TIMEOUT`
3. Use `OPT_PERSISTENT_NAME` (PHP-FPM)

### Dead Connections

**Symptoms:** Random query failures with connection errors

**Solutions:**
1. Enable `OPT_TEST_BEFORE_ACQUIRE`
2. Set `OPT_MAX_LIFETIME` shorter than database timeout
3. Set `OPT_IDLE_TIMEOUT` shorter than firewall timeout
