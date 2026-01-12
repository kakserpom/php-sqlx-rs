# Drivers

php-sqlx provides database-specific driver classes, all sharing a common interface. The `DriverFactory` creates the appropriate driver based on your connection URL.

## Creating Drivers

### Using DriverFactory (Recommended)

```php
use Sqlx\DriverFactory;

// Auto-detects database type from URL scheme
$pg = DriverFactory::make("postgres://user:pass@localhost/mydb");
$mysql = DriverFactory::make("mysql://user:pass@localhost/mydb");
$mssql = DriverFactory::make("mssql://user:pass@localhost/mydb");
```

### Direct Instantiation

```php
use Sqlx\PgDriver;
use Sqlx\MySqlDriver;
use Sqlx\MssqlDriver;

$pg = new PgDriver("postgres://localhost/mydb");
$mysql = new MySqlDriver("mysql://localhost/mydb");
$mssql = new MssqlDriver("mssql://localhost/mydb");
```

### With Options Array

```php
use Sqlx\DriverFactory;
use Sqlx\DriverOptions;

$driver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://localhost/mydb",
    DriverOptions::OPT_MAX_CONNECTIONS => 10,
    DriverOptions::OPT_ASSOC_ARRAYS => true,
]);
```

## Driver Classes

| Class | Databases |
|-------|-----------|
| `Sqlx\PgDriver` | PostgreSQL |
| `Sqlx\MySqlDriver` | MySQL, MariaDB |
| `Sqlx\MssqlDriver` | Microsoft SQL Server |

All drivers implement the same interface, so you can write database-agnostic code.

## Driver Lifecycle

### Closing Connections

Drivers manage a connection pool internally. To release resources explicitly:

```php
$driver->close();
```

After closing, the driver cannot be used for further queries.

### Checking Status

```php
if ($driver->isClosed()) {
    echo "Driver is closed";
}
```

## Configuration Methods

### Application Name

Set an identifier visible in database monitoring tools:

```php
$driver->setApplicationName("my-api-server");
```

This shows up in:
- PostgreSQL: `pg_stat_activity.application_name`
- MySQL: Session variable `@sqlx_application_name`
- MSSQL: `SESSION_CONTEXT(N'application_name')`

### Client Info

Set metadata for debugging and monitoring:

```php
$driver->setClientInfo("my-app", [
    'request_id' => $requestId,
    'user_id' => $userId,
]);
```

## Result Format

By default, query results are returned as objects (`stdClass`). Check or change this:

```php
// Check current setting
if ($driver->assocArrays()) {
    echo "Results are arrays";
} else {
    echo "Results are objects";
}
```

To get arrays by default, configure when creating the driver:

```php
$driver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://localhost/mydb",
    DriverOptions::OPT_ASSOC_ARRAYS => true,
]);
```

Or use the `*Assoc` method variants:

```php
$row = $driver->queryRowAssoc("SELECT * FROM users WHERE id = ?", [1]);
// Returns: ['id' => 1, 'name' => 'Alice', ...]
```

## Read Replicas

Check if read replicas are configured:

```php
if ($driver->hasReadReplicas()) {
    echo "Read replicas are available";
}
```

See [Read Replicas](../advanced/read-replicas.md) for configuration details.

## Thread Safety

Drivers are thread-safe and can be shared across requests in persistent/worker modes (e.g., Swoole, RoadRunner). The internal connection pool handles concurrent access.

## Example: Repository Pattern

```php
class UserRepository
{
    public function __construct(
        private \Sqlx\PgDriver $driver
    ) {}

    public function find(int $id): ?object
    {
        return $this->driver->queryMaybeRow(
            "SELECT * FROM users WHERE id = ?",
            [$id]
        );
    }

    public function findByEmail(string $email): ?object
    {
        return $this->driver->queryMaybeRow(
            "SELECT * FROM users WHERE email = ?",
            [$email]
        );
    }

    public function create(array $data): void
    {
        $this->driver->insert('users', $data);
    }

    public function update(int $id, array $data): int
    {
        $sets = [];
        $params = [];
        foreach ($data as $key => $value) {
            $sets[] = "$key = ?";
            $params[] = $value;
        }
        $params[] = $id;

        return $this->driver->execute(
            "UPDATE users SET " . implode(', ', $sets) . " WHERE id = ?",
            $params
        );
    }
}
```
