# IN Clause Handling

php-sqlx provides smart handling of SQL IN clauses, automatically expanding arrays and handling edge cases like empty sets.

## Basic Array Expansion

Pass an array parameter for automatic expansion:

```php
$users = $driver->queryAll(
    "SELECT * FROM users WHERE id IN (?)",
    [[1, 2, 3]]
);
// Executed: SELECT * FROM users WHERE id IN ($1, $2, $3)
// Parameters: [1, 2, 3]
```

With named parameters:

```php
$users = $driver->queryAll(
    "SELECT * FROM users WHERE id IN ($ids)",
    ['ids' => [1, 2, 3]]
);
```

## NOT IN Clauses

Works the same way:

```php
$users = $driver->queryAll(
    "SELECT * FROM users WHERE id NOT IN (?)",
    [[1, 2, 3]]
);
// Executed: SELECT * FROM users WHERE id NOT IN ($1, $2, $3)
```

## Empty Array Handling

By default, empty arrays are collapsed to `FALSE`:

```php
$users = $driver->queryAll(
    "SELECT * FROM users WHERE id IN (?)",
    [[]]  // empty array
);
// Executed: SELECT * FROM users WHERE FALSE
// Returns: []
```

For NOT IN, empty arrays become `TRUE`:

```php
$users = $driver->queryAll(
    "SELECT * FROM users WHERE id NOT IN (?)",
    [[]]  // empty array
);
// Executed: SELECT * FROM users WHERE TRUE
// Returns: all users
```

This behavior is intuitive:
- `IN ()` - "is the value in this empty set?" → always false
- `NOT IN ()` - "is the value not in this empty set?" → always true

### Disabling Collapsible IN

If you prefer SQL-standard behavior (which would error on empty IN):

```php
$driver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://localhost/mydb",
    DriverOptions::OPT_COLLAPSIBLE_IN => false,
]);

// Now throws an exception for empty arrays
$driver->queryAll("SELECT * FROM users WHERE id IN (?)", [[]]);
// Throws: ParameterException - empty array not allowed
```

## Type-Safe IN Clauses

Combine array expansion with type checking:

```php
// Integer array
$users = $driver->queryAll(
    "SELECT * FROM users WHERE id IN (?ia)",
    [[1, 2, 3]]
);

// String array
$users = $driver->queryAll(
    "SELECT * FROM users WHERE status IN (?sa)",
    [['active', 'pending']]
);

// Type mismatch throws
$users = $driver->queryAll(
    "SELECT * FROM users WHERE id IN (?ia)",
    [[1, "two", 3]]  // Throws! "two" is not an integer
);
```

## Multiple IN Clauses

Each array parameter is expanded independently:

```php
$users = $driver->queryAll(
    "SELECT * FROM users WHERE id IN (?) AND role IN (?)",
    [[1, 2, 3], ['admin', 'moderator']]
);
// Executed: SELECT * FROM users WHERE id IN ($1, $2, $3) AND role IN ($4, $5)
```

## IN with Conditional Blocks

Combine IN clauses with conditional blocks for optional filters:

```php
$users = $driver->queryAll("
    SELECT * FROM users
    WHERE 1=1
    {{ AND id IN ($ids) }}
    {{ AND status IN ($statuses) }}
", [
    'ids' => [1, 2, 3],
    // 'statuses' not provided - block omitted
]);
// Executed: SELECT * FROM users WHERE 1=1 AND id IN ($1, $2, $3)
```

## Subqueries vs Arrays

For large sets, consider using a subquery instead:

```php
// Array expansion - good for small sets (< 100 items)
$driver->queryAll(
    "SELECT * FROM orders WHERE customer_id IN (?)",
    [$customerIds]
);

// Subquery - better for large sets or dynamic conditions
$driver->queryAll(
    "SELECT * FROM orders WHERE customer_id IN (
        SELECT id FROM customers WHERE region = ?
    )",
    ['west']
);
```

## Performance Considerations

### Array Size Limits

Most databases have limits on the number of parameters:
- PostgreSQL: ~32,000 parameters
- MySQL: ~65,000 parameters
- MSSQL: ~2,100 parameters

For very large sets, use:

```php
// Batch processing
$chunks = array_chunk($largeIdArray, 1000);
$results = [];
foreach ($chunks as $chunk) {
    $results = array_merge(
        $results,
        $driver->queryAll("SELECT * FROM users WHERE id IN (?)", [$chunk])
    );
}

// Or use a temporary table / CTE
$driver->execute("CREATE TEMP TABLE temp_ids (id INT)");
$driver->insertMany('temp_ids', array_map(fn($id) => ['id' => $id], $largeIdArray));
$results = $driver->queryAll("SELECT u.* FROM users u JOIN temp_ids t ON u.id = t.id");
```

### Query Plan Caching

Queries with different array sizes generate different SQL, which may affect query plan caching:

```php
// These generate different SQL (different number of placeholders)
$driver->queryAll("SELECT * FROM users WHERE id IN (?)", [[1, 2]]);
// → SELECT * FROM users WHERE id IN ($1, $2)

$driver->queryAll("SELECT * FROM users WHERE id IN (?)", [[1, 2, 3]]);
// → SELECT * FROM users WHERE id IN ($1, $2, $3)
```

For frequently-called queries with variable array sizes, consider using `= ANY()` syntax (PostgreSQL) or temporary tables.

## Common Patterns

### Optional Filter with Default

```php
$statusFilter = $statuses ?? ['active'];  // default to active only

$users = $driver->queryAll(
    "SELECT * FROM users WHERE status IN (?)",
    [$statusFilter]
);
```

### Excluding IDs

```php
$excludeIds = [1, 2, 3];  // IDs to exclude

$users = $driver->queryAll(
    "SELECT * FROM users WHERE id NOT IN (?)",
    [$excludeIds ?: [0]]  // use [0] if empty to avoid matching nothing
);
```

### Combining Arrays

```php
$adminIds = [1, 2];
$moderatorIds = [3, 4, 5];

$users = $driver->queryAll(
    "SELECT * FROM users WHERE id IN (?)",
    [array_merge($adminIds, $moderatorIds)]
);
```
