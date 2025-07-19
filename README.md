# SQLx PHP Extension

The extension is powered by Rust 🦀 and [SQLx](https://github.com/launchbadge/sqlx), enabling safe, fast, and expressive
database access with additional SQL syntax. Bundled with a [**query builder**](QUERY-BUILDER.md).

**Postgres**, **MySQL** and **Mssql** are supported.

The project's goals are centered on providing a **secure** and **ergonomic** way to interact with SQL-based DBM systems
without any compromise on performance. The author's not big on PHP, but as a security researcher he understood the
necessity of modernizing the toolkit of great many PHP developers. The idea came up, and bish bash bosh, a couple of
weekends later the project was all but done. More to come.

The project is still kind of experimental, so any feedback/ideas will be greatly appreciated!

It's built using [ext-php-rs](https://github.com/davidcole1340/ext-php-rs).

## Features

- AST-based SQL augmentation (e.g., conditional blocks)
- Named parameters with `$param`, `:param`, or positional `:1` syntax
- Automatic result conversion to PHP arrays or objects
- Painless `IN (?)` / `NOT IN (?)` clauses expansion and collapse
- Safe and robust `ORDER BY` / `GROUP BY` clauses
- Pagination with `PAGINATE`
- Safe and robust `SELECT`
- SQL transactions are supported in full
- Powerful Query Builder.
- Native JSON support (with lazy decoding and [SIMD](https://docs.rs/simd-json/latest/simd_json/) 🚀)
- Optional persistent connections (with connection pooling)

---

## Augmented SQL Syntax

This extension introduces a powerful SQL preprocessor that supports conditional blocks, optional fragments, and named
parameters. Both positional (`?`, `$1`, `:1`) and named (`$param`, `:param`) placeholders are supported. All can be used
interchangeably.

---

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

### Painless `IN (?)` / `NOT IN (?)` clauses expansion and collapse

Passing an array as a parameter to a single placeholder automatically expands it:

```php
// Expands to: SELECT * FROM people WHERE name IN (?, ?, ?)
// with values ['Peter', 'John', 'Jane']
$rows = $driver->queryAll(
  'SELECT * FROM people WHERE name IN :names', [
    'names' => ['Peter', 'John', 'Jane']
  ]
);
```

Omitting the parameter or passing an empty array will make `IN` collapse into boolean `FALSE`.

```php
var_dump($driver->dry(
  'SELECT * FROM people WHERE name IN :names', [
    'names' => []
  ]
));
```

```
array(2) {
  [0]=>
  string(53) "SELECT * FROM people WHERE FALSE /* name IN :names */"
  [1]=>
  array(0) {
  }
}
```

Same goes for `NOT IN`, except it will collapse into boolean `TRUE`.

```php
var_dump($driver->dry(
  'SELECT * FROM people WHERE name NOT IN :names', [
    'names' => []
  ]
));
```

```
array(2) {
  [0]=>
  string(56) "SELECT * FROM people WHERE TRUE /* name NOT IN :names */"
  [1]=>
  array(0) {
  }
}
```

> Makes sense, right? Given that `x IN (1, 2, 3)` is sugar for `(x = 1 OR x = 2 OR x = 3)`
> and `x NOT IN (1, 2, 3)` is sugar for `(x != 1 AND x != 2 AND x != 3)`.

Keep in mind that you can not only use it in `WHERE`, but also in `ON` clauses when joining.

> It is true that in simpler cases of `IN :empty` like the above example you could just
> immediately return an empty result set without sending it to DBMS, but there could be a `JOIN`
> or a `UNION`.
---

### Safe and robust `ORDER BY` / `GROUP BY` clauses

`Sql\ByClause` helper class for safe `ORDER BY` / `GROUP BY` clauses from user input.

> __SAFETY CONCERNS__
> - 🟢 You can safely pass any user input as sorting settings.
> - 🔴 Do NOT pass unsanitized user input into `ByClause` constructor to avoid SQL injection vulnerabilities. If you
    absolutely have to, then apply `array_values()` to the argument to avoid SQL injections.

**Examples**:

```php
// Let's define allowed columns
$orderBy = new Sqlx\ByClause([
    'name',
    'created_at',
    'random' => 'RANDOM()'
]);

// Equivalent to: SELECT * FROM users ORDER BY `name` ASC, RANDOM()
$driver->queryAll('SELECT * FROM users ORDER BY :order_by', [
  'order_by' => $orderBy([
    ['name', Sqlx\ByClause::ASC],
    'random'
  ])
]);

```

Field names are case-sensitive, but they get trimmed.

---

Pagination with `PAGINATE`
---
`Sql\PaginateClause` helper class for safe pagination based on user input.

```php
// Let's define pagination rules
$pagination = new Sqlx\PaginateClause;
$pagination->perPage(5);
$pagination->maxPerPage(20);

// Equivalent to: SELECT * FROM people ORDER by id LIMIT 5 OFFSET 500
$rows = $driver->queryAll(
  'SELECT * FROM people ORDER by id PAGINATE :pagination', [
    'pagination' => $pagination(100)
  ]
);


// Equivalent to: SELECT * FROM people ORDER by id LIMIT 10 OFFSET 1000
$rows = $driver->queryAll(
  'SELECT * FROM people ORDER by id PAGINATE :pagination', [
    'pagination' => $pagination(100, 10)
  ]
);

// Equivalent to: SELECT * FROM people ORDER by id LIMIT 5 OFFSET 0
$rows = $driver->queryAll(
  'SELECT * FROM people ORDER by id PAGINATE :pagination', [
    'pagination' => $pagination()
  ]
);
```

You can safely pass any unsanitized values as arguments, but keep in mind that `perPage()`/`maxPerPage()`/
`defaultPerPage()`
functions take a positive integer and throw an exception otherwise.

---

### Safe and robust `SELECT`

A helper class for safe `SELECT` clauses from user input.

> __SAFETY CONCERNS__
> - 🟢 You can safely pass any user input as invocation argument.
> - 🔴 Do NOT pass unsanitized user input into `SelectClause` constructor to avoid SQL injection vulnerabilities. If you
    absolutely have to, then apply `array_values()` to the argument to avoid SQL injections.

**Examples**:

```php
$select = new Sqlx\SelectClause([
    'id',
    'created_at',
    'name',
    'num_posts' => 'COUNT(posts.*)'
]);

// Equivalent to: SELECT `id`, `name`, COUNT(posts.*) AS `num_posts` FROM users
$rows = $driver->queryAll('SELECT :select FROM users', [
  'select' => $select(['id','name', 'num_posts'])
]);
```

Note that column names are case-sensitive, but they get trimmed.

--- 

## Transactions

```php
$driver->begin(function($driver) {
    // All queries inside this function will be wrapped in a transaction.
    // You can use all driver functions here.
    
    $driver->insert('users', ['name' => 'John', 'age' => 25]);
    $driver->insert('users', ['name' => 'Mary', 'age' => 20]);
    
    // return false; 
});
```

A `ROLLBACK` happens if the closure returns `false` or throws an exception.
Otherwise, a `COMMIT` gets sent when functions finishes normally.

Additional supported methods to be called from inside a closure:

- `savepoint(name: String)`
- `rollbackToSavepoint(name: String)`
- `releaseSavepoint(name: String)`

---

## Parameter types

Supported parameter types:

```php
"text"
123
3.14
true
[1, 2, 3]
```

> ✅ PostgreSQL `BIGINT` values are safely mapped to PHP integers:
> ```php
>  var_dump($driver->queryValue('SELECT ((1::BIGINT << 62) - 1) * 2 + 1');
>  // Output: int(9223372036854775807)
>```

Nested arrays are automatically flattened and bound in order.

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

## [Query Builder](QUERY-BUILDER.md) overview

You can fluently build SQL queries using `$driver->builder()`:

```php
$query = $driver->builder()
    ->select("*")
    ->from("users")
    ->where(["active" => true])
    ->orderBy("created_at DESC")
    ->limit(10);
```

The builder supports most SQL clauses:

* `select()`, `from()`, `where()`, `groupBy()`, `orderBy()`, `having()`
* `insert()`, `values()`, `valuesMany()`, `returning()`
* `update()`, `set()`
* `deleteFrom()`, `using()`
* `with()`, `withRecursive()`
* `join()`, `leftJoin()`, `rightJoin()`, `fullJoin()`, `naturalJoin()`, `crossJoin()`
* `onConflict()`, `onDuplicateKeyUpdate()`
* `limit()`, `offset()`, `paginate()`
* `union()`, `unionAll()`
* `forUpdate()`, `forShare()`
* `truncateTable()`, `raw()`, `end()`

Each method returns the builder itself, allowing fluent chaining.

---

### Insert: Multi-row Example

Use `valuesMany()` to insert multiple rows in one statement:

```php
$driver->builder()->insert("users")->valuesMany([
    ["Alice", "alice@example.com"],
    ["Bob", "bob@example.com"]
]);

// or with named keys:
$driver->builder()->insert("users")->valuesMany([
    ["name" => "Alice", "email" => "alice@example.com"],
    ["name" => "Bob",   "email" => "bob@example.com"]
]);
```

---

### Executing the Query

After building the query, you can run it just like with prepared statements:

```php
$query->execute();
// OR
$row = $query->queryRow();
// OR
$rows = $query->queryAll();
```

You can also preview the rendered SQL and parameters without executing:

```php
var_dump((string) $query); // SQL with placeholders rendered
```

<br />

### ❗️[Query Builder guide](QUERY-BUILDER.md)❗

<br />

---

## Installation

Install with [`cargo-php`](https://github.com/davidcole1340/ext-php-rs):

```bash
cargo install cargo-php --locked
cd php-sqlx-cdylib
```

For macOS:

```bash
export MACOSX_DEPLOYMENT_TARGET=$(sw_vers -productVersion | tr -d '\n')
export RUSTFLAGS="-C link-arg=-undefined -C link-arg=dynamic_lookup"
#export RUSTFLAGS="-Zmacro-backtrace -Zproc-macro-backtrace -Clink-arg=-undefined -Clink-arg=dynamic_lookup"
cargo install cargo-php --locked
cd php-sqlx-cdylib
cargo php install --release --yes
```

---

## API

### `Sql\DriverFactory`

```php
$driver = Sqlx\DriverFactory::make("postgres://user:pass@localhost/db");
```

Or with options:

```php
$driver = Sqlx\DriverFactory::make([
    Sqlx\DriverOptions::OPT_URL => 'postgres://user:pass@localhost/db',
    Sqlx\DriverOptions::OPT_ASSOC_ARRAYS => true,   // return arrays instead of objects
    Sqlx\DriverOptions::OPT_PERSISTENT_NAME => 'main_db'
    Sqlx\DriverOptions::OPT_MAX_CONNECTIONS => 5,
    Sqlx\DriverOptions::OPT_MIN_CONNECTIONS => 0,
    //Sqlx\DriverOptions::OPT_MAX_LIFETIME => "30 min",
    Sqlx\DriverOptions::OPT_IDLE_TIMEOUT => 120,
    Sqlx\DriverOptions::OPT_ACQUIRE_TIMEOUT => 10,
]);
```

<details>
<summary>DriverOptions reference</summary>

| Option                      | Type                    | Description                                                                         | Default      |
|-----------------------------|-------------------------|-------------------------------------------------------------------------------------|--------------|
| `OPT_URL`                   | `string`                | **Required.** Database connection URL. Example: `postgres://user:pass@localhost/db` | *(required)* |
| `OPT_ASSOC_ARRAYS`          | `bool`                  | If true, rows are returned as associative arrays instead of objects                 | `false`      |
| `OPT_PERSISTENT_NAME`       | `string \| null`        | Enables persistent connection pool reuse under a given name                         | `null`       |
| `OPT_MAX_CONNECTIONS`       | `int > 0`               | Maximum number of connections in the pool                                           | `10`         |
| `OPT_MIN_CONNECTIONS`       | `int ≥ 0`               | Minimum number of connections to keep alive                                         | `0`          |
| `OPT_MAX_LIFETIME`          | `string \| int \| null` | Max lifetime of a pooled connection. Accepts `"30s"`, `"5 min"`, or seconds         | `null`       |
| `OPT_IDLE_TIMEOUT`          | `string \| int \| null` | Idle timeout before closing pooled connections. Same format as above                | `null`       |
| `OPT_ACQUIRE_TIMEOUT`       | `string \| int \| null` | Timeout to wait for a connection from the pool                                      | `null`       |
| `OPT_TEST_BEFORE_ACQUIRE`   | `bool`                  | Whether to test connections before acquisition                                      | `false`      |
| `OPT_COLLAPSIBLE_IN`        | `bool`                  | Enables automatic collapsing of `IN ()` / `NOT IN ()` into `FALSE` / `TRUE`         | `true`       |
| `OPT_AST_CACHE_SHARD_COUNT` | `int > 0`               | Number of internal SQL AST cache shards (advanced tuning)                           | `8`          |
| `OPT_AST_CACHE_SHARD_SIZE`  | `int > 0`               | Max number of entries per AST cache shard                                           | `256`        |

</details>

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

| Method                   | Returns                                 | Notes                                  |
|--------------------------|-----------------------------------------|----------------------------------------|
| `queryDictionary()`      | `array<string \| int, array \| object>` | key = first column, value = entire row |
| `queryDictionaryAssoc()` | `array<string \| int, array>`           | ∟ forces associative arrays            |
| `queryDictionaryObj()`   | `array<string \| int, object>`          | ∟ forces objects                       |

> - ⚠️ First column **must** be scalar, otherwise an exception will be thrown.
> - 🔀 The iteration order is preserved.

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

| Method                         | Returns                       | Notes                                                   |
|--------------------------------|-------------------------------|---------------------------------------------------------|
| `queryColumnDictionary()`      | `array<string \| int, mixed>` | key = first column, value = second column               |
| `queryColumnDictionaryAssoc()` | ↑                             | ∟ enforces array mode for second column if it's a JSON  |
| `queryColumnDictionaryObj()`   | ↑                             | ∟ enforces object mode for second column if it's a JSON |

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
array(1) {
  ["IT"]=>
  array(2) {
    [0]=>
    object(stdClass)#2 (2) {
      ["department"]=>
      string(2) "IT"
      ["name"]=>
      string(5) "Alice"
    }
    [1]=>
    object(stdClass)#3 (2) {
      ["department"]=>
      string(2) "IT"
      ["name"]=>
      string(3) "Bob"
    }
  }
}
*/
```

---

## Performance

Well, it's blazingly fast. Nothing like similar projects written in userland PHP.

The AST cache eliminates repeated parsing overhead and speeds up query rendering.

JSON is handled with SIMD.

### Rust benchmark suite

It is useful for measuring the performance of backend parts such as AST parsing/rendering.

Command:

```shell
cargo bench
```

Here are M1 Max results for parsing and rendering a hefty query. No caching, naturally.

```
     Running benches/benchmark.rs (target/release/deps/benchmark-eaed67cfaa034b35)
Gnuplot not found, using plotters backend
Ast::parse_small        time:   [2.5877 µs 2.5928 µs 2.5981 µs]
                        change: [−0.4151% −0.0902% +0.2146%] (p = 0.57 > 0.05)
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe

Ast::parse_big          time:   [7.2626 µs 7.2785 µs 7.2958 µs]
                        change: [+0.1364% +0.4485% +0.7694%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

Ast::render_big         time:   [1.9188 µs 1.9215 µs 1.9243 µs]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild

```

### PHP benchmarks

Command:

```shell
cd benches
curl -s https://raw.githubusercontent.com/composer/getcomposer.org/f3108f64b4e1c1ce6eb462b159956461592b3e3e/web/installer | php -- --quiet
./composer.phar require phpbench/phpbench --dev
./vendor/bin/phpbench run benchmark.php
```

Or use Docker:

```shell
docker build . -t php-sqlx-benches
docker run php-sqlx-benches
```

M1 Max results:

```
    benchDrySmall...........................I0 - Mo2.009μs (±0.00%)
    benchDryBig.............................I0 - Mo5.105μs (±0.00%)
    benchSelect1kRows.......................I0 - Mo927.888μs (±0.00%)
```

## License

MIT
