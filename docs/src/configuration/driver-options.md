# Driver Options

php-sqlx drivers can be configured with various options when created.

## Using Options

```php
use Sqlx\DriverFactory;
use Sqlx\DriverOptions;

$driver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://localhost/mydb",
    DriverOptions::OPT_MAX_CONNECTIONS => 10,
    DriverOptions::OPT_ASSOC_ARRAYS => true,
]);
```

## Core Options

### OPT_URL

**Required**. The database connection URL.

```php
DriverOptions::OPT_URL => "postgres://user:pass@localhost:5432/mydb"
```

See [Connection Strings](../getting-started/connection-strings.md) for URL formats.

### OPT_ASSOC_ARRAYS

Return query results as associative arrays instead of objects.

```php
DriverOptions::OPT_ASSOC_ARRAYS => true  // Default: false
```

```php
// With OPT_ASSOC_ARRAYS => false (default)
$user = $driver->queryRow("SELECT * FROM users WHERE id = 1");
echo $user->name;  // Object property access

// With OPT_ASSOC_ARRAYS => true
$user = $driver->queryRow("SELECT * FROM users WHERE id = 1");
echo $user['name'];  // Array access
```

### OPT_READONLY

Mark the driver as read-only. Write operations will throw `NotPermittedException`.

```php
DriverOptions::OPT_READONLY => true  // Default: false
```

Useful for:
- Replica-only connections
- Preventing accidental writes
- Analytics/reporting connections

### OPT_PERSISTENT_NAME

Enable persistent connections with a named pool.

```php
DriverOptions::OPT_PERSISTENT_NAME => "myapp_primary"
```

Connections persist across PHP requests (in PHP-FPM). Different names create separate pools.

## Query Behavior

### OPT_COLLAPSIBLE_IN

When `true`, empty arrays in IN clauses become `FALSE` (or `TRUE` for NOT IN).

```php
DriverOptions::OPT_COLLAPSIBLE_IN => true  // Default: true
```

```php
// With OPT_COLLAPSIBLE_IN => true
$driver->queryAll("SELECT * FROM users WHERE id IN (?)", [[]]);
// Becomes: SELECT * FROM users WHERE FALSE

// With OPT_COLLAPSIBLE_IN => false
$driver->queryAll("SELECT * FROM users WHERE id IN (?)", [[]]);
// Throws ParameterException
```

## AST Cache Options

php-sqlx caches parsed query ASTs for performance.

### OPT_AST_CACHE_SHARD_COUNT

Number of cache shards (for concurrent access).

```php
DriverOptions::OPT_AST_CACHE_SHARD_COUNT => 8  // Default: 8
```

### OPT_AST_CACHE_SHARD_SIZE

Maximum entries per shard.

```php
DriverOptions::OPT_AST_CACHE_SHARD_SIZE => 256  // Default: 256
```

Total cache capacity = shard_count × shard_size (default: 2048 entries)

## All Options Reference

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `OPT_URL` | string | (required) | Database connection URL |
| `OPT_ASSOC_ARRAYS` | bool | `false` | Return arrays instead of objects |
| `OPT_READONLY` | bool | `false` | Disable write operations |
| `OPT_PERSISTENT_NAME` | string | `null` | Persistent pool name |
| `OPT_COLLAPSIBLE_IN` | bool | `true` | Collapse empty IN to FALSE |
| `OPT_AST_CACHE_SHARD_COUNT` | int | `8` | AST cache shards |
| `OPT_AST_CACHE_SHARD_SIZE` | int | `256` | Entries per cache shard |
| `OPT_MAX_CONNECTIONS` | int | `2` | Max pool connections |
| `OPT_MIN_CONNECTIONS` | int | `0` | Min idle connections |
| `OPT_MAX_LIFETIME` | string/int | `null` | Connection max age |
| `OPT_IDLE_TIMEOUT` | string/int | `null` | Idle connection timeout |
| `OPT_ACQUIRE_TIMEOUT` | string/int | `null` | Pool acquire timeout |
| `OPT_TEST_BEFORE_ACQUIRE` | bool | `false` | Ping before acquiring |
| `OPT_READ_REPLICAS` | array | `[]` | Read replica URLs |
| `OPT_RETRY_MAX_ATTEMPTS` | int | `0` | Max retry attempts |
| `OPT_RETRY_INITIAL_BACKOFF` | string/int | `"100ms"` | Initial retry delay |
| `OPT_RETRY_MAX_BACKOFF` | string/int | `"10s"` | Max retry delay |
| `OPT_RETRY_MULTIPLIER` | float | `2.0` | Backoff multiplier |

## Duration Formats

Options that accept durations support:

- String format: `"100ms"`, `"30s"`, `"5m"`, `"1h"`
- Integer: Seconds

```php
DriverOptions::OPT_IDLE_TIMEOUT => "5m"      // 5 minutes
DriverOptions::OPT_IDLE_TIMEOUT => "30s"     // 30 seconds
DriverOptions::OPT_IDLE_TIMEOUT => "100ms"   // 100 milliseconds
DriverOptions::OPT_IDLE_TIMEOUT => 300       // 300 seconds
```

## Environment-Based Configuration

```php
$driver = DriverFactory::make([
    DriverOptions::OPT_URL => getenv('DATABASE_URL'),
    DriverOptions::OPT_MAX_CONNECTIONS => (int) getenv('DB_POOL_SIZE') ?: 5,
    DriverOptions::OPT_ASSOC_ARRAYS => getenv('DB_ASSOC_ARRAYS') === 'true',
]);
```

## Profile-Based Configuration

```php
$profiles = [
    'development' => [
        DriverOptions::OPT_URL => "postgres://localhost/myapp_dev",
        DriverOptions::OPT_MAX_CONNECTIONS => 2,
    ],
    'production' => [
        DriverOptions::OPT_URL => getenv('DATABASE_URL'),
        DriverOptions::OPT_MAX_CONNECTIONS => 10,
        DriverOptions::OPT_READ_REPLICAS => [
            getenv('DATABASE_REPLICA_1'),
            getenv('DATABASE_REPLICA_2'),
        ],
        DriverOptions::OPT_TEST_BEFORE_ACQUIRE => true,
    ],
];

$env = getenv('APP_ENV') ?: 'development';
$driver = DriverFactory::make($profiles[$env]);
```
