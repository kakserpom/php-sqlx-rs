# php-sqlx-rs

A PHP extension powered by Rust 🦀 and [SQLx](https://github.com/launchbadge/sqlx), enabling safe, fast, and expressive
database access with additional SQL syntax. It's built using
the [ext-php-rs]((https://github.com/davidcole1340/ext-php-rs))
crate.

The project's goals are centered on providing a **secure** and **ergonomic** way to interact with SQL-based DBM systems
without
any compromise on performance. The author's not big on PHP, but as a security researcher he understood the necessity of
modernizing the toolkit of the great many PHP developers. The idea came up, and bish bash bosh, a couple of weekends
later the project was all but done. More to come.

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
    Sqlx\Driver::OPT_ASSOC_ARRAYS => true,
    Sqlx\Driver::OPT_PERSISTENT_NAME => 'main_db'
]);
```

#### queryRow / queryAll

```php
$row = $driver->queryRow('SELECT * FROM users WHERE id = $id', [
    'id' => 1,
]);

$rows = $driver->queryAll("SELECT * FROM users WHERE status = \$status", [
    'status' => 'active',
]);
```

You can also use `:param` and `:1` placeholders:

```php
$row = $driver->queryRow("SELECT * FROM users WHERE id = :id", [
    'id' => 1,
]);

$row = $driver->queryRow("SELECT * FROM users WHERE id = :1", [1]);
```

#### queryMaybeRow

Same as `queryRow`, but returns `null` if not found.

#### execute

```php
$affected = $driver->execute("UPDATE users SET status = \$status WHERE id = \$id", [
    'id' => 1,
    'status' => 'inactive'
]);
```

#### insert

```php
$affected = $driver->insert("users", [
    'name' => 'Alice',
    'email' => 'alice@example.com'
]);
```

#### dry

```php
[$sql, $params] = $driver->dry("SELECT * FROM logs WHERE level = \$level", [
    'level' => 'warn',
]);
```

---

### Sqlx\PreparedQuery

Prepared query bound to a driver:

```php
$query = $driver->prepare("SELECT * FROM logs WHERE level = \$level");
$rows = $query->queryAll(['level' => 'warn']);
```

All the same methods as `Driver` are supported:

- `execute()`
- `queryRow()` / `queryRowAssoc()` / `queryRowObj()`
- `queryAll()` / `queryAllAssoc()` / `queryAllObj()`

---

### Sqlx\OrderBy

A helper class for rendering safe `ORDER BY` clauses from user input.

```php
$orderBy = new Sqlx\OrderBy([
    "name",
    "created_at",
    "posts" => "COUNT(posts.*)"
]);

// Equivalent to: ORDER BY name ASC, COUNT(posts.*) DESC
$rendered = $orderBy([
    ["name", "ASC"],
    ["posts", "DESC"]
]);
```

The `$rendered` value can be passed as a parameter to an SQL query with a placeholder:

```php
$driver->queryAll("SELECT * FROM users ORDER BY ?", [$rendered]);
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

Nested arrays are automatically flattened and bound in-order.

### Painless `IN (?)`

Passing an array as a parameter to a single placeholder automatically expands it:

```php
var_dump($driver->queryAll(
    'SELECT * FROM people WHERE name IN (?)',
    [['Peter', 'John', 'Jane']]
));
```

Example output:

```php
array(1) {
  [0] =>
  object(stdClass)#2 (2) {
    ["name"] => string(4) "John"
    ["age"] => int(22)
  }
}
```

---

## JSON Support

PostgreSQL `json` and `jsonb` types are automatically decoded into PHP arrays or objects:

```php
var_dump($driver->queryRow(
    'SELECT $1::json AS json',
    ['{"foo": ["bar", "baz"]}'
]));
/* Output:
object(stdClass)#3 (1) {
  ["json"] =>
  object(stdClass)#2 (1) {
    ["foo"] =>
    array(2) {
      [0] => string(3) "bar"
      [1] => string(3) "baz"
    }
  }
} */
var_dump($driver->queryRowAssoc(
    'SELECT $1::json AS json',
    ['{"foo": ["bar", "baz"]}'
]));
/* Output:
 array(1) {
  ["json"] =>
  array(1) {
    ["foo"] =>
    array(2) {
      [0] => string(3) "bar"
      [1] => string(3) "baz"
    }
  }
} */
```

---

## BigInt Support

PostgreSQL `BIGINT` values are safely mapped to PHP integers:

```php
var_dump($driver->queryRow('SELECT ((1::BIGINT << 62) - 1) * 2 + 1 AS largest')->largest);
```

Output:

```php
int(9223372036854775807)
```

---

## Notes

- The AST cache reduces repeated parsing overhead and speeds up query rendering.
- Supports both positional `$1`, `:1` and named `$param`, `:param` placeholders automatically.

---

## License

MIT

