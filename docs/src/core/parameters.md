# Parameters & Placeholders

php-sqlx supports multiple placeholder styles and automatic parameter binding for safe, injection-free queries.

## Placeholder Styles

All styles are interchangeable and can be mixed within a query (though mixing is not recommended for clarity).

### Positional (?)

The classic PDO style:

```php
$user = $driver->queryRow(
    "SELECT * FROM users WHERE id = ? AND status = ?",
    [1, 'active']
);
```

### Numbered ($1, $2)

PostgreSQL style:

```php
$user = $driver->queryRow(
    "SELECT * FROM users WHERE id = $1 AND status = $2",
    [1, 'active']
);
```

### Named ($name)

Dollar-sign named parameters:

```php
$user = $driver->queryRow(
    "SELECT * FROM users WHERE id = $id AND status = $status",
    ['id' => 1, 'status' => 'active']
);
```

### Named (:name)

Colon-style named parameters:

```php
$user = $driver->queryRow(
    "SELECT * FROM users WHERE id = :id AND status = :status",
    ['id' => 1, 'status' => 'active']
);
```

## Parameter Types

php-sqlx automatically converts PHP types to appropriate database types:

| PHP Type | Database Type |
|----------|---------------|
| `int` | INTEGER |
| `float` | FLOAT/DOUBLE |
| `string` | VARCHAR/TEXT |
| `bool` | BOOLEAN (or 1/0 for MSSQL) |
| `null` | NULL |
| `array` | Expanded for IN clauses, or JSON |

## Array Parameters

Arrays are automatically expanded for IN clauses:

```php
$users = $driver->queryAll(
    "SELECT * FROM users WHERE id IN (?)",
    [[1, 2, 3]]
);
// Becomes: SELECT * FROM users WHERE id IN ($1, $2, $3)
```

With named parameters:

```php
$users = $driver->queryAll(
    "SELECT * FROM users WHERE status IN ($statuses)",
    ['statuses' => ['active', 'pending']]
);
```

## JSON Parameters

Arrays/objects are automatically serialized to JSON when needed:

```php
$driver->execute(
    "INSERT INTO events (data) VALUES (?)",
    [['event' => 'login', 'ip' => '192.168.1.1']]
);
// The array is JSON-encoded
```

## NULL Handling

Pass `null` directly:

```php
$driver->execute(
    "UPDATE users SET deleted_at = ? WHERE id = ?",
    [null, 1]
);
```

## Parameter Reuse

Named parameters can be reused:

```php
$results = $driver->queryAll(
    "SELECT * FROM events WHERE date >= $date OR created_at >= $date",
    ['date' => '2024-01-01']
);
```

## Empty Arrays

By default, empty arrays in IN clauses are collapsed to FALSE:

```php
$users = $driver->queryAll(
    "SELECT * FROM users WHERE id IN (?)",
    [[]]  // empty array
);
// Becomes: SELECT * FROM users WHERE FALSE
// Returns empty result set
```

This behavior can be disabled via `OPT_COLLAPSIBLE_IN`.

## Type-Safe Placeholders

php-sqlx provides type-safe placeholder variants. See [Type-Safe Placeholders](../augmented-sql/type-safe-placeholders.md) for details.

```php
// Unsigned integer only
$driver->queryAll("SELECT * FROM users WHERE age >= ?u", [18]);

// String only
$driver->queryAll("SELECT * FROM users WHERE name = ?s", ['Alice']);
```

## Best Practices

### Always Use Parameters

Never interpolate values directly:

```php
// WRONG - SQL injection risk!
$driver->queryRow("SELECT * FROM users WHERE id = $id");

// RIGHT
$driver->queryRow("SELECT * FROM users WHERE id = ?", [$id]);
```

### Use Named Parameters for Clarity

For queries with many parameters:

```php
// Hard to read
$driver->execute(
    "INSERT INTO orders (user_id, product_id, quantity, price, status) VALUES (?, ?, ?, ?, ?)",
    [$userId, $productId, $quantity, $price, 'pending']
);

// Much clearer
$driver->execute(
    "INSERT INTO orders (user_id, product_id, quantity, price, status)
     VALUES ($user_id, $product_id, $quantity, $price, $status)",
    [
        'user_id' => $userId,
        'product_id' => $productId,
        'quantity' => $quantity,
        'price' => $price,
        'status' => 'pending',
    ]
);
```

### Use Appropriate Types

Let the driver handle type conversion:

```php
// The driver handles this correctly
$driver->queryRow("SELECT * FROM users WHERE id = ?", [42]);       // int
$driver->queryRow("SELECT * FROM users WHERE id = ?", ["42"]);     // string converted to int
$driver->queryRow("SELECT * FROM users WHERE active = ?", [true]); // bool
```
