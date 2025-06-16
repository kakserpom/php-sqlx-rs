# php-sqlx-rs

A PHP extension powered by Rust 🦀 and [SQLx](https://github.com/launchbadge/sqlx), enabling safe, fast, and expressive
database access with additional SQL syntax. It's built using
the [ext-php-rs](\(https://github.com/davidcole1340/ext-php-rs\)) crate.

The project's goals are centered on providing a **secure** and **ergonomic** way to interact with SQL-based DBM systems
without any compromise on performance. The author's not big on PHP, but as a security researcher he understood the
necessity of modernizing the toolkit of the great many PHP developers. The idea came up, and bish bash bosh, a couple of
weekends later the project was all but done. More to come.

The project is still kind of experimental, so any feedback/ideas will be greatly appreciated!

## Features

- Optional persistent connections (with connection pooling)
- AST-based SQL augmentation (e.g., conditional blocks)
- Named parameters with `$param`, `:param`, or positional `:1` syntax
- Automatic result conversion to PHP arrays or objects
- Native JSON and bigint support
- Painless `IN (?)` clause expansion

---

## Augmented SQL Syntax

This extension introduces a powerful SQL preprocessor that supports conditional blocks, optional fragments, and named
parameters.

### Conditional Blocks

Wrap parts of your query with double braces `{{ ... }}` to make them conditional:

```sql
SELECT *
FROM users
WHERE TRUE {{ AND name = $name }}
    {{
  AND status = $status }}
```

If a named parameter used inside the block is not provided, the entire block is omitted from the final query.

Nested conditional blocks are supported:

```sql
SELECT *
FROM logs
WHERE TRUE {{ AND date > $since {{
  AND level = $level }} }}
```

In the above example the `level` condition will only be rendered when both `$level` and `$since` are set.

---

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
    Sqlx\Driver::OPT_URL => 'postgres://user:pass@localhost/db',
    Sqlx\Driver::OPT_ASSOC_ARRAYS => true,   // return arrays instead of objects
    Sqlx\Driver::OPT_PERSISTENT_NAME => 'main_db'
]);
```

#### Basics

- `assocArrays(): bool` – returns **true** if the driver is currently set to produce associative arrays instead of
  objects.
- `prepare(string $sql): Sqlx\PreparedQuery` – returns a reusable prepared query object bound to the same driver.

#### Row helpers

| Method                 | Returns                             | Notes                     |
|------------------------|-------------------------------------|---------------------------|
| `queryRow()`           | first row (array \| object)         | error if no rows returned |
| `queryRowAssoc()`      | first row (array)                   | ∟ enforces array mode     |
| `queryRowObj()`        | first row (object)                  | ∟ enforces object mode    |
| `queryMaybeRow()`      | first row (array \| object \| null) | null if no rows returned  |
| `queryMaybeRowAssoc()` | first row (array \| null)           | ∟ enforces array mode     |
| `queryMaybeRowObj()`   | first row (object \| null)          | ∟ enforces object mode    |            

#### Column helpers (single-row)

| Method                       | Returns                        | Notes                                   |
|------------------------------|--------------------------------|-----------------------------------------|
| `queryRowColumn()`           | first row column value         | error if no rows returned               |
| `queryRowColumnAssoc()`      | ↑                              | ∟ enforces array mode for JSON objects  |
| `queryRowColumnObj()`        | ↑                              | ∟ enforces object mode for JSON objects |
| `queryMaybeRowColumn()`      | first row column value or null | null if no rows returned                |
| `queryMaybeRowColumnAssoc()` | ↑                              | ∟ enforces array mode for JSON objects  |
| `queryMaybeRowColumnObj()`   | ↑                              | ∟ enforces object mode for JSON objects |

#### Column helpers (multi-row)

| Method               | Returns                                | Notes                                   |
|----------------------|----------------------------------------|-----------------------------------------|
| `queryColumn()`      | array of column's values from each row | error if no rows returned               |
| `queryColumnAssoc()` | ↑                                      | ∟ enforces array mode for JSON objects  |
| `queryColumnObj()`   | ↑                                      | ∟ enforces object mode for JSON objects |

#### List helpers (all rows)

| Method            | Returns               |
|-------------------|-----------------------|
| `queryAll()`      | array of rows         |
| `queryAllAssoc()` | array of assoc arrays |
| `queryAllObj()`   | array of objects      |

#### Mutation helpers

- `execute(string $sql, array $param``s = null): int` – run **INSERT/UPDATE/DELETE** and return affected count.
- `insert(string $table, array $row): int` – convenience wrapper around `INSERT`.

#### Utilities

- `dry(string $sql, array $params = null): array` – render final SQL + bound params without executing. Handy for
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
- `queryRow*()` / `queryMaybeRow*()`
- `queryRowColumn*()` / `queryMaybeRowColumn*()`
- `queryAll*()` / `queryColumn*()`

---

### Sqlx\OrderBy

A helper class for rendering safe `ORDER BY` clauses from user input.

```php
$orderBy = new Sqlx\OrderBy([
    'name',
    'created_at',
    'posts' => 'COUNT(posts.*)'
]);

// Equivalent to: ORDER BY name ASC, COUNT(posts.*) DESC
$rendered = $orderBy([
    ['name', Sqlx\OrderBy::ASC],
    ['posts', Sqlx\OrderBy::DESC]
]);

$driver->queryAll('SELECT * FROM users ORDER BY ?', [$rendered]);
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

### Painless `IN (?)`

Passing an array as a parameter to a single placeholder automatically expands it:

```php
var_dump($driver->queryAll(
    'SELECT * FROM people WHERE name IN (?)',
    [['Peter', 'John', 'Jane']]
));
```

---

## JSON Support

PostgreSQL `json` and `jsonb` types are automatically decoded into PHP arrays or objects.

```php
var_dump($driver->queryRow(
    'SELECT $1::json AS json',
    ['{"foo": ["bar", "baz"]}']
));
```

---

## BigInt Support

PostgreSQL `BIGINT` values are safely mapped to PHP integers:

```php
var_dump($driver->queryRow('SELECT ((1::BIGINT << 62) - 1) * 2 + 1 AS largest')->largest);
```

---

## Notes

- The AST cache reduces repeated parsing overhead and speeds up query rendering.
- Supports both positional `$1`, `:1` and named `$param`, `:param` placeholders automatically.

---

## License

MIT

