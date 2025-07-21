# SQLx PHP Extension: Query Builder Guide

This guide covers the full functionality of the SQLx query builder provided by the `builder()` method in
`Sqlx\PgDriver`, `Sqlx\MySqlDriver`, or `Sqlx\MssqlDriver`. It allows safe, fluent SQL construction in PHP using
composable camelCase methods.

---

## Getting Started

```php
$driver = new Sqlx\PgDriver("postgres://user:pass@localhost/db");
$builder = $driver->builder();
```

Use `builder()` to start a fluent query. Every method returns the builder instance for chaining.

```php
$builder->select("*")->from("users")->where(["active" => true]);
```

`$builder->builder()` creates a fresh new query builder instance (not a clone), useful for building subqueries.

Keep in mind that the query builder does not guarantee correctness of your query, every method you call just appends
a piece to your query. You need to call them in the right order.

---

## Complex Subquery Example

```php
$builder = $driver->builder()
    ->with(
        'active_users',
        $driver->builder()
            ->select(['id', 'name', 'created_at'])
            ->from('users')
            ->where([
                ['status', '=', 'active'],
                ['deleted_at', 'IS NULL'],
            ])
    )
    ->select(
        SelectClause::allowed([
            'comment_id' => 'comments.id',
            'comment_body' => 'comments.body',
            'user_name' => 'active_users.name',
            'created_at' => 'comments.created_at',
            'num_likes' => 'COUNT(likes.id)',
        ])->input(['comment_id', 'user_name', 'comment_body', 'num_likes'])
    )
    ->from('comments')
    ->leftJoin('active_users', 'comments.user_id = active_users.id')
    ->leftJoin('likes', 'likes.comment_id = comments.id')
    ->where([
        ['comments.created_at', '>', '2024-01-01'],
        ['comments.status', '=', 'published'],
        OR_([
            ['active_users.name', 'ILIKE', '%john%'],
            ['active_users.name', 'ICONTAINS', 'jane'],
        ]),
    ])
    ->orderBy(['comments.created_at' => 'DESC']);
```

### Explanation:

* `with(...)` defines a CTE (active\_users) filtering active, non-deleted users.
* `SelectClause::allowed(...)->input(...)` protects against SQL injection.
* `leftJoin(...)` joins the CTE and `likes` table.
* `where(...)` includes published comments after Jan 1st, optionally matching "john" or "jane" in user names.
* `orderBy(...)` sorts by latest comment.

This is a full-featured example showcasing CTEs, subqueries, aggregation, joins, and safe selection.

---

## Subquery examples:

**Simple subquery with WHERE IN:**

```php
$sub = $driver->builder()
  ->select("user_id")
  ->from("logins")
  ->where(["success" => true])
  ->groupBy("user_id")
  ->having([["COUNT(*)", ">", 3]]);

$main = $driver->builder()
  ->select("*")
  ->from("users")
  ->where([["id", "in", $sub]]);
```

**Using subquery as a derived table with JOIN:**

```php
$latestLogins = $driver->builder()
  ->select(["user_id", "MAX(created_at) AS last_login"])
  ->from("logins")
  ->groupBy("user_id");

$main = $driver->builder()
  ->select("u.*, l.last_login")
  ->from(["u" => "users"])
  ->join(["l" => $latestLogins], "u.id = l.user_id");
```

**Subquery in DELETE USING clause:**

```php
$inactive = $driver->builder()
  ->select("id")
  ->from("users")
  ->where(["active" => false]);

$driver->builder()
  ->deleteFrom("sessions")
  ->using($inactive)
  ->where([["sessions.user_id", "=", "users.id"]]);
```

---

## SELECT Queries

### select()

```php
$builder->select("*");
$builder->select(["id", "name", "created_at"]);
```

Supports:

* raw string: `"id, name"`
* array of columns
* `SelectClause` object

### from()

```php
$builder->from("users");
$builder->from(["u" => "users"]);
```

Supports:

* string
* array of aliases (e.g. `["u" => "users"]`)

### where() / having()

```php
$builder->where(["status" => "active"]);

$builder->where([
  ["created_at", ">", "2024-01-01"]
]);

$builder->having([
  ["COUNT(id)", ">", 10]
]);

$builder->having([
  "COUNT(id) > ?" => [10]
]);
```

Supports:

* associative arrays (key = column, value = value)
* indexed arrays of `[column, operator, value]`
* nested `OrClause`

Standard operators:

| Operator                           | Description                               |
|------------------------------------|-------------------------------------------|
| `=`, `eq`                          | Equal                                     |
| `!=`, `<>`, `neq`, `ne`            | Not equal                                 |
| `>`, `gt`                          | Greater than                              |
| `>=`, `gte`                        | Greater than or equal                     |
| `<`, `lt`                          | Less than                                 |
| `<=`, `lte`                        | Less than or equal                        |
| `LIKE`, `like`                     | SQL `LIKE`, case-sensitive                |
| `NOT LIKE`, `not like`, `nlike`    | Negated `LIKE`, case-sensitive            |
| `ILIKE`, `ilike`                   | Case-insensitive `LIKE` (PostgreSQL only) |
| `NOT ILIKE`, `not ilike`, `nilike` | Negated case-insensitive `LIKE`           |
| `IN`, `in`                         | Membership test                           |
| `NOT IN`, `not in`                 | Negated membership test                   |
| `IS NULL`, `is null`               | Tests for NULL                            |
| `IS NOT NULL`, `is not null`       | Tests for NOT NULL                        |

---

Additional `LIKE`/`ILIKE` derivatives with auto-escaping of meta-characters (`%` and `_`):

| Operator                          | Description                           |
|-----------------------------------|---------------------------------------|
| `IEQ` / `NOT IEQ`                 | `ILIKE value` / `NOT ILIKE value`     |
| `CONTAINS` / `NOT CONTAINS`       | `LIKE %value%` / `NOT LIKE %value%`   |
| `ICONTAINS` / `NOT ICONTAINS`     | `ILIKE %value%` / `NOT ILIKE %value%` |
| `STARTSWITH` / `NOT STARTSWITH`   | `LIKE value%` / `LIKE value%`         |
| `ISTARTSWITH` / `NOT ISTARTSWITH` | `ILIKE value%` /  `NOT ILIKE value%`  |
| `ENDSWITH` / `NOT ENDSWITH`       | `LIKE %value` / `NOT LIKE %value`     |
| `IENDSWITH` / `NOT IENDSWITH`     | `ILIKE %value` / `NOT ILIKE %value`   |

Example:

```php
$driver->builder()
  ->select('*')
  ->from('users')->where([
    ['phone', 'startswith', '+7']
  ]);
```

---

### groupBy()

```php
$builder->groupBy("type");
$builder->groupBy(["type", "region"]);
```

### orderBy()

```php
$builder->orderBy("name DESC");
$builder->orderBy(["created_at" => "DESC"]);
```

Supports:

* string
* array of columns or `ByClause`

### limit() / offset()

```php
$builder->limit(10);
$builder->limit(10, 50); // with offset
$builder->offset(100);
```

### forUpdate() / forShare()

```php
$builder->forUpdate();
$builder->forShare();
```

---

## INSERT / REPLACE / UPDATE / DELETE Queries

### insertInto()

```php
$builder->insertInto("users");
```

Appends an `INSERT INTO` clause to the query.

You can follow it with `set()` or `values()` to define inserted values.

---

### replaceInto()

```php
$builder->replaceInto("users");
```

Appends a `REPLACE INTO` clause (MySQL-specific). Use `set()` or `values()` afterward.

---

### deleteFrom()

```php
$builder->deleteFrom("users");
```

Starts a `DELETE FROM` clause. Can be used with `where()` and `using()` (for PostgreSQL-style joins):

```php
$builder->deleteFrom("sessions")->using("users")->where([
    "sessions.user_id = users.id",
    ["users.active", "=", false],
]);
```

---

### update()

```php
$builder->update("users");
```

Starts an `UPDATE` statement targeting the given table.

Use in combination with `set()` and `where()`:

```php
$builder->update("users")
  ->set(["name" => "John", "active" => true])
  ->where(["id" => 123]);
```

---

### set()

```php
$builder->set(["column" => "value"]);
```

Specifies column-value pairs for `UPDATE` or `INSERT`:

* Keys are column names
* Values are literals, placeholders, or subqueries

Example:

```php
$builder->insertInto("users")->set([
  "name" => "Alice",
  "created_at = NOW()",
]);
```

---

Let me know if you want to also document:

* `values()` for insert
* `onConflictDoUpdate()` and `onDuplicateKeyUpdate()`
* `returning()` for PostgreSQL / MSSQL inserts/updates/deletes
  ...
