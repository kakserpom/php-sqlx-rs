# Joins

The Query Builder supports all standard SQL join types.

## Join Types

### INNER JOIN

Returns rows that have matching values in both tables:

```php
$orders = $driver->builder()
    ->select(['orders.*', 'customers.name AS customer_name'])
    ->from('orders')
    ->innerJoin('customers', 'customers.id = orders.customer_id')
    ->queryAll();

// Shorthand alias
$builder->join('customers', 'customers.id = orders.customer_id');
```

### LEFT JOIN

Returns all rows from the left table and matched rows from the right table:

```php
$users = $driver->builder()
    ->select(['users.*', 'profiles.bio'])
    ->from('users')
    ->leftJoin('profiles', 'profiles.user_id = users.id')
    ->queryAll();
// Users without profiles will have NULL bio
```

### RIGHT JOIN

Returns all rows from the right table and matched rows from the left table:

```php
$departments = $driver->builder()
    ->select(['departments.*', 'employees.name'])
    ->from('employees')
    ->rightJoin('departments', 'departments.id = employees.department_id')
    ->queryAll();
// Departments without employees will be included
```

### FULL OUTER JOIN

Returns all rows when there's a match in either table:

```php
$all = $driver->builder()
    ->select(['a.*', 'b.*'])
    ->from('table_a a')
    ->fullJoin('table_b b', 'a.key = b.key')
    ->queryAll();
```

### CROSS JOIN

Returns the Cartesian product of both tables:

```php
$combinations = $driver->builder()
    ->select(['colors.name AS color', 'sizes.name AS size'])
    ->from('colors')
    ->crossJoin('sizes')
    ->queryAll();
// Every color paired with every size
```

### NATURAL JOIN

Joins on columns with matching names:

```php
$result = $driver->builder()
    ->select('*')
    ->from('orders')
    ->naturalJoin('customers')
    ->queryAll();
// Automatically joins on columns with the same name
```

## Join Conditions

### Simple Equality

```php
$builder->leftJoin('orders', 'orders.customer_id = customers.id');
```

### Multiple Conditions

```php
$builder->leftJoin(
    'orders',
    'orders.customer_id = customers.id AND orders.status = \'completed\''
);
```

### With Parameters

```php
$builder->leftJoin(
    'orders',
    'orders.customer_id = customers.id AND orders.created_at > ?',
    ['2024-01-01']
);
```

## Multiple Joins

Chain multiple joins:

```php
$orders = $driver->builder()
    ->select([
        'orders.id',
        'customers.name AS customer',
        'products.name AS product',
        'order_items.quantity'
    ])
    ->from('orders')
    ->innerJoin('customers', 'customers.id = orders.customer_id')
    ->innerJoin('order_items', 'order_items.order_id = orders.id')
    ->innerJoin('products', 'products.id = order_items.product_id')
    ->queryAll();
```

## Table Aliases

Use aliases for self-joins or clarity:

```php
// Self-join: employees with their managers
$employees = $driver->builder()
    ->select(['e.name AS employee', 'm.name AS manager'])
    ->from('employees e')
    ->leftJoin('employees m', 'm.id = e.manager_id')
    ->queryAll();
```

## Subquery Joins

Join with a subquery:

```php
$topCustomers = $driver->builder()
    ->select(['customer_id', 'SUM(total) AS total_spent'])
    ->from('orders')
    ->groupBy('customer_id')
    ->having('SUM(total) > 1000');

$result = $driver->builder()
    ->select(['c.*', 't.total_spent'])
    ->from('customers c')
    ->innerJoin("({$topCustomers}) t", 't.customer_id = c.id')
    ->queryAll();
```

## Conditional Joins

Include joins conditionally:

```php
$builder = $driver->builder()
    ->select(['users.*'])
    ->from('users');

if ($includeProfile) {
    $builder
        ->select(['profiles.bio', 'profiles.avatar'])
        ->leftJoin('profiles', 'profiles.user_id = users.id');
}

if ($includeStats) {
    $builder
        ->select(['COUNT(orders.id) AS order_count'])
        ->leftJoin('orders', 'orders.user_id = users.id')
        ->groupBy('users.id');
}

$users = $builder->queryAll();
```

## Join Performance Tips

### Index Join Columns

Ensure columns used in join conditions are indexed:

```sql
CREATE INDEX idx_orders_customer_id ON orders(customer_id);
```

### Limit Joined Data

Filter before joining when possible:

```php
// Less efficient - joins all, then filters
$builder
    ->from('orders')
    ->innerJoin('customers', 'customers.id = orders.customer_id')
    ->where([['orders.status', '=', 'pending']]);

// More efficient - filter subquery, then join
$pendingOrders = $driver->builder()
    ->select('*')
    ->from('orders')
    ->where([['status', '=', 'pending']]);

$builder
    ->from("({$pendingOrders}) o")
    ->innerJoin('customers', 'customers.id = o.customer_id');
```

### Select Only Needed Columns

```php
// Slow - selects all columns from all tables
$builder->select('*')->from('a')->join('b', '...')->join('c', '...');

// Fast - select only what you need
$builder
    ->select(['a.id', 'a.name', 'b.status', 'c.total'])
    ->from('a')
    ->join('b', '...')
    ->join('c', '...');
```
