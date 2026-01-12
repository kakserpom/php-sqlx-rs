# Migrating from PDO

This guide helps you migrate from PDO to php-sqlx.

## Quick Comparison

| PDO | php-sqlx |
|-----|----------|
| `new PDO($dsn, $user, $pass)` | `DriverFactory::make($url)` |
| `$pdo->query($sql)` | `$driver->queryAll($sql)` |
| `$pdo->prepare($sql)` | `$driver->prepare($sql)` |
| `$stmt->execute($params)` | `$driver->execute($sql, $params)` |
| `$stmt->fetch()` | `$driver->queryRow($sql, $params)` |
| `$stmt->fetchAll()` | `$driver->queryAll($sql, $params)` |

## Connection

### PDO

```php
$pdo = new PDO(
    "pgsql:host=localhost;dbname=mydb",
    "user",
    "password",
    [PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION]
);
```

### php-sqlx

```php
$driver = Sqlx\DriverFactory::make("postgres://user:password@localhost/mydb");
```

## Basic Queries

### PDO

```php
// Fetch single row
$stmt = $pdo->prepare("SELECT * FROM users WHERE id = ?");
$stmt->execute([$id]);
$user = $stmt->fetch(PDO::FETCH_OBJ);

// Fetch all rows
$stmt = $pdo->prepare("SELECT * FROM users WHERE status = ?");
$stmt->execute(['active']);
$users = $stmt->fetchAll(PDO::FETCH_OBJ);
```

### php-sqlx

```php
// Fetch single row
$user = $driver->queryRow("SELECT * FROM users WHERE id = ?", [$id]);

// Fetch all rows
$users = $driver->queryAll("SELECT * FROM users WHERE status = ?", ['active']);
```

## Named Parameters

### PDO

```php
$stmt = $pdo->prepare("SELECT * FROM users WHERE name = :name AND status = :status");
$stmt->execute(['name' => 'Alice', 'status' => 'active']);
$user = $stmt->fetch(PDO::FETCH_OBJ);
```

### php-sqlx

```php
$user = $driver->queryRow(
    "SELECT * FROM users WHERE name = $name AND status = $status",
    ['name' => 'Alice', 'status' => 'active']
);
```

Note: php-sqlx uses `$name` instead of `:name`, but `:name` also works.

## Insert/Update/Delete

### PDO

```php
$stmt = $pdo->prepare("INSERT INTO users (name, email) VALUES (?, ?)");
$stmt->execute(['Alice', 'alice@example.com']);
$affectedRows = $stmt->rowCount();
```

### php-sqlx

```php
$affectedRows = $driver->execute(
    "INSERT INTO users (name, email) VALUES (?, ?)",
    ['Alice', 'alice@example.com']
);

// Or use insert() helper
$driver->insert('users', [
    'name' => 'Alice',
    'email' => 'alice@example.com'
]);
```

## Transactions

### PDO

```php
$pdo->beginTransaction();
try {
    $pdo->exec("INSERT INTO orders ...");
    $pdo->exec("UPDATE inventory ...");
    $pdo->commit();
} catch (Exception $e) {
    $pdo->rollBack();
    throw $e;
}
```

### php-sqlx

```php
$driver->begin(function($driver) {
    $driver->execute("INSERT INTO orders ...", [...]);
    $driver->execute("UPDATE inventory ...", [...]);
    return true;  // Commit
});
```

## Prepared Statements

### PDO

```php
$stmt = $pdo->prepare("SELECT * FROM users WHERE id = ?");

foreach ($ids as $id) {
    $stmt->execute([$id]);
    $users[] = $stmt->fetch(PDO::FETCH_OBJ);
}
```

### php-sqlx

```php
$stmt = $driver->prepare("SELECT * FROM users WHERE id = ?");

foreach ($ids as $id) {
    $users[] = $stmt->queryRow([$id]);
}
```

## Fetch Modes

### PDO

```php
// Object
$user = $stmt->fetch(PDO::FETCH_OBJ);

// Associative array
$user = $stmt->fetch(PDO::FETCH_ASSOC);

// Numeric array
$user = $stmt->fetch(PDO::FETCH_NUM);
```

### php-sqlx

```php
// Object (default)
$user = $driver->queryRow("SELECT * FROM users WHERE id = ?", [$id]);

// Associative array
$user = $driver->queryRowAssoc("SELECT * FROM users WHERE id = ?", [$id]);

// Or configure globally
$driver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://localhost/mydb",
    DriverOptions::OPT_ASSOC_ARRAYS => true,
]);
```

## Error Handling

### PDO

```php
try {
    $stmt = $pdo->prepare("SELECT * FROM nonexistent");
    $stmt->execute();
} catch (PDOException $e) {
    echo $e->getMessage();
}
```

### php-sqlx

```php
use Sqlx\Exceptions\QueryException;

try {
    $driver->queryAll("SELECT * FROM nonexistent");
} catch (QueryException $e) {
    echo $e->getMessage();
    echo $e->getSql();  // The failing SQL
}
```

## Feature Mapping

| PDO Feature | php-sqlx Equivalent |
|-------------|---------------------|
| `PDO::ATTR_ERRMODE` | Always exceptions (like `ERRMODE_EXCEPTION`) |
| `PDO::FETCH_OBJ` | Default, or `queryRowObj()` |
| `PDO::FETCH_ASSOC` | `queryRowAssoc()` or `OPT_ASSOC_ARRAYS` |
| `PDO::FETCH_COLUMN` | `queryColumn()` |
| `$stmt->rowCount()` | Return value of `execute()` |
| `$pdo->lastInsertId()` | Use `RETURNING` clause |

## Features Not in PDO

php-sqlx provides additional features:

### Conditional Blocks

```php
$users = $driver->queryAll("
    SELECT * FROM users
    WHERE 1=1
    {{ AND status = $status }}
    {{ AND role = $role }}
", ['status' => 'active']);  // role block is omitted
```

### Type-Safe Placeholders

```php
$driver->queryAll("SELECT * FROM users WHERE age >= ?u", [18]);  // Unsigned only
```

### Dictionary Queries

```php
$usersById = $driver->queryDictionary("SELECT id, * FROM users");
// [1 => {...}, 2 => {...}, ...]
```

### Query Builder

```php
$users = $driver->builder()
    ->select(['id', 'name', 'email'])
    ->from('users')
    ->where([['status', '=', 'active']])
    ->orderBy(['name' => 'ASC'])
    ->queryAll();
```

### Built-in Connection Pooling

```php
$driver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://localhost/mydb",
    DriverOptions::OPT_MAX_CONNECTIONS => 10,
    DriverOptions::OPT_READ_REPLICAS => ["postgres://replica/mydb"],
]);
```

## Migration Strategy

### Gradual Migration

1. Create a wrapper that provides both interfaces
2. Migrate code module by module
3. Remove PDO when migration is complete

```php
class DatabaseConnection
{
    private $driver;

    public function __construct(string $url)
    {
        $this->driver = DriverFactory::make($url);
    }

    // Legacy PDO-style methods
    public function query(string $sql): array
    {
        return $this->driver->queryAll($sql);
    }

    public function prepare(string $sql): PreparedStatement
    {
        return new PreparedStatement($this->driver->prepare($sql));
    }

    // New php-sqlx methods
    public function queryRow(string $sql, array $params = []): ?object
    {
        return $this->driver->queryMaybeRow($sql, $params);
    }

    // ... etc
}
```

### Full Migration

For new projects or full rewrites:

```php
// Before (PDO)
class UserRepository
{
    private PDO $pdo;

    public function find(int $id): ?array
    {
        $stmt = $this->pdo->prepare("SELECT * FROM users WHERE id = ?");
        $stmt->execute([$id]);
        return $stmt->fetch(PDO::FETCH_ASSOC) ?: null;
    }
}

// After (php-sqlx)
class UserRepository
{
    private \Sqlx\DriverInterface $driver;

    public function find(int $id): ?object
    {
        return $this->driver->queryMaybeRow(
            "SELECT * FROM users WHERE id = ?",
            [$id]
        );
    }
}
```
