# Quick Start

This guide will get you up and running with php-sqlx in minutes.

## Creating a Driver

The `DriverFactory` creates the appropriate driver based on your connection URL:

```php
<?php

use Sqlx\DriverFactory;

// PostgreSQL
$driver = DriverFactory::make("postgres://user:password@localhost:5432/mydb");

// MySQL
$driver = DriverFactory::make("mysql://user:password@localhost:3306/mydb");

// MSSQL
$driver = DriverFactory::make("mssql://user:password@localhost:1433/mydb");
```

## Basic Queries

### Fetching a Single Row

```php
// Returns an object (stdClass by default)
$user = $driver->queryRow("SELECT * FROM users WHERE id = ?", [1]);
echo $user->name;

// Returns null if no row found
$user = $driver->queryMaybeRow("SELECT * FROM users WHERE id = ?", [999]);
if ($user === null) {
    echo "User not found";
}
```

### Fetching Multiple Rows

```php
$users = $driver->queryAll("SELECT * FROM users WHERE active = ?", [true]);

foreach ($users as $user) {
    echo "{$user->name}: {$user->email}\n";
}
```

### Fetching a Single Value

```php
$count = $driver->queryValue("SELECT COUNT(*) FROM users");
echo "Total users: $count";

$name = $driver->queryValue(
    "SELECT name FROM users WHERE id = ?",
    [1],
    'name'  // column name or index
);
```

### Fetching a Column

```php
$emails = $driver->queryColumn("SELECT email FROM users WHERE active = ?", [true]);
// Returns: ['alice@example.com', 'bob@example.com', ...]
```

## Inserting Data

### Using execute()

```php
$affected = $driver->execute(
    "INSERT INTO users (name, email) VALUES (?, ?)",
    ['Alice', 'alice@example.com']
);
echo "Inserted $affected row(s)";
```

### Using insert()

```php
$driver->insert('users', [
    'name' => 'Bob',
    'email' => 'bob@example.com',
    'created_at' => date('Y-m-d H:i:s')
]);
```

### Bulk Insert

```php
$driver->insertMany('users', [
    ['name' => 'Alice', 'email' => 'alice@example.com'],
    ['name' => 'Bob', 'email' => 'bob@example.com'],
    ['name' => 'Carol', 'email' => 'carol@example.com'],
]);
```

## Updating Data

```php
$affected = $driver->execute(
    "UPDATE users SET name = ? WHERE id = ?",
    ['Alice Smith', 1]
);
```

## Deleting Data

```php
$affected = $driver->execute(
    "DELETE FROM users WHERE id = ?",
    [1]
);
```

## Using Named Parameters

```php
$user = $driver->queryRow(
    "SELECT * FROM users WHERE name = $name AND status = $status",
    ['name' => 'Alice', 'status' => 'active']
);
```

## Dictionary Queries

Get results as key-value maps:

```php
// Map id => entire row
$usersById = $driver->queryDictionary("SELECT id, * FROM users");
// Result: [1 => {id: 1, name: 'Alice', ...}, 2 => {...}]

// Map id => name only
$namesById = $driver->queryColumnDictionary("SELECT id, name FROM users");
// Result: [1 => 'Alice', 2 => 'Bob', ...]

// Group by a column
$usersByRole = $driver->queryGroupedDictionary("SELECT role, * FROM users");
// Result: ['admin' => [{...}, {...}], 'user' => [{...}]]
```

## Error Handling

```php
use Sqlx\Exceptions\QueryException;
use Sqlx\Exceptions\ConnectionException;

try {
    $result = $driver->queryRow("SELECT * FROM nonexistent");
} catch (QueryException $e) {
    echo "Query failed: " . $e->getMessage();
} catch (ConnectionException $e) {
    echo "Connection failed: " . $e->getMessage();
}
```

## Result Formats

By default, rows are returned as objects. You can get arrays instead:

```php
// Per-query
$user = $driver->queryRowAssoc("SELECT * FROM users WHERE id = ?", [1]);
// Returns: ['id' => 1, 'name' => 'Alice', ...]

// Or configure globally
$driver = DriverFactory::make([
    Sqlx\DriverOptions::OPT_URL => "postgres://localhost/mydb",
    Sqlx\DriverOptions::OPT_ASSOC_ARRAYS => true
]);
```

## What's Next?

- [Connection Strings](./connection-strings.md) - Learn about connection URL formats
- [Query Methods](../core/query-methods.md) - Explore all query methods
- [Augmented SQL](../augmented-sql/overview.md) - Learn about conditional blocks and type-safe placeholders
