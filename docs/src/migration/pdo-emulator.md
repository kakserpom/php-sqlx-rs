# PDO Emulator

php-sqlx includes a PDO emulation layer for easier migration from PDO-based code.

## Overview

The PDO emulator provides PDO-compatible classes that wrap php-sqlx drivers:

```php
use PdoEmulator\PDO;

// Works like regular PDO
$pdo = new PDO("postgres://user:pass@localhost/mydb");
$stmt = $pdo->prepare("SELECT * FROM users WHERE id = ?");
$stmt->execute([1]);
$user = $stmt->fetch(PDO::FETCH_OBJ);
```

## Installation

Include the emulator file:

```php
require_once 'path/to/pdo.php';

use PdoEmulator\PDO;
use PdoEmulator\PDOStatement;
```

## Creating a Connection

```php
use PdoEmulator\PDO;

// URL-style connection
$pdo = new PDO("postgres://user:pass@localhost/mydb");
$pdo = new PDO("mysql://user:pass@localhost/mydb");
$pdo = new PDO("mssql://user:pass@localhost/mydb");
```

## Supported Methods

### PDO Class

| Method | Support | Notes |
|--------|---------|-------|
| `prepare($sql)` | Full | Returns PDOStatement |
| `query($sql)` | Full | Direct query execution |
| `exec($sql)` | Full | Returns affected rows |
| `beginTransaction()` | Limited | No nested transactions |
| `commit()` | Limited | See notes |
| `rollBack()` | Limited | See notes |
| `inTransaction()` | Full | |
| `lastInsertId()` | Partial | PostgreSQL only with RETURNING |
| `setAttribute()` | Partial | Limited attributes |
| `getAttribute()` | Partial | Limited attributes |
| `errorInfo()` | Full | |
| `quote($string)` | Full | |

### PDOStatement Class

| Method | Support | Notes |
|--------|---------|-------|
| `execute($params)` | Full | |
| `fetch($mode)` | Full | |
| `fetchAll($mode)` | Full | |
| `fetchColumn($col)` | Full | |
| `rowCount()` | Full | |
| `columnCount()` | Full | |
| `bindParam()` | Full | |
| `bindValue()` | Full | |
| `closeCursor()` | Full | |

## Fetch Modes

```php
use PdoEmulator\PDO;

$stmt = $pdo->query("SELECT * FROM users");

// Object (default)
$row = $stmt->fetch(PDO::FETCH_OBJ);

// Associative array
$row = $stmt->fetch(PDO::FETCH_ASSOC);

// Both numeric and associative
$row = $stmt->fetch(PDO::FETCH_BOTH);

// Numeric array
$row = $stmt->fetch(PDO::FETCH_NUM);
```

## Error Handling

```php
use PdoEmulator\PDO;
use PdoEmulator\PDOException;

// Errors throw PDOException (like PDO::ERRMODE_EXCEPTION)
try {
    $pdo->query("SELECT * FROM nonexistent");
} catch (PDOException $e) {
    echo $e->getMessage();
    print_r($e->errorInfo);  // [SQLSTATE, code, message]
}
```

## Transaction Limitations

The emulator has limited transaction support:

```php
// Basic transactions work
$pdo->beginTransaction();
$pdo->exec("INSERT INTO ...");
$pdo->commit();

// But nested transactions are not supported
$pdo->beginTransaction();
$pdo->beginTransaction();  // Error!
```

For complex transaction needs, use the native php-sqlx API:

```php
$driver->begin(function($driver) {
    $driver->savepoint('sp1');
    // ...
    $driver->rollbackToSavepoint('sp1');
    return true;
});
```

## Iterating Results

PDOStatement is iterable:

```php
$stmt = $pdo->query("SELECT * FROM users");

foreach ($stmt as $row) {
    echo $row->name . "\n";
}
```

## Migration Strategy

### Step 1: Replace PDO Class

```php
// Before
use PDO;
$pdo = new PDO("pgsql:host=localhost;dbname=mydb", "user", "pass");

// After
use PdoEmulator\PDO;
$pdo = new PDO("postgres://user:pass@localhost/mydb");
```

### Step 2: Test Existing Code

Your existing PDO-based code should work with minimal changes:

```php
// This code works with both real PDO and the emulator
$stmt = $pdo->prepare("SELECT * FROM users WHERE id = ?");
$stmt->execute([$id]);
$user = $stmt->fetch(PDO::FETCH_OBJ);
```

### Step 3: Gradual Native Migration

Replace PDO patterns with native php-sqlx for better features:

```php
// PDO-style (works but verbose)
$stmt = $pdo->prepare("SELECT * FROM users WHERE status = ?");
$stmt->execute(['active']);
$users = $stmt->fetchAll(PDO::FETCH_OBJ);

// Native php-sqlx (simpler)
$users = $driver->queryAll("SELECT * FROM users WHERE status = ?", ['active']);
```

## Accessing Native Driver

Get the underlying php-sqlx driver:

```php
$pdo = new PDO("postgres://localhost/mydb");

// Access native driver for advanced features
$driver = $pdo->getDriver();

// Use native features
$users = $driver->queryDictionary("SELECT id, * FROM users");
$driver->begin(function($driver) {
    // Native transactions with savepoints
});
```

## Limitations

The emulator doesn't support:

1. **PDO DSN Format**: Use URL format instead
   ```php
   // Not supported: "pgsql:host=localhost;dbname=mydb"
   // Use: "postgres://user:pass@localhost/mydb"
   ```

2. **All PDO Attributes**: Only common attributes are supported
   ```php
   // Supported
   $pdo->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

   // Not supported
   $pdo->setAttribute(PDO::ATTR_PERSISTENT, true);
   ```

3. **Nested Transactions**: Use native API for savepoints

4. **Some Fetch Modes**:
   ```php
   // Not supported
   PDO::FETCH_CLASS
   PDO::FETCH_INTO
   PDO::FETCH_LAZY
   PDO::FETCH_NAMED
   ```

## When to Use the Emulator

**Good for:**
- Quick migration from PDO
- Legacy code that can't be rewritten
- Testing php-sqlx with existing code

**Better to use native API for:**
- New code
- Performance-critical applications
- Advanced features (conditional blocks, type-safe placeholders)
- Complex transactions with savepoints

## Example: Full Migration

```php
// config.php - Before
$pdo = new \PDO(
    "pgsql:host=localhost;dbname=myapp",
    "user",
    "password"
);

// config.php - After (Step 1: Use emulator)
use PdoEmulator\PDO;
$pdo = new PDO("postgres://user:password@localhost/myapp");

// config.php - After (Step 2: Native driver)
$driver = Sqlx\DriverFactory::make("postgres://user:password@localhost/myapp");

// Repository - Before (PDO)
class UserRepository {
    public function __construct(private \PDO $pdo) {}

    public function find(int $id): ?array {
        $stmt = $this->pdo->prepare("SELECT * FROM users WHERE id = ?");
        $stmt->execute([$id]);
        return $stmt->fetch(\PDO::FETCH_ASSOC) ?: null;
    }
}

// Repository - After (Native)
class UserRepository {
    public function __construct(private \Sqlx\DriverInterface $driver) {}

    public function find(int $id): ?object {
        return $this->driver->queryMaybeRow("SELECT * FROM users WHERE id = ?", [$id]);
    }
}
```
