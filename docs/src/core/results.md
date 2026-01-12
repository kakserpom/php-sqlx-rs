# Result Handling

php-sqlx provides flexible options for handling query results, including objects, arrays, and specialized dictionary formats.

## Result Formats

### Objects (Default)

By default, rows are returned as `stdClass` objects:

```php
$user = $driver->queryRow("SELECT id, name, email FROM users WHERE id = ?", [1]);

echo $user->id;     // 1
echo $user->name;   // Alice
echo $user->email;  // alice@example.com
```

### Associative Arrays

Use `*Assoc` method variants:

```php
$user = $driver->queryRowAssoc("SELECT id, name, email FROM users WHERE id = ?", [1]);

echo $user['id'];     // 1
echo $user['name'];   // Alice
echo $user['email'];  // alice@example.com
```

Or configure globally:

```php
$driver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://localhost/mydb",
    DriverOptions::OPT_ASSOC_ARRAYS => true,
]);

// Now all methods return arrays by default
$user = $driver->queryRow("SELECT * FROM users WHERE id = ?", [1]);
echo $user['name'];  // Works!
```

### Force Object Mode

Use `*Obj` variants to always get objects, even with `OPT_ASSOC_ARRAYS` enabled:

```php
$user = $driver->queryRowObj("SELECT * FROM users WHERE id = ?", [1]);
echo $user->name;  // Always an object
```

## Type Conversion

php-sqlx automatically converts database types to appropriate PHP types:

| Database Type | PHP Type |
|---------------|----------|
| INTEGER, BIGINT, SMALLINT | `int` |
| FLOAT, DOUBLE, REAL | `float` |
| DECIMAL, NUMERIC | `string` (to preserve precision) |
| VARCHAR, TEXT, CHAR | `string` |
| BOOLEAN | `bool` |
| NULL | `null` |
| DATE, TIMESTAMP | `string` (ISO format) |
| JSON, JSONB | `array` or `object` (lazy-decoded) |
| BYTEA, BLOB | `string` (binary) |
| UUID | `string` |
| ARRAY | `array` |

## JSON Handling

JSON columns are automatically decoded:

```php
// Database column 'metadata' contains: {"role": "admin", "permissions": ["read", "write"]}
$user = $driver->queryRow("SELECT metadata FROM users WHERE id = ?", [1]);

echo $user->metadata->role;  // admin
print_r($user->metadata->permissions);  // ['read', 'write']
```

With associative arrays mode:

```php
$user = $driver->queryRowAssoc("SELECT metadata FROM users WHERE id = ?", [1]);
echo $user['metadata']['role'];  // admin
```

### Lazy JSON Decoding

JSON values are decoded lazily on first access, improving performance when you don't need all JSON fields.

## Null Handling

NULL values from the database are returned as PHP `null`:

```php
$user = $driver->queryRow("SELECT name, deleted_at FROM users WHERE id = ?", [1]);

if ($user->deleted_at === null) {
    echo "User is active";
}
```

## Empty Results

Different methods handle empty results differently:

```php
// Throws SqlxException if no rows
$user = $driver->queryRow("SELECT * FROM users WHERE id = ?", [999]);

// Returns null if no rows
$user = $driver->queryMaybeRow("SELECT * FROM users WHERE id = ?", [999]);

// Returns empty array if no rows
$users = $driver->queryAll("SELECT * FROM users WHERE id = ?", [999]);
```

## Dictionary Results

### queryDictionary

Returns a map where the first column is the key:

```php
$users = $driver->queryDictionary("SELECT id, name, email FROM users");
// [
//   1 => {id: 1, name: 'Alice', email: 'alice@...'},
//   2 => {id: 2, name: 'Bob', email: 'bob@...'},
// ]

// Access by ID
if (isset($users[42])) {
    echo $users[42]->name;
}
```

### queryColumnDictionary

Maps first column to second column:

```php
$names = $driver->queryColumnDictionary("SELECT id, name FROM users");
// [1 => 'Alice', 2 => 'Bob', 3 => 'Carol']

echo $names[1];  // Alice
```

### queryGroupedDictionary

Groups rows by the first column:

```php
$byDepartment = $driver->queryGroupedDictionary(
    "SELECT department, id, name FROM employees"
);
// [
//   'Engineering' => [{id: 1, name: 'Alice'}, {id: 2, name: 'Bob'}],
//   'Sales' => [{id: 3, name: 'Carol'}],
// ]

foreach ($byDepartment['Engineering'] as $employee) {
    echo $employee->name . "\n";
}
```

### queryGroupedColumnDictionary

Groups a single column's values:

```php
$emailsByDept = $driver->queryGroupedColumnDictionary(
    "SELECT department, email FROM employees"
);
// [
//   'Engineering' => ['alice@...', 'bob@...'],
//   'Sales' => ['carol@...'],
// ]
```

## Column Selection

For single-value and column queries, specify which column to extract:

```php
// By name
$email = $driver->queryValue(
    "SELECT id, name, email FROM users WHERE id = ?",
    [1],
    'email'
);

// By index (0-based)
$email = $driver->queryValue(
    "SELECT id, name, email FROM users WHERE id = ?",
    [1],
    2  // third column
);
```

## Large Result Sets

For very large result sets, consider:

1. **Pagination**: Use LIMIT/OFFSET or cursor-based pagination
2. **Streaming**: Not yet supported; use pagination instead
3. **Column selection**: Only SELECT the columns you need

```php
// Don't do this for large tables
$all = $driver->queryAll("SELECT * FROM huge_table");

// Do this instead
$page = $driver->queryAll(
    "SELECT id, name FROM huge_table ORDER BY id LIMIT ? OFFSET ?",
    [100, 0]
);
```

## Performance Tips

1. **Select only needed columns**:
   ```php
   // Slow - fetches all columns
   $user = $driver->queryRow("SELECT * FROM users WHERE id = ?", [1]);

   // Fast - fetches only what's needed
   $user = $driver->queryRow("SELECT id, name FROM users WHERE id = ?", [1]);
   ```

2. **Use appropriate query methods**:
   ```php
   // Inefficient - fetches entire row for one value
   $row = $driver->queryRow("SELECT COUNT(*) as cnt FROM users");
   $count = $row->cnt;

   // Efficient - fetches just the value
   $count = $driver->queryValue("SELECT COUNT(*) FROM users");
   ```

3. **Use dictionaries for lookups**:
   ```php
   // Inefficient - N queries
   foreach ($orderIds as $id) {
       $order = $driver->queryRow("SELECT * FROM orders WHERE id = ?", [$id]);
   }

   // Efficient - 1 query
   $orders = $driver->queryDictionary(
       "SELECT id, * FROM orders WHERE id IN (?)",
       [$orderIds]
   );
   foreach ($orderIds as $id) {
       $order = $orders[$id] ?? null;
   }
   ```
