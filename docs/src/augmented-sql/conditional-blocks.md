# Conditional Blocks

Conditional blocks allow parts of your SQL to be dynamically included or excluded based on whether parameters are provided.

## Basic Syntax

Wrap optional SQL fragments in `{{ }}`:

```php
$users = $driver->queryAll("
    SELECT * FROM users
    WHERE 1=1
    {{ AND status = $status }}
", ['status' => 'active']);
// Executed: SELECT * FROM users WHERE 1=1 AND status = $1
```

When the parameter is missing or `null`, the entire block is omitted:

```php
$users = $driver->queryAll("
    SELECT * FROM users
    WHERE 1=1
    {{ AND status = $status }}
", []);
// Executed: SELECT * FROM users WHERE 1=1
```

## Multiple Conditions

Chain multiple conditional blocks:

```php
$users = $driver->queryAll("
    SELECT * FROM users
    WHERE 1=1
    {{ AND status = $status }}
    {{ AND role = $role }}
    {{ AND department = $department }}
    {{ AND created_at >= $since }}
    {{ AND created_at <= $until }}
", [
    'status' => 'active',
    'department' => 'Engineering',
]);
// Executed: SELECT * FROM users WHERE 1=1 AND status = $1 AND department = $2
```

## The 1=1 Pattern

The `WHERE 1=1` pattern is commonly used with conditional blocks because it:

1. Makes all conditions optional (they all start with AND)
2. Avoids syntax errors when all conditions are omitted
3. Is optimized away by the database query planner

```php
// Without 1=1, you'd need complex logic:
"WHERE {{ status = $status }} {{ AND role = $role }}"  // Broken if status missing!

// With 1=1, it's always valid:
"WHERE 1=1 {{ AND status = $status }} {{ AND role = $role }}"  // Always works
```

## Nested Blocks

Blocks can be nested:

```php
$query = "
    SELECT * FROM orders
    WHERE 1=1
    {{ AND customer_id = $customer_id
       {{ AND status = $status }}
    }}
";
```

The outer block must be included for inner blocks to be evaluated.

## Block with Multiple Parameters

A block can reference multiple parameters:

```php
$users = $driver->queryAll("
    SELECT * FROM users
    WHERE 1=1
    {{ AND created_at BETWEEN $start_date AND $end_date }}
", [
    'start_date' => '2024-01-01',
    'end_date' => '2024-12-31',
]);
```

**Important**: All parameters in a block must be provided, or the entire block is omitted.

```php
// Only start_date provided - entire block is omitted
$users = $driver->queryAll("
    SELECT * FROM users
    WHERE 1=1
    {{ AND created_at BETWEEN $start_date AND $end_date }}
", [
    'start_date' => '2024-01-01',
    // 'end_date' missing
]);
// Executed: SELECT * FROM users WHERE 1=1
```

## Conditional Joins

Use blocks for optional joins:

```php
$query = "
    SELECT u.*, {{ p.bio, p.avatar }}
    FROM users u
    {{ LEFT JOIN profiles p ON p.user_id = u.id }}
    WHERE u.active = true
    {{ AND p.verified = $verified }}
";

// With profile data
$users = $driver->queryAll($query, ['verified' => true]);

// Without profile join
$users = $driver->queryAll($query, []);
```

## Conditional ORDER BY

```php
$users = $driver->queryAll("
    SELECT * FROM users
    WHERE active = true
    {{ ORDER BY $order_column $order_dir }}
", [
    'order_column' => 'created_at',
    'order_dir' => 'DESC',
]);
```

> **Warning**: Be careful with dynamic column names. Use the Query Builder's `orderBy()` with whitelisting for untrusted input.

## NULL vs Missing Parameters

There's a distinction between `null` values and missing parameters:

```php
// Parameter is missing - block omitted
$driver->queryAll("SELECT * FROM users {{ WHERE deleted_at = $deleted }}", []);
// Executed: SELECT * FROM users

// Parameter is null - block omitted (null is treated as "not provided")
$driver->queryAll("SELECT * FROM users {{ WHERE deleted_at = $deleted }}", ['deleted' => null]);
// Executed: SELECT * FROM users

// To explicitly check for NULL, use IS NULL:
$driver->queryAll("SELECT * FROM users WHERE deleted_at IS NULL", []);
```

## Nullable Placeholders

For parameters that should be included even when `null`, use nullable placeholders:

```php
// ?ni = nullable integer
$driver->queryAll("
    SELECT * FROM users
    {{ WHERE manager_id = ?ni }}
", [null]);
// Executed: SELECT * FROM users WHERE manager_id = NULL
```

See [Type-Safe Placeholders](./type-safe-placeholders.md) for all nullable types.

## Positional Parameters in Blocks

Blocks work with positional parameters too:

```php
$users = $driver->queryAll("
    SELECT * FROM users
    WHERE 1=1
    {{ AND status = ? }}
    {{ AND role = ? }}
", ['active']);  // Only first block included
// Executed: SELECT * FROM users WHERE 1=1 AND status = $1
```

## Best Practices

### Use Descriptive Parameter Names

```php
// Clear intent
"{{ AND created_at >= $created_after }}"
"{{ AND price <= $max_price }}"

// Avoid ambiguous names
"{{ AND created_at >= $date }}"  // Which date?
```

### Keep Blocks Focused

```php
// Good - each block is independent
"{{ AND status = $status }}"
"{{ AND role = $role }}"

// Avoid - complex logic in blocks
"{{ AND (status = $status OR (role = $role AND active = true)) }}"
```

### Validate Critical Filters

For queries where certain filters should always be applied:

```php
// Ensure tenant isolation is always applied
if (!isset($params['tenant_id'])) {
    throw new \InvalidArgumentException('tenant_id is required');
}

$driver->queryAll("
    SELECT * FROM data
    WHERE tenant_id = $tenant_id
    {{ AND status = $status }}
", $params);
```
