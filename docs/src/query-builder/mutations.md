# INSERT, UPDATE, DELETE

The Query Builder supports data modification queries with the same fluent interface.

## INSERT

### Basic Insert

```php
$driver->builder()
    ->insertInto('users')
    ->set([
        'name' => 'Alice',
        'email' => 'alice@example.com',
        'created_at' => date('Y-m-d H:i:s')
    ])
    ->execute();
```

### Insert with RETURNING (PostgreSQL)

```php
$user = $driver->builder()
    ->insertInto('users')
    ->set(['name' => 'Alice', 'email' => 'alice@example.com'])
    ->returning(['id', 'created_at'])
    ->queryRow();

echo "Created user with ID: {$user->id}";
```

### Using values() and valuesMany()

Alternative syntax for inserts:

```php
// Single row with values()
$driver->builder()
    ->insertInto('users')
    ->values(['name' => 'Alice', 'email' => 'alice@example.com'])
    ->execute();

// Multiple rows with valuesMany()
$driver->builder()
    ->insertInto('users')
    ->valuesMany([
        ['name' => 'Alice', 'email' => 'alice@example.com'],
        ['name' => 'Bob', 'email' => 'bob@example.com'],
    ])
    ->execute();
```

### Bulk Insert

For multiple rows, you can also use the driver's `insertMany()` method:

```php
$driver->insertMany('users', [
    ['name' => 'Alice', 'email' => 'alice@example.com'],
    ['name' => 'Bob', 'email' => 'bob@example.com'],
]);
```

## UPDATE

### Basic Update

```php
$affected = $driver->builder()
    ->update('users')
    ->set([
        'status' => 'inactive',
        'updated_at' => date('Y-m-d H:i:s')
    ])
    ->where([['id', '=', 1]])
    ->execute();

echo "Updated $affected rows";
```

### Update with Conditions

```php
$driver->builder()
    ->update('users')
    ->set(['last_login' => date('Y-m-d H:i:s')])
    ->where([
        ['status', '=', 'active'],
        ['email_verified', '=', true]
    ])
    ->execute();
```

### Update with RETURNING (PostgreSQL)

```php
$updated = $driver->builder()
    ->update('users')
    ->set(['status' => 'premium'])
    ->where([['id', '=', 1]])
    ->returning(['id', 'name', 'status'])
    ->queryRow();
```

### Update with Subquery

```php
$driver->builder()
    ->update('orders')
    ->set(['status' => 'archived'])
    ->where("customer_id IN (SELECT id FROM customers WHERE inactive = true)")
    ->execute();
```

## DELETE

### Basic Delete

```php
$affected = $driver->builder()
    ->deleteFrom('users')
    ->where([['id', '=', 1]])
    ->execute();
```

### Delete with Conditions

```php
$driver->builder()
    ->deleteFrom('sessions')
    ->where([
        ['expires_at', '<', date('Y-m-d H:i:s')],
    ])
    ->execute();
```

### Delete with USING (PostgreSQL)

```php
$driver->builder()
    ->deleteFrom('order_items')
    ->using('orders')
    ->where('order_items.order_id = orders.id')
    ->where([['orders.status', '=', 'cancelled']])
    ->execute();
```

### Delete with RETURNING (PostgreSQL)

```php
$deleted = $driver->builder()
    ->deleteFrom('users')
    ->where([['status', '=', 'spam']])
    ->returning(['id', 'email'])
    ->queryAll();

foreach ($deleted as $user) {
    echo "Deleted: {$user->email}\n";
}
```

## UPSERT (INSERT ... ON CONFLICT)

### PostgreSQL

```php
$driver->builder()
    ->insertInto('users')
    ->set([
        'email' => 'alice@example.com',
        'name' => 'Alice',
        'login_count' => 1
    ])
    ->onConflict('email', [
        'name' => 'Alice',
        'login_count' => 'login_count + 1'  // Raw SQL expression
    ])
    ->execute();
```

Multiple conflict columns:

```php
$driver->builder()
    ->insertInto('metrics')
    ->set([
        'user_id' => 1,
        'date' => '2024-01-01',
        'views' => 1
    ])
    ->onConflict(['user_id', 'date'], [
        'views' => 'metrics.views + 1'
    ])
    ->execute();
```

### MySQL

```php
$driver->builder()
    ->insertInto('users')
    ->set([
        'email' => 'alice@example.com',
        'name' => 'Alice',
        'login_count' => 1
    ])
    ->onDuplicateKeyUpdate([
        'name' => 'Alice',
        'login_count' => 'login_count + 1'
    ])
    ->execute();
```

## REPLACE INTO (MySQL)

MySQL's `REPLACE INTO` deletes the existing row and inserts a new one:

```php
$driver->builder()
    ->replaceInto('users')
    ->set([
        'id' => 1,
        'name' => 'Alice',
        'email' => 'alice@example.com'
    ])
    ->execute();
```

**Note**: This is MySQL-specific. For PostgreSQL, use `INSERT ... ON CONFLICT`.

## TRUNCATE

Remove all rows from a table efficiently:

```php
$driver->builder()
    ->truncateTable('logs')
    ->execute();
```

**Warning**: TRUNCATE cannot be rolled back in some databases and resets auto-increment counters.

## Safety: Always Use WHERE

**Warning**: UPDATE and DELETE without WHERE affect all rows!

```php
// DANGEROUS - updates ALL users!
$driver->builder()
    ->update('users')
    ->set(['status' => 'inactive'])
    ->execute();  // No WHERE clause!

// SAFE - updates specific user
$driver->builder()
    ->update('users')
    ->set(['status' => 'inactive'])
    ->where([['id', '=', 1]])
    ->execute();
```

Consider adding a safeguard in your code:

```php
function safeUpdate(Builder $builder): int
{
    $sql = (string) $builder;
    if (stripos($sql, 'WHERE') === false) {
        throw new \RuntimeException('UPDATE without WHERE clause is not allowed');
    }
    return $builder->execute();
}
```

## Transactions

Wrap multiple mutations in a transaction:

```php
$driver->begin(function($driver) {
    $driver->builder()
        ->insertInto('orders')
        ->set(['user_id' => 1, 'total' => 100])
        ->execute();

    $driver->builder()
        ->update('inventory')
        ->set(['quantity' => 'quantity - 1'])
        ->where([['product_id', '=', 42]])
        ->execute();

    return true;  // Commit
});
```

See [Transactions](../advanced/transactions.md) for more details.

## Return Values

| Method | Returns |
|--------|---------|
| `execute()` | Number of affected rows |
| `queryRow()` | Single returned row (with RETURNING) |
| `queryAll()` | All returned rows (with RETURNING) |
