# Clause Helpers

Clause helpers provide safe, whitelist-based handling of user input for SELECT, ORDER BY, GROUP BY, and pagination.

## Why Clause Helpers?

User input in ORDER BY clauses can be dangerous:

```php
// DANGEROUS - SQL injection possible!
$order = $_GET['sort'];  // Could be "name; DROP TABLE users;--"
$builder->orderBy($order);
```

Clause helpers validate against a whitelist:

```php
// SAFE - only whitelisted columns allowed
$byClause = new ByClause(['name', 'email', 'created_at']);
$builder->orderBy($byClause->input($_GET['sort']));
```

## SelectClause

Control which columns can be selected:

```php
use Sqlx\SelectClause;

// Define allowed columns
$select = new SelectClause(['id', 'name', 'email', 'status', 'created_at']);

// User requests specific columns
$userColumns = ['name', 'email', 'invalid_column'];

$users = $driver->builder()
    ->select($select->input($userColumns))  // Only 'name' and 'email' are used
    ->from('users')
    ->queryAll();
```

### Dynamic Column Selection

```php
$select = new SelectClause(['id', 'name', 'email', 'bio', 'avatar']);

// Basic fields only
$basic = $select->input(['id', 'name']);

// Full profile
$full = $select->input(['id', 'name', 'email', 'bio', 'avatar']);

$users = $driver->builder()
    ->select($includeProfile ? $full : $basic)
    ->from('users')
    ->queryAll();
```

## ByClause

For ORDER BY and GROUP BY with whitelisting:

```php
use Sqlx\ByClause;

// Define allowed columns
$orderBy = new ByClause(['name', 'email', 'created_at', 'status']);

// User input: ['name' => 'ASC', 'created_at' => 'DESC']
$userSort = $_GET['sort'] ?? ['created_at' => 'DESC'];

$users = $driver->builder()
    ->select('*')
    ->from('users')
    ->orderBy($orderBy->input($userSort))
    ->queryAll();
```

### With Default Order

```php
$orderBy = new ByClause(['name', 'email', 'created_at']);

// If user doesn't specify, use default
$sort = !empty($_GET['sort'])
    ? $orderBy->input($_GET['sort'])
    : $orderBy->input(['created_at' => 'DESC']);
```

### For GROUP BY

```php
$groupBy = new ByClause(['status', 'role', 'department']);

$stats = $driver->builder()
    ->select(['status', 'COUNT(*) AS count'])
    ->from('users')
    ->groupBy($groupBy->input($_GET['group']))
    ->queryAll();
```

## PaginateClause

Safe pagination with configurable limits:

```php
use Sqlx\PaginateClause;

$paginate = new PaginateClause();
$paginate
    ->perPage(20)      // Default items per page
    ->minPerPage(1)    // Minimum allowed
    ->maxPerPage(100); // Maximum allowed

// User requests page 3 with 50 items
$page = (int) ($_GET['page'] ?? 1);
$perPage = (int) ($_GET['per_page'] ?? 20);

$users = $driver->builder()
    ->select('*')
    ->from('users')
    ->paginate($paginate($page, $perPage))
    ->queryAll();
```

### Pagination Values

The clause calculates LIMIT and OFFSET:

```php
$paginate = new PaginateClause();
$paginate->perPage(10);

$result = $paginate(1, null);  // Page 1: LIMIT 10 OFFSET 0
$result = $paginate(2, null);  // Page 2: LIMIT 10 OFFSET 10
$result = $paginate(3, 25);    // Page 3 with 25/page: LIMIT 25 OFFSET 50
```

### Enforcing Limits

```php
$paginate = new PaginateClause();
$paginate->maxPerPage(50);

// User tries to request 1000 items
$result = $paginate(1, 1000);  // Actually uses LIMIT 50
```

## Combining Helpers

Use all helpers together:

```php
use Sqlx\SelectClause;
use Sqlx\ByClause;
use Sqlx\PaginateClause;

$select = new SelectClause(['id', 'name', 'email', 'status', 'created_at']);
$orderBy = new ByClause(['name', 'email', 'created_at', 'status']);
$paginate = (new PaginateClause())->perPage(20)->maxPerPage(100);

$columns = $_GET['fields'] ?? ['id', 'name', 'email'];
$sort = $_GET['sort'] ?? ['created_at' => 'DESC'];
$page = (int) ($_GET['page'] ?? 1);
$perPage = (int) ($_GET['per_page'] ?? 20);

$users = $driver->builder()
    ->select($select->input($columns))
    ->from('users')
    ->orderBy($orderBy->input($sort))
    ->paginate($paginate($page, $perPage))
    ->queryAll();
```

## API-Style Response

```php
function listUsers(array $params): array
{
    $select = new SelectClause(['id', 'name', 'email', 'status']);
    $orderBy = new ByClause(['name', 'email', 'created_at']);
    $paginate = (new PaginateClause())->perPage(20)->maxPerPage(100);

    $page = (int) ($params['page'] ?? 1);
    $perPage = (int) ($params['per_page'] ?? 20);

    // Get total count
    $total = $driver->queryValue("SELECT COUNT(*) FROM users");

    // Get paginated results
    $users = $driver->builder()
        ->select($select->input($params['fields'] ?? ['id', 'name']))
        ->from('users')
        ->orderBy($orderBy->input($params['sort'] ?? ['id' => 'ASC']))
        ->paginate($paginate($page, $perPage))
        ->queryAll();

    return [
        'data' => $users,
        'meta' => [
            'total' => $total,
            'page' => $page,
            'per_page' => $perPage,
            'total_pages' => ceil($total / $perPage),
        ]
    ];
}
```

## Invalid Input Handling

By default, invalid columns are silently ignored:

```php
$select = new SelectClause(['id', 'name']);
$result = $select->input(['id', 'password', 'secret']);
// Only 'id' is included, 'password' and 'secret' are ignored
```

If you need stricter validation, check before using:

```php
$allowed = ['id', 'name', 'email'];
$requested = $_GET['fields'];

$invalid = array_diff($requested, $allowed);
if (!empty($invalid)) {
    throw new \InvalidArgumentException(
        'Invalid fields: ' . implode(', ', $invalid)
    );
}
```
