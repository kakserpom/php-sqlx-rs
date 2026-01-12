# Read Replicas

php-sqlx supports read replicas for distributing read queries across multiple database servers.

## Why Use Read Replicas?

1. **Scale reads**: Distribute SELECT queries across multiple servers
2. **Reduce primary load**: Keep the primary database free for writes
3. **Improve availability**: If one replica is down, others can serve reads

## Configuration

### Basic Setup

```php
use Sqlx\DriverFactory;
use Sqlx\DriverOptions;

$driver = DriverFactory::make([
    // Primary (for writes)
    DriverOptions::OPT_URL => "postgres://user:pass@primary.db.example.com/mydb",

    // Read replicas
    DriverOptions::OPT_READ_REPLICAS => [
        "postgres://user:pass@replica1.db.example.com/mydb",
        "postgres://user:pass@replica2.db.example.com/mydb",
    ],
]);
```

### Weighted Replicas

Distribute load unevenly (e.g., for different server capacities):

```php
DriverOptions::OPT_READ_REPLICAS => [
    ['url' => "postgres://replica1.db.example.com/mydb", 'weight' => 2],
    ['url' => "postgres://replica2.db.example.com/mydb", 'weight' => 1],
],
// replica1 gets ~67% of reads, replica2 gets ~33%
```

### Mixed Format

```php
DriverOptions::OPT_READ_REPLICAS => [
    "postgres://replica1.db.example.com/mydb",  // weight = 1 (default)
    ['url' => "postgres://replica2.db.example.com/mydb", 'weight' => 3],
],
```

## How Routing Works

### Automatic Routing

By default, reads go to replicas and writes go to the primary:

```php
// Routed to a replica
$users = $driver->queryAll("SELECT * FROM users");

// Routed to primary
$driver->execute("INSERT INTO users ...", [...]);
```

### Using the Read Builder

Explicitly use replicas:

```php
$builder = $driver->readBuilder();  // Uses read replicas

$users = $builder
    ->select('*')
    ->from('users')
    ->queryAll();  // Goes to a replica
```

### Checking Replica Availability

```php
if ($driver->hasReadReplicas()) {
    // Can use read replicas
}
```

## Replica Selection

php-sqlx selects replicas using weighted random selection:

```php
// With weights [2, 1, 1]
// Server 1: 50% chance
// Server 2: 25% chance
// Server 3: 25% chance
```

## Replication Lag

Replicas may be behind the primary. Be aware of this for:

### Read-After-Write

```php
// Write to primary
$driver->execute("INSERT INTO users (name) VALUES (?)", ['Alice']);

// Read from replica - might not see the new user yet!
$user = $driver->queryRow("SELECT * FROM users WHERE name = ?", ['Alice']);
```

### Solutions

**1. Read from primary after writes:**
```php
$driver->execute("INSERT INTO users ...", [...]);

// Use the write builder (goes to primary)
$user = $driver->builder()  // Not readBuilder()
    ->select('*')
    ->from('users')
    ->where([['name', '=', 'Alice']])
    ->queryRow();
```

**2. Use transactions:**
```php
$driver->begin(function($driver) {
    $driver->execute("INSERT INTO users ...", [...]);
    // Within transaction, all queries use the same connection (primary)
    $user = $driver->queryRow("SELECT * FROM users WHERE name = ?", ['Alice']);
    return true;
});
```

**3. Add a delay:**
```php
$driver->execute("INSERT INTO users ...", [...]);
usleep(100000);  // Wait 100ms for replication
$user = $driver->queryRow("SELECT * FROM users WHERE name = ?", ['Alice']);
```

## Failover

If a replica is unavailable:

1. Queries fail over to other replicas
2. If all replicas are down, reads fall back to the primary
3. The unavailable replica is temporarily removed from rotation

## Connection Pools

Each replica maintains its own connection pool:

```php
$driver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://primary/mydb",
    DriverOptions::OPT_READ_REPLICAS => ["postgres://replica1/mydb", "postgres://replica2/mydb"],
    DriverOptions::OPT_MAX_CONNECTIONS => 5,  // 5 connections each to primary, replica1, replica2
]);
```

## Read-Only Mode

Create a driver that only uses replicas (no writes allowed):

```php
$readOnlyDriver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://replica1/mydb",
    DriverOptions::OPT_READONLY => true,
]);

$readOnlyDriver->queryAll("SELECT * FROM users");  // OK
$readOnlyDriver->execute("INSERT ...", [...]);     // Throws NotPermittedException!
```

## Best Practices

### Separate Read and Write Paths

```php
class UserRepository
{
    public function __construct(
        private $driver
    ) {}

    // Reads can use replicas
    public function findAll(): array
    {
        return $this->driver->readBuilder()
            ->select('*')
            ->from('users')
            ->queryAll();
    }

    // Writes use primary
    public function create(array $data): void
    {
        $this->driver->insert('users', $data);
    }

    // Read-after-write uses primary
    public function createAndReturn(array $data): object
    {
        $this->driver->insert('users', $data);

        // Use write builder to read from primary
        return $this->driver->builder()
            ->select('*')
            ->from('users')
            ->where([['email', '=', $data['email']]])
            ->queryRow();
    }
}
```

### Monitor Replication Lag

```sql
-- PostgreSQL: Check replica lag
SELECT client_addr, state, sent_lsn, write_lsn, flush_lsn, replay_lsn
FROM pg_stat_replication;

-- MySQL: Check replica lag
SHOW SLAVE STATUS\G
```

### Use Appropriate Weights

Match weights to server capacity:

```php
// replica1: 8 CPU, 32GB RAM
// replica2: 4 CPU, 16GB RAM
DriverOptions::OPT_READ_REPLICAS => [
    ['url' => "postgres://replica1/mydb", 'weight' => 2],
    ['url' => "postgres://replica2/mydb", 'weight' => 1],
],
```
