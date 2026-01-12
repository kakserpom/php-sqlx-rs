# SELECT Queries

The Query Builder provides a comprehensive API for building SELECT queries.

## select()

Specify columns to select:

```php
// Select all columns
$builder->select('*');

// Select specific columns
$builder->select(['id', 'name', 'email']);

// With aliases
$builder->select(['id', 'name AS username', 'email']);

// Raw expressions
$builder->select(['id', 'COUNT(*) AS total']);
```

## from()

Specify the table(s) to query:

```php
// Single table
$builder->from('users');

// With alias
$builder->from('users u');

// Multiple tables (cross join)
$builder->from(['users', 'orders']);
```

## where()

Add WHERE conditions:

### Array Syntax

```php
// Simple equality
$builder->where([['status', '=', 'active']]);

// Multiple conditions (AND)
$builder->where([
    ['status', '=', 'active'],
    ['role', '=', 'admin']
]);
// WHERE "status" = $1 AND "role" = $2
```

### Supported Operators

```php
$builder->where([['age', '>=', 18]]);        // Greater than or equal
$builder->where([['age', '<=', 65]]);        // Less than or equal
$builder->where([['age', '>', 21]]);         // Greater than
$builder->where([['age', '<', 30]]);         // Less than
$builder->where([['name', '!=', 'Admin']]);  // Not equal
$builder->where([['name', '<>', 'Admin']]);  // Not equal (alternate)
$builder->where([['name', 'LIKE', '%john%']]);  // LIKE
$builder->where([['name', 'ILIKE', '%john%']]); // Case-insensitive LIKE (PostgreSQL)
$builder->where([['id', 'IN', [1, 2, 3]]]);  // IN array
$builder->where([['id', 'NOT IN', [1, 2, 3]]]);  // NOT IN array
$builder->where([['deleted_at', 'IS NULL']]);   // IS NULL
$builder->where([['deleted_at', 'IS NOT NULL']]);  // IS NOT NULL
```

### OR Conditions

Use `OR_()` for OR logic:

```php
use function Sqlx\OR_;

$builder->where([
    ['status', '=', 'active'],
    OR_([
        ['role', '=', 'admin'],
        ['role', '=', 'moderator']
    ])
]);
// WHERE "status" = $1 AND ("role" = $2 OR "role" = $3)
```

### String Conditions

For complex conditions, use raw SQL:

```php
$builder->where('created_at > NOW() - INTERVAL \'30 days\'');

// With parameters
$builder->where('created_at > ?', ['2024-01-01']);
```

### Chaining where()

Multiple `where()` calls are ANDed together:

```php
$builder
    ->where([['status', '=', 'active']])
    ->where([['age', '>=', 18]]);
// WHERE "status" = $1 AND "age" >= $2
```

## orderBy()

Sort results:

```php
// Single column
$builder->orderBy(['created_at' => 'DESC']);

// Multiple columns
$builder->orderBy([
    'status' => 'ASC',
    'created_at' => 'DESC'
]);

// String syntax
$builder->orderBy('created_at DESC');

// Array of strings
$builder->orderBy(['created_at DESC', 'id ASC']);
```

## groupBy()

Group results:

```php
// Single column
$builder->groupBy('status');

// Multiple columns
$builder->groupBy(['status', 'role']);

$driver->builder()
    ->select(['status', 'COUNT(*) AS count'])
    ->from('users')
    ->groupBy('status')
    ->queryAll();
```

## having()

Filter grouped results:

```php
$driver->builder()
    ->select(['status', 'COUNT(*) AS count'])
    ->from('users')
    ->groupBy('status')
    ->having('COUNT(*) > ?', [10])
    ->queryAll();
```

## limit() and offset()

Pagination:

```php
// Limit only
$builder->limit(10);

// Limit with offset
$builder->limit(10, 20);  // 10 rows starting at row 20

// Separate offset
$builder->limit(10)->offset(20);
```

## Subqueries

Use builders as subqueries:

```php
$subquery = $driver->builder()
    ->select('user_id')
    ->from('orders')
    ->where([['total', '>', 1000]]);

$users = $driver->builder()
    ->select('*')
    ->from('users')
    ->where("id IN ({$subquery})")  // Embed subquery
    ->queryAll();
```

## Common Table Expressions (CTEs)

### with()

```php
$builder
    ->with('active_users', 'SELECT * FROM users WHERE active = true')
    ->select('*')
    ->from('active_users')
    ->where([['role', '=', 'admin']]);
```

With parameters:

```php
$builder
    ->with('recent_orders', 'SELECT * FROM orders WHERE created_at > ?', ['2024-01-01'])
    ->select('*')
    ->from('recent_orders');
```

### withRecursive()

```php
$builder
    ->withRecursive(
        'subordinates(id, name, manager_id)',
        "SELECT id, name, manager_id FROM employees WHERE id = ?
         UNION ALL
         SELECT e.id, e.name, e.manager_id FROM employees e
         JOIN subordinates s ON e.manager_id = s.id",
        [1]
    )
    ->select('*')
    ->from('subordinates');
```

## Row Locking

Lock selected rows for update:

### forUpdate()

Acquire exclusive locks on selected rows:

```php
$driver->begin(function($driver) {
    $user = $driver->builder()
        ->select('*')
        ->from('users')
        ->where([['id', '=', 1]])
        ->forUpdate()
        ->queryRow();

    // Update with exclusive lock held
    $driver->builder()
        ->update('users')
        ->set(['balance' => $user->balance - 100])
        ->where([['id', '=', 1]])
        ->execute();

    return true;
});
```

### forShare()

Acquire shared locks (allows concurrent reads, blocks writes):

```php
$user = $driver->builder()
    ->select('*')
    ->from('users')
    ->where([['id', '=', 1]])
    ->forShare()
    ->queryRow();
```

## UNION

Combine queries:

```php
$admins = $driver->builder()
    ->select(['id', 'name'])
    ->from('admins');

$users = $driver->builder()
    ->select(['id', 'name'])
    ->from('users')
    ->union($admins)
    ->queryAll();

// UNION ALL (keeps duplicates)
$users = $driver->builder()
    ->select(['id', 'name'])
    ->from('users')
    ->unionAll($admins)
    ->queryAll();
```

## Complete Example

```php
$users = $driver->builder()
    ->select(['u.id', 'u.name', 'u.email', 'COUNT(o.id) AS order_count'])
    ->from('users u')
    ->leftJoin('orders o', 'o.user_id = u.id')
    ->where([
        ['u.status', '=', 'active'],
        ['u.created_at', '>=', '2024-01-01']
    ])
    ->groupBy(['u.id', 'u.name', 'u.email'])
    ->having('COUNT(o.id) > ?', [5])
    ->orderBy(['order_count' => 'DESC'])
    ->limit(10)
    ->queryAll();
```
