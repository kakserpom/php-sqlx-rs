# Type-Safe Placeholders

Type-safe placeholders validate that parameters match expected types before executing queries, catching errors early and preventing data corruption.

## Why Type-Safe Placeholders?

Standard placeholders accept any value:

```php
// This silently inserts "abc" as age, potentially causing issues
$driver->execute("UPDATE users SET age = ? WHERE id = ?", ["abc", 1]);
```

Type-safe placeholders catch this:

```php
// Throws ParameterException: expected integer, got string
$driver->execute("UPDATE users SET age = ?i WHERE id = ?", ["abc", 1]);
```

## Placeholder Types

### Integer (`?i`)

Accepts only integers:

```php
$driver->queryAll("SELECT * FROM users WHERE id = ?i", [42]);      // OK
$driver->queryAll("SELECT * FROM users WHERE id = ?i", ["42"]);    // OK (string "42" converts)
$driver->queryAll("SELECT * FROM users WHERE id = ?i", ["abc"]);   // Throws!
$driver->queryAll("SELECT * FROM users WHERE id = ?i", [3.14]);    // Throws!
```

### Unsigned Integer (`?u`)

Accepts only non-negative integers:

```php
$driver->queryAll("SELECT * FROM users WHERE age >= ?u", [18]);    // OK
$driver->queryAll("SELECT * FROM users WHERE age >= ?u", [-1]);    // Throws!
$driver->queryAll("SELECT * FROM users WHERE age >= ?u", [0]);     // OK
```

### Decimal (`?d`)

Accepts integers, floats, or numeric strings:

```php
$driver->queryAll("SELECT * FROM products WHERE price = ?d", [19.99]);   // OK
$driver->queryAll("SELECT * FROM products WHERE price = ?d", ["19.99"]); // OK
$driver->queryAll("SELECT * FROM products WHERE price = ?d", [20]);      // OK
$driver->queryAll("SELECT * FROM products WHERE price = ?d", ["abc"]);   // Throws!
```

### Unsigned Decimal (`?ud`)

Accepts non-negative decimals:

```php
$driver->queryAll("SELECT * FROM products WHERE price >= ?ud", [0.00]);   // OK
$driver->queryAll("SELECT * FROM products WHERE price >= ?ud", [-0.01]); // Throws!
```

### String (`?s`)

Accepts only strings:

```php
$driver->queryAll("SELECT * FROM users WHERE name = ?s", ["Alice"]);  // OK
$driver->queryAll("SELECT * FROM users WHERE name = ?s", [123]);      // Throws!
```

### JSON (`?j`)

Accepts arrays or objects, serializes to JSON:

```php
$driver->execute("INSERT INTO events (data) VALUES (?j)", [
    ['event' => 'click', 'x' => 100, 'y' => 200]
]);  // OK - serialized to JSON
```

## Array Placeholders

Each scalar type has an array variant with `a` suffix:

| Scalar | Array | Description |
|--------|-------|-------------|
| `?i` | `?ia` | Integer array |
| `?u` | `?ua` | Unsigned integer array |
| `?d` | `?da` | Decimal array |
| `?ud` | `?uda` | Unsigned decimal array |
| `?s` | `?sa` | String array |
| `?j` | `?ja` | JSON array |

```php
// Integer array
$driver->queryAll(
    "SELECT * FROM users WHERE id IN (?ia)",
    [[1, 2, 3]]
);

// String array
$driver->queryAll(
    "SELECT * FROM users WHERE status IN (?sa)",
    [['active', 'pending']]
);

// All elements must match the type
$driver->queryAll(
    "SELECT * FROM users WHERE id IN (?ia)",
    [[1, "two", 3]]  // Throws! "two" is not an integer
);
```

## Nullable Placeholders

Nullable variants accept `null` in addition to the base type. Prefix with `n`:

| Type | Nullable | Description |
|------|----------|-------------|
| `?i` | `?ni` | Nullable integer |
| `?u` | `?nu` | Nullable unsigned integer |
| `?d` | `?nd` | Nullable decimal |
| `?ud` | `?nud` | Nullable unsigned decimal |
| `?s` | `?ns` | Nullable string |

```php
// Allow NULL
$driver->execute(
    "UPDATE users SET manager_id = ?ni WHERE id = ?i",
    [null, 1]
);

// In conditional blocks, nullable allows the block to be included with null
$driver->queryAll("
    SELECT * FROM users
    {{ WHERE manager_id = ?ni }}
", [null]);
// Executed: SELECT * FROM users WHERE manager_id = NULL
```

## Named Type-Safe Placeholders

Type annotations work with named parameters too:

```php
$driver->queryAll(
    "SELECT * FROM users WHERE age >= $min_age:u AND status = $status:s",
    ['min_age' => 18, 'status' => 'active']
);
```

Syntax: `$name:type` or `:name:type`

## Type Validation Errors

When validation fails, a `ParameterException` is thrown with details:

```php
use Sqlx\Exceptions\ParameterException;

try {
    $driver->queryAll("SELECT * FROM users WHERE age = ?u", [-5]);
} catch (ParameterException $e) {
    echo $e->getMessage();
    // "Parameter 1: expected unsigned integer, got -5"
}
```

## Array Element Validation

Array placeholders validate each element:

```php
try {
    $driver->queryAll(
        "SELECT * FROM users WHERE id IN (?ia)",
        [[1, 2, "three", 4]]
    );
} catch (ParameterException $e) {
    echo $e->getMessage();
    // "Parameter 1[2]: expected integer, got string 'three'"
}
```

## Nullable Arrays

Array elements can be nullable:

```php
// Array of nullable integers
$driver->queryAll(
    "SELECT * FROM data WHERE value IN (?nia)",  // ?nia = nullable int array
    [[1, null, 3]]  // OK - null allowed in array
);
```

## Best Practices

### Use Type-Safe Placeholders for User Input

```php
// User-provided age should be validated
$age = $_GET['age'];
$driver->queryAll(
    "SELECT * FROM users WHERE age >= ?u",
    [$age]  // Throws if not a valid unsigned integer
);
```

### Match Database Column Types

```php
// If 'price' is DECIMAL(10,2), use ?d
$driver->execute(
    "INSERT INTO products (name, price) VALUES (?s, ?d)",
    [$name, $price]
);
```

### Document Expected Types

```php
/**
 * Find users by criteria
 *
 * @param int|null $minAge Minimum age (unsigned)
 * @param string|null $status Status filter
 */
function findUsers(?int $minAge, ?string $status): array
{
    return $this->driver->queryAll("
        SELECT * FROM users
        WHERE 1=1
        {{ AND age >= ?u }}
        {{ AND status = ?s }}
    ", array_filter([$minAge, $status], fn($v) => $v !== null));
}
```

## Comparison with Untyped Placeholders

| Feature | `?` | `?i`, `?s`, etc. |
|---------|-----|------------------|
| Type checking | No | Yes |
| Null handling | Allowed | Requires `?ni`, `?ns`, etc. |
| Performance | Slightly faster | Slightly slower (validation) |
| Safety | Basic | Enhanced |

Use type-safe placeholders when:
- Accepting user input
- Data integrity is critical
- Working with strict column types

Use standard `?` when:
- Performance is critical
- Values are already validated
- Working with dynamic/mixed types
