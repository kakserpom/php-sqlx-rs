# php-sqlx-rs

A PHP extension powered by Rust 🦀 and [SQLx](https://github.com/launchbadge/sqlx), enabling safe, fast, and expressive
database access with additional SQL syntax. It's built using
the [ext-php-rs](https://github.com/davidcole1340/ext-php-rs) crate.

As of this moment only Postgres and MySQL are supported.

The project's goals are centered on providing a **secure** and **ergonomic** way to interact with SQL-based DBM systems
without any compromise on performance. The author's not big on PHP, but as a security researcher he understood the
necessity of modernizing the toolkit of the great many PHP developers. The idea came up, and bish bash bosh, a couple of
weekends later the project was all but done. More to come.

The project is still kind of experimental, so any feedback/ideas will be greatly appreciated!

## Features

- AST-based SQL augmentation (e.g., conditional blocks)
- Named parameters with `$param`, `:param`, or positional `:1` syntax
- Automatic result conversion to PHP arrays or objects
- Painless `IN (?)` clause expansion
- Safe and robust `ORDER BY`
- Native JSON and bigint support
- Optional persistent connections (with connection pooling)

---

## Augmented SQL Syntax

This extension introduces a powerful SQL preprocessor that supports conditional blocks, optional fragments, and named
parameters.

### Conditional Blocks

Wrap parts of your query with double braces `{{ ... }}` to make them conditional:

```
SELECT *
FROM users
WHERE TRUE
  {{ AND name = $name }}
  {{ AND status = $status }}
```

If a named parameter used inside the block is not provided, the entire block is omitted from the final query.

Nested conditional blocks are supported:

```
SELECT *
FROM logs
WHERE TRUE {{ AND date > $since {{ AND level = $level }} }}
```

In the above example the `level` condition will only be rendered when both `$level` and `$since` are set.

```
SELECT *
FROM logs
WHERE date > $since {{ AND level = $level }}
```

The above example will throw an exception if `$since` is not set.

---

### Painless `IN (?)`

Passing an array as a parameter to a single placeholder automatically expands it:

```php
var_dump($driver->queryAll(
    'SELECT * FROM people WHERE name IN (:names)',
    ['names' => ['Peter', 'John', 'Jane']]
));
```

### Safe and robust `ORDER BY`

A helper class for safe `ORDER BY` clauses from user input.

**SAFETY:**

- You can safely pass any user input as sorting settings.
- Do NOT pass user input into the `OrderBy` constructor to avoid SQL injection vulnerabilities.

**Examples**:

```php
$orderBy = new Sqlx\OrderBy([
    'name',
    'created_at',
    'posts' => 'COUNT(posts.*)'
]);

// Equivalent to: SELECT * FROM users ORDER BY `name` ASC, COUNT(posts.*) DESC
$driver->queryAll('SELECT * FROM users ORDER BY :order_by', ['order_by' => $orderBy([
    ['name', Sqlx\OrderBy::ASC],
    ['posts', Sqlx\OrderBy::DESC]
])]);

// This will throw an exception: Missing required placeholder `order_by`
$driver->queryAll('SELECT * FROM users ORDER BY :order_by', ['order_by' => $orderBy([
    ['zzzz', Sqlx\OrderBy::ASC],
])]);

// Equivalent to: SELECT * FROM users
$driver->queryAll('SELECT * FROM users {{ ORDER BY :order_by }}', ['order_by' => $orderBy([
    ['zzzz', Sqlx\OrderBy::ASC],
])]);
```

Note that the direction constants (`OrderBy::ASC` and `OrderBy::DESC`) are just strings (`'ASC'` and `'DESC'`).
You can pass strings (case-insensitive).

So this code works:

```php
// Equivalent to: SELECT * FROM users ORDER BY `name` DESC
$driver->queryAll('SELECT * FROM users {{ ORDER BY :order_by }}', ['order_by' => $orderBy([
['  name  ', ' DeSc  '],
])]);
```

Note that field names are case-sensitive. Incorrect `direction` string silently defaults to `ASC`.

---

## JSON Support

PostgreSQL/MySQL JSON types are automatically decoded into PHP arrays or objects.

```php
var_dump($driver->queryValue(
    'SELECT $1::json',
    ['{"foo": ["bar", "baz"]}']
));
/* Output:
object(stdClass)#2 (1) {
  ["foo"]=>
  array(2) {
    [0]=>
    string(3) "bar"
    [1]=>
    string(3) "baz"
  }
}*/

var_dump($driver->queryRow(
    'SELECT $1::json AS col',
    ['{"foo": ["bar", "baz"]}']
)->col->foo[0]);
// Output: string(3) "bar"
```

## Installation

Install with [`cargo-php`](https://github.com/davidcole1340/ext-php-rs):

```bash
cargo install cargo-php --locked
cargo php install --release --yes
```

For macOS:

```bash
export MACOSX_DEPLOYMENT_TARGET=$(sw_vers -productVersion | tr -d '\n')
export RUSTFLAGS="-C link-arg=-undefined -C link-arg=dynamic_lookup"
cargo install cargo-php --locked
cargo php install --release --yes
```

---

## API

### Sqlx\Driver

```php
$driver = new Sqlx\Driver("postgres://user:pass@localhost/db");
```

Or with options:

```php
$driver = new Sqlx\Driver([
    Sqlx\DriverOptions::OPT_URL => 'postgres://user:pass@localhost/db',
    Sqlx\DriverOptions::OPT_ASSOC_ARRAYS => true,   // return arrays instead of objects
    Sqlx\DriverOptions::OPT_PERSISTENT_NAME => 'main_db'
]);
```

#### Basics

- `assocArrays(): bool` – returns **true** if the driver is currently set to produce associative arrays instead of
  objects.
- `prepare(string $query): Sqlx\PreparedQuery` – returns a reusable prepared query object bound to the same driver.

#### Row helpers

| Method                 | Returns                             | Notes                         |
|------------------------|-------------------------------------|-------------------------------|
| `queryRow()`           | first row (array \| object)         | exception if no rows returned |
| `queryRowAssoc()`      | first row (array)                   | ∟ enforces array mode         |
| `queryRowObj()`        | first row (object)                  | ∟ enforces object mode        |
| `queryMaybeRow()`      | first row (array \| object \| null) | null if no rows returned      |
| `queryMaybeRowAssoc()` | first row (array \| null)           | ∟ enforces array mode         |
| `queryMaybeRowObj()`   | first row (object \| null)          | ∟ enforces object mode        |            

#### Column helpers (single-row)

| Method                   | Returns                        | Notes                                   |
|--------------------------|--------------------------------|-----------------------------------------|
| `queryValue()`           | first row column value         | exception if no rows returned           |
| `queryValueAssoc()`      | ↑                              | ∟ enforces array mode for JSON objects  |
| `queryValueObj()`        | ↑                              | ∟ enforces object mode for JSON objects |
| `queryMaybeValue()`      | first row column value or null | null if no rows returned                |
| `queryMaybeValueAssoc()` | ↑                              | ∟ enforces array mode for JSON objects  |
| `queryMaybeValueObj()`   | ↑                              | ∟ enforces object mode for JSON objects |

#### Column helpers (multi-row)

| Method               | Returns                                | Notes                                   |
|----------------------|----------------------------------------|-----------------------------------------|
| `queryColumn()`      | array of column's values from each row | exception if no rows returned           |
| `queryColumnAssoc()` | ↑                                      | ∟ enforces array mode for JSON objects  |
| `queryColumnObj()`   | ↑                                      | ∟ enforces object mode for JSON objects |

#### List helpers (all rows)

| Method            | Returns               |
|-------------------|-----------------------|
| `queryAll()`      | array of rows         |
| `queryAllAssoc()` | array of assoc arrays |
| `queryAllObj()`   | array of objects      |

#### Mutation helpers

- `execute(string $query, array $parameters = null): int` – run **INSERT/UPDATE/DELETE** and return affected count.
- `insert(string $table, array $row): int` – convenience wrapper around `INSERT`.

#### Utilities

- `dry(string $query, array $parameters = null): array` – render final SQL + bound parameters without executing. Handy
  for
  debugging.

---

### Sqlx\PreparedQuery

Prepared queries expose exactly the same surface as the driver, but without the SQL argument:

```php
$query = $driver->prepare("SELECT * FROM logs WHERE level = $level");
$rows  = $query->queryAll(['level' => 'warn']);
```

All helpers listed above have their prepared-query counterparts:

- `execute()`
- `queryRow()` / `queryRowAssoc()` / `queryRowObj()`
- `queryAll()` / `queryAllAssoc()` / `queryAllObj()`
- `queryDictionary()` / `queryDictionaryAssoc()` / `queryDictionaryObj()`
- `queryGroupedDictionary()` / `queryGroupedDictionaryAssoc()` / `queryGroupedDictionaryObj()`
- `queryColumnDictionary()` / `queryColumnDictionaryAssoc()` / `queryColumnDictionaryObj()`

---

### Dictionary helpers (first column as key, row as value)

| Method                   | Returns                         | Notes                                  |
|--------------------------|---------------------------------|----------------------------------------|
| `queryDictionary()`      | `array<string, array \|object>` | key = first column, value = entire row |
| `queryDictionaryAssoc()` | `array<string, array>`          | ∟ forces associative arrays            |
| `queryDictionaryObj()`   | `array<string, object>`         | ∟ forces objects                       |

> ⚠️ First column **must** be convertible to string, otherwise an exception will be thrown.  
> 🔀 The iteration order is preserved.

```php
var_dump($driver->queryGroupedColumnDictionary(
    'SELECT department, name FROM employees WHERE department IN (?)',
    [['IT', 'HR']]
));
/* Output:
array(2) {
  ["IT"]=> array("Alice", "Bob")
  ["HR"]=> array("Eve")
}
*/
```

---

### Column Dictionary helpers (first column as key, second as value)

| Method                         | Returns                | Notes                                                   |
|--------------------------------|------------------------|---------------------------------------------------------|
| `queryColumnDictionary()`      | `array<string, mixed>` | key = first column, value = second column               |
| `queryColumnDictionaryAssoc()` | ↑                      | ∟ enforces array mode for second column if it's a JSON  |
| `queryColumnDictionaryObj()`   | ↑                      | ∟ enforces object mode for second column if it's a JSON |

```php
var_dump($driver->queryColumnDictionary(
    'SELECT name, age FROM people WHERE name IN (?)',
    [["Peter", "John", "Jane"]]
));
/* Output:
array(1) {
  ["John"]=>
  int(22)
}
*/
```

---

### Grouped Dictionary helpers (first column as key, many rows per key)

| Method                          | Returns                                 | Notes                                             |
|---------------------------------|-----------------------------------------|---------------------------------------------------|
| `queryGroupedDictionary()`      | `array<string, array<array \| object>>` | key = first column, value = list of matching rows |
| `queryGroupedDictionaryAssoc()` | `array<string, array<array>`            | ∟ forces associative arrays                       |
| `queryGroupedDictionaryObj()`   | `array<string, array<object>>`          | ∟ forces objects                                  |

```php
var_dump($driver->queryGroupedDictionary(
    'SELECT department, name FROM employees WHERE department IN (?)',
    [['IT', 'HR']]
));
/* Output:
array(2) {
  ["IT"]=>
  array(2) {
    [0]=> array("department" => "IT", "name" => "Alice")
    [1]=> array("department" => "IT", "name" => "Bob")
  },
  ["HR"]=>
  array(1) {
    [0]=> array("department" => "HR", "name" => "Eve")
  }
}
*/
```

---

## Data Binding

Supported parameter types:

```php
"text"
123
3.14
true
[1, 2, 3]
```

Nested arrays are automatically flattened and bound in order.

---

## BigInt Support

PostgreSQL `BIGINT` values are safely mapped to PHP integers:

```php
var_dump($driver->queryValue('SELECT ((1::BIGINT << 62) - 1) * 2 + 1');
// Output: int(9223372036854775807)
```

---

## Notes

- The AST cache reduces repeated parsing overhead and speeds up query rendering.
- Supports both positional `$1`, `:1` and named `$param`, `:param` placeholders automatically.

---

## License

MIT

