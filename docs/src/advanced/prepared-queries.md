# Prepared Queries

Prepared queries (prepared statements) are pre-compiled queries that can be executed multiple times with different parameters efficiently.

## Creating Prepared Queries

```php
$stmt = $driver->prepare("SELECT * FROM users WHERE status = ?");

// Execute multiple times
$activeUsers = $stmt->queryAll(['active']);
$pendingUsers = $stmt->queryAll(['pending']);
$inactiveUsers = $stmt->queryAll(['inactive']);
```

## Benefits

1. **Performance**: Query is parsed and planned once, executed many times
2. **Security**: Parameters are always properly escaped
3. **Clarity**: Separates query structure from data

## Query Methods

Prepared queries support all the same methods as drivers:

```php
$stmt = $driver->prepare("SELECT * FROM users WHERE id = ?");

// Single row
$user = $stmt->queryRow([1]);

// Maybe row
$user = $stmt->queryMaybeRow([999]);

// All rows (with different query)
$stmt = $driver->prepare("SELECT * FROM users WHERE status = ?");
$users = $stmt->queryAll(['active']);

// Value
$stmt = $driver->prepare("SELECT COUNT(*) FROM users WHERE status = ?");
$count = $stmt->queryValue(['active']);

// Column
$stmt = $driver->prepare("SELECT email FROM users WHERE status = ?");
$emails = $stmt->queryColumn(['active']);
```

### Dictionary Methods

```php
$stmt = $driver->prepare("SELECT id, name, email FROM users WHERE status = ?");

$usersById = $stmt->queryDictionary(['active']);
$namesById = $stmt->queryColumnDictionary(['active']);
$usersByRole = $stmt->queryGroupedDictionary(['active']);
```

### Execute (for modifications)

```php
$stmt = $driver->prepare("UPDATE users SET last_login = NOW() WHERE id = ?");

$stmt->execute([1]);
$stmt->execute([2]);
$stmt->execute([3]);
```

## Result Formats

Like driver methods, prepared queries support format variants:

```php
$stmt = $driver->prepare("SELECT * FROM users WHERE id = ?");

// Object (default)
$user = $stmt->queryRow([1]);
echo $user->name;

// Associative array
$user = $stmt->queryRowAssoc([1]);
echo $user['name'];

// Force object
$user = $stmt->queryRowObj([1]);
```

## Parameterized Queries vs Prepared Queries

**Regular parameterized query:**
```php
// Parsed and planned each time
$driver->queryRow("SELECT * FROM users WHERE id = ?", [1]);
$driver->queryRow("SELECT * FROM users WHERE id = ?", [2]);
```

**Prepared query:**
```php
// Parsed once, executed multiple times
$stmt = $driver->prepare("SELECT * FROM users WHERE id = ?");
$stmt->queryRow([1]);  // Uses cached plan
$stmt->queryRow([2]);  // Uses cached plan
```

## When to Use Prepared Queries

### Good Use Cases

**Batch operations:**
```php
$stmt = $driver->prepare("INSERT INTO logs (user_id, action, timestamp) VALUES (?, ?, ?)");

foreach ($logEntries as $entry) {
    $stmt->execute([$entry['user_id'], $entry['action'], $entry['timestamp']]);
}
```

**Repeated queries in a loop:**
```php
$stmt = $driver->prepare("SELECT * FROM products WHERE category_id = ?");

foreach ($categoryIds as $id) {
    $products[$id] = $stmt->queryAll([$id]);
}
```

**High-frequency queries:**
```php
// In a long-running worker
$getUserStmt = $driver->prepare("SELECT * FROM users WHERE id = ?");
$getOrdersStmt = $driver->prepare("SELECT * FROM orders WHERE user_id = ?");

while ($job = $queue->pop()) {
    $user = $getUserStmt->queryRow([$job->userId]);
    $orders = $getOrdersStmt->queryAll([$job->userId]);
    // Process...
}
```

### Less Beneficial Cases

**One-time queries:**
```php
// No benefit - only executed once
$stmt = $driver->prepare("SELECT * FROM users WHERE id = ?");
$user = $stmt->queryRow([1]);

// Just use driver method directly
$user = $driver->queryRow("SELECT * FROM users WHERE id = ?", [1]);
```

**Dynamic queries:**
```php
// Prepared statements can't have dynamic structure
$columns = ['id', 'name', 'email'];
$stmt = $driver->prepare("SELECT " . implode(', ', $columns) . " FROM users");
// This defeats the purpose - query text changes each time
```

## Named Parameters

```php
$stmt = $driver->prepare(
    "SELECT * FROM users WHERE status = $status AND role = $role"
);

$admins = $stmt->queryAll(['status' => 'active', 'role' => 'admin']);
$users = $stmt->queryAll(['status' => 'active', 'role' => 'user']);
```

## Performance Considerations

### Statement Caching

php-sqlx caches prepared statements internally. The same query string returns the same cached statement:

```php
// These use the same cached statement
$stmt1 = $driver->prepare("SELECT * FROM users WHERE id = ?");
$stmt2 = $driver->prepare("SELECT * FROM users WHERE id = ?");
// $stmt1 and $stmt2 share the same underlying prepared statement
```

### Memory Usage

Prepared statements consume server memory. For many different queries, unprepared execution may be more efficient:

```php
// Many unique queries - prepared statements may not help
foreach ($queries as $query) {
    $driver->prepare($query)->execute();  // Each creates new statement
}

// Better - just execute directly
foreach ($queries as $query) {
    $driver->execute($query);
}
```

## Database-Specific Notes

### PostgreSQL

PostgreSQL caches execution plans after multiple executions. First few runs may be slower as plans are optimized.

### MySQL

MySQL prepared statements are connection-specific. In pooled environments, statements may be re-prepared on different connections.

### MSSQL

MSSQL uses `sp_executesql` for parameterized queries, which provides plan caching benefits similar to prepared statements.
