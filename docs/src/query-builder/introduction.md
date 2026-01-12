# Query Builder Introduction

The Query Builder provides a fluent, type-safe API for constructing SQL queries programmatically. It's perfect for dynamic queries where conditions, ordering, and pagination come from user input.

## Creating a Builder

```php
// Get a builder from the driver
$builder = $driver->builder();

// Or get a read-optimized builder
$readBuilder = $driver->readBuilder();
```

## Basic Example

```php
$users = $driver->builder()
    ->select(['id', 'name', 'email'])
    ->from('users')
    ->where([['status', '=', 'active']])
    ->orderBy(['created_at' => 'DESC'])
    ->limit(10)
    ->queryAll();
```

## Fluent Interface

All builder methods return `$this`, allowing method chaining:

```php
$query = $driver->builder()
    ->select('*')
    ->from('orders')
    ->leftJoin('customers', 'customers.id = orders.customer_id')
    ->where([['orders.total', '>', 100]])
    ->orderBy(['orders.created_at' => 'DESC'])
    ->limit(50);

// Execute when ready
$orders = $query->queryAll();
```

## Builder Types

### Write Query Builder

For all query types including INSERT, UPDATE, DELETE:

```php
$builder = $driver->builder();

// SELECT
$users = $builder->select('*')->from('users')->queryAll();

// INSERT
$builder->insertInto('users')->set(['name' => 'Alice'])->execute();

// UPDATE
$builder->update('users')->set(['status' => 'inactive'])->where([['id', '=', 1]])->execute();

// DELETE
$builder->deleteFrom('users')->where([['id', '=', 1]])->execute();
```

### Read Query Builder

Optimized for SELECT queries, uses read replicas when available:

```php
$builder = $driver->readBuilder();

$users = $builder
    ->select('*')
    ->from('users')
    ->queryAll();  // May use a read replica
```

## Executing Queries

The builder supports all the same query methods as the driver:

```php
$builder = $driver->builder()->select('*')->from('users');

// Single row
$user = $builder->where([['id', '=', 1]])->queryRow();

// Maybe single row
$user = $builder->where([['id', '=', 999]])->queryMaybeRow();

// All rows
$users = $builder->queryAll();

// Single value
$count = $driver->builder()
    ->select('COUNT(*)')
    ->from('users')
    ->queryValue();

// As dictionary
$usersById = $driver->builder()
    ->select(['id', 'name', 'email'])
    ->from('users')
    ->queryDictionary();
```

## Debugging

### Dry Run

See the generated SQL without executing:

```php
[$sql, $params] = $driver->builder()
    ->select('*')
    ->from('users')
    ->where([['status', '=', 'active']])
    ->dry();

echo $sql;
// SELECT * FROM "users" WHERE "status" = $1

print_r($params);
// ['active']
```

### String Conversion

Convert to fully-rendered SQL with inline parameters:

```php
$query = $driver->builder()
    ->select('*')
    ->from('users')
    ->where([['status', '=', 'active']]);

echo (string) $query;
// SELECT * FROM "users" WHERE "status" = 'active'
```

> **Warning**: The string output is for debugging only. Always use parameterized execution.

## Database-Specific Builders

Each database has its own builder class that generates appropriate syntax:

| Driver | Builder Class |
|--------|---------------|
| PostgreSQL | `Sqlx\PgWriteQueryBuilder` |
| MySQL | `Sqlx\MySqlWriteQueryBuilder` |
| MSSQL | `Sqlx\MssqlWriteQueryBuilder` |

The driver automatically creates the correct builder type.

## Builder vs Raw SQL

**Use the Builder when:**
- Building dynamic queries from user input
- Whitelisting columns for ORDER BY, SELECT
- Creating complex joins programmatically
- Need database-agnostic queries

**Use Raw SQL when:**
- Query is static/simple
- Using database-specific features
- Performance is critical (builder has slight overhead)
- Query is complex and builder makes it harder to read

```php
// Good use of builder - dynamic filtering
$builder = $driver->builder()->select('*')->from('users');

if ($request->has('status')) {
    $builder->where([['status', '=', $request->get('status')]]);
}
if ($request->has('role')) {
    $builder->where([['role', '=', $request->get('role')]]);
}

// Good use of raw SQL - static query
$stats = $driver->queryRow("
    SELECT
        COUNT(*) as total,
        SUM(CASE WHEN active THEN 1 ELSE 0 END) as active_count,
        AVG(age) as avg_age
    FROM users
    WHERE created_at > NOW() - INTERVAL '30 days'
");
```

## Next Steps

- [SELECT Queries](./select.md) - Learn about SELECT, FROM, WHERE
- [INSERT, UPDATE, DELETE](./mutations.md) - Data modification queries
- [Joins](./joins.md) - All join types
- [Clause Helpers](./clause-helpers.md) - Safe column whitelisting
