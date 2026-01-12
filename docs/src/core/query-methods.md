# Query Methods

php-sqlx provides a rich set of query methods for different use cases. Each method is optimized for its specific purpose.

## Method Naming Convention

Methods follow a consistent naming pattern:

- **Base method**: `queryRow()` - uses driver's default format
- **Assoc variant**: `queryRowAssoc()` - returns associative arrays
- **Obj variant**: `queryRowObj()` - returns objects

## Single Row Queries

### queryRow

Fetch exactly one row. Throws an exception if no rows found.

```php
$user = $driver->queryRow("SELECT * FROM users WHERE id = ?", [1]);
echo $user->name;
```

### queryMaybeRow

Fetch one row or `null` if not found.

```php
$user = $driver->queryMaybeRow("SELECT * FROM users WHERE id = ?", [999]);
if ($user === null) {
    echo "Not found";
}
```

## Multiple Row Queries

### queryAll

Fetch all matching rows as an array.

```php
$users = $driver->queryAll("SELECT * FROM users WHERE active = ?", [true]);
foreach ($users as $user) {
    echo $user->name . "\n";
}
```

Returns an empty array if no rows match.

## Single Value Queries

### queryValue

Fetch a single value from the first row. Throws if no rows.

```php
// First column by default
$count = $driver->queryValue("SELECT COUNT(*) FROM users");

// Specific column by name
$name = $driver->queryValue(
    "SELECT id, name, email FROM users WHERE id = ?",
    [1],
    'name'
);

// Specific column by index
$name = $driver->queryValue(
    "SELECT id, name, email FROM users WHERE id = ?",
    [1],
    1  // second column (0-indexed)
);
```

### queryMaybeValue

Fetch a single value or `null` if no rows.

```php
$email = $driver->queryMaybeValue(
    "SELECT email FROM users WHERE id = ?",
    [999]
);
// Returns null if user doesn't exist
```

## Column Queries

### queryColumn

Fetch a single column from all rows as a flat array.

```php
$emails = $driver->queryColumn("SELECT email FROM users WHERE active = ?", [true]);
// Returns: ['alice@example.com', 'bob@example.com', ...]

// Specific column
$names = $driver->queryColumn(
    "SELECT id, name, email FROM users",
    null,
    'name'
);
```

## Dictionary Queries

Dictionary queries return associative arrays where the first column becomes the key.

### queryDictionary

Map the first column to entire rows.

```php
$usersById = $driver->queryDictionary("SELECT id, name, email FROM users");
// Returns: [
//   1 => {id: 1, name: 'Alice', email: 'alice@...'},
//   2 => {id: 2, name: 'Bob', email: 'bob@...'},
// ]

$user = $usersById[1];
echo $user->name;  // Alice
```

### queryColumnDictionary

Map the first column to the second column only.

```php
$emailsById = $driver->queryColumnDictionary("SELECT id, email FROM users");
// Returns: [1 => 'alice@example.com', 2 => 'bob@example.com']
```

### queryGroupedDictionary

Group multiple rows by the first column.

```php
$usersByRole = $driver->queryGroupedDictionary(
    "SELECT role, id, name FROM users ORDER BY role, name"
);
// Returns: [
//   'admin' => [{id: 1, name: 'Alice'}, {id: 3, name: 'Carol'}],
//   'user' => [{id: 2, name: 'Bob'}],
// ]
```

### queryGroupedColumnDictionary

Group second column values by the first column.

```php
$emailsByDepartment = $driver->queryGroupedColumnDictionary(
    "SELECT department, email FROM users"
);
// Returns: [
//   'Engineering' => ['alice@...', 'bob@...'],
//   'Sales' => ['carol@...'],
// ]
```

## Data Modification

### execute

Execute INSERT, UPDATE, or DELETE statements. Returns affected row count.

```php
$affected = $driver->execute(
    "UPDATE users SET last_login = NOW() WHERE id = ?",
    [1]
);
echo "Updated $affected rows";
```

### insert

Insert a single row using an associative array.

```php
$driver->insert('users', [
    'name' => 'Alice',
    'email' => 'alice@example.com',
    'created_at' => date('Y-m-d H:i:s'),
]);
```

### insertMany

Bulk insert multiple rows efficiently.

```php
$driver->insertMany('users', [
    ['name' => 'Alice', 'email' => 'alice@example.com'],
    ['name' => 'Bob', 'email' => 'bob@example.com'],
    ['name' => 'Carol', 'email' => 'carol@example.com'],
]);
```

### upsert

Insert or update on conflict (PostgreSQL/MySQL).

```php
// PostgreSQL
$driver->upsert(
    'users',
    ['id' => 1, 'name' => 'Alice', 'email' => 'alice@example.com'],
    ['id'],           // conflict columns
    ['name', 'email'] // columns to update on conflict
);
```

## Schema Introspection

### describeTable

Get column metadata for a table.

```php
$columns = $driver->describeTable('users');
foreach ($columns as $col) {
    echo "{$col['name']}: {$col['type']}";
    if ($col['nullable']) echo " (nullable)";
    echo "\n";
}
```

Returns an array with:
- `name` - Column name
- `type` - Data type (e.g., "varchar(255)")
- `nullable` - Whether NULL is allowed
- `default` - Default value
- `ordinal` - Column position (1-based)

With schema:

```php
$columns = $driver->describeTable('users', 'public');
```

## Utility Methods

### quote

Safely quote a value for SQL.

```php
$quoted = $driver->quote("O'Brien");
// Returns: 'O''Brien'
```

### quoteLike

Quote a string for use in LIKE patterns, escaping `%` and `_`.

```php
$pattern = $driver->quoteLike("100%");
$driver->queryAll("SELECT * FROM products WHERE name LIKE ?", ["%$pattern%"]);
```

### quoteIdentifier

Quote a table or column name.

```php
$quoted = $driver->quoteIdentifier("user-data");
// PostgreSQL: "user-data"
// MySQL: `user-data`
// MSSQL: [user-data]
```

### dry

Render a query without executing it. Useful for debugging.

```php
[$sql, $params] = $driver->dry(
    "SELECT * FROM users WHERE id = ? AND status = $status",
    ['id' => 1, 'status' => 'active']
);
echo $sql;    // SELECT * FROM users WHERE id = $1 AND status = $2
print_r($params);  // [1, 'active']
```

## Method Reference Table

| Method | Returns | Throws on Empty |
|--------|---------|-----------------|
| `queryRow` | Single row | Yes |
| `queryMaybeRow` | Row or `null` | No |
| `queryAll` | Array of rows | No (empty array) |
| `queryValue` | Single value | Yes |
| `queryMaybeValue` | Value or `null` | No |
| `queryColumn` | Array of values | No (empty array) |
| `queryDictionary` | Key => Row map | No |
| `queryColumnDictionary` | Key => Value map | No |
| `queryGroupedDictionary` | Key => Rows map | No |
| `queryGroupedColumnDictionary` | Key => Values map | No |
| `execute` | Affected rows | No |
