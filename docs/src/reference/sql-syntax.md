# SQL Syntax Reference

Complete reference for php-sqlx augmented SQL syntax.

## Placeholders

### Positional Placeholders

```sql
-- Question mark (standard)
SELECT * FROM users WHERE id = ? AND status = ?

-- Numbered (PostgreSQL style)
SELECT * FROM users WHERE id = $1 AND status = $2

-- Colon-numbered
SELECT * FROM users WHERE id = :1 AND status = :2
```

### Named Placeholders

```sql
-- Dollar-sign (recommended)
SELECT * FROM users WHERE id = $id AND status = $status

-- Colon-prefix
SELECT * FROM users WHERE id = :id AND status = :status
```

## Type-Safe Placeholders

### Scalar Types

| Placeholder | Type | Accepts |
|-------------|------|---------|
| `?i` | Integer | Integers, numeric strings |
| `?u` | Unsigned Integer | Non-negative integers |
| `?d` | Decimal | Integers, floats, numeric strings |
| `?ud` | Unsigned Decimal | Non-negative decimals |
| `?s` | String | Strings only |
| `?j` | JSON | Arrays, objects |

```sql
SELECT * FROM users WHERE age >= ?u
SELECT * FROM products WHERE price = ?d
SELECT * FROM users WHERE name = ?s
INSERT INTO events (data) VALUES (?j)
```

### Nullable Types

Prefix with `n` for nullable:

| Placeholder | Type |
|-------------|------|
| `?ni` | Nullable Integer |
| `?nu` | Nullable Unsigned Integer |
| `?nd` | Nullable Decimal |
| `?nud` | Nullable Unsigned Decimal |
| `?ns` | Nullable String |

```sql
UPDATE users SET manager_id = ?ni WHERE id = ?i
```

### Array Types

Suffix with `a` for arrays:

| Placeholder | Type |
|-------------|------|
| `?ia` | Integer Array |
| `?ua` | Unsigned Integer Array |
| `?da` | Decimal Array |
| `?uda` | Unsigned Decimal Array |
| `?sa` | String Array |
| `?ja` | JSON Array |

```sql
SELECT * FROM users WHERE id IN (?ia)
SELECT * FROM products WHERE category IN (?sa)
```

### Named Type-Safe Placeholders

```sql
SELECT * FROM users WHERE age >= $min_age:u AND name = $name:s
SELECT * FROM users WHERE id IN ($ids:ia)
```

## Conditional Blocks

### Basic Syntax

```sql
SELECT * FROM users
WHERE 1=1
{{ AND status = $status }}
{{ AND role = $role }}
```

Block is included only if all referenced parameters are provided and non-null.

### Nested Blocks

```sql
SELECT * FROM orders
WHERE 1=1
{{ AND customer_id = $customer_id
   {{ AND status = $status }}
}}
```

### Multiple Parameters in Block

```sql
SELECT * FROM users
WHERE 1=1
{{ AND created_at BETWEEN $start_date AND $end_date }}
```

All parameters must be provided for the block to be included.

## IN Clause Expansion

### Array Expansion

```sql
-- Input
SELECT * FROM users WHERE id IN (?)
-- With params: [[1, 2, 3]]

-- Output (PostgreSQL)
SELECT * FROM users WHERE id IN ($1, $2, $3)
-- With params: [1, 2, 3]
```

### Empty Array Handling

```sql
-- Input with empty array
SELECT * FROM users WHERE id IN (?)
-- With params: [[]]

-- Output (with OPT_COLLAPSIBLE_IN = true)
SELECT * FROM users WHERE FALSE
```

For NOT IN:
```sql
-- Input with empty array
SELECT * FROM users WHERE id NOT IN (?)

-- Output
SELECT * FROM users WHERE TRUE
```

## Query Builder WHERE Syntax

### Array Format

```php
// [column, operator, value]
[['status', '=', 'active']]
[['age', '>=', 18]]
[['name', 'LIKE', '%john%']]
[['id', 'IN', [1, 2, 3]]]
[['deleted_at', 'IS NULL']]
```

### Supported Operators

| Operator | Example |
|----------|---------|
| `=` | `['status', '=', 'active']` |
| `!=`, `<>` | `['status', '!=', 'deleted']` |
| `>`, `>=` | `['age', '>=', 18]` |
| `<`, `<=` | `['age', '<', 65]` |
| `LIKE` | `['name', 'LIKE', '%john%']` |
| `ILIKE` | `['name', 'ILIKE', '%john%']` (PostgreSQL) |
| `IN` | `['id', 'IN', [1, 2, 3]]` |
| `NOT IN` | `['id', 'NOT IN', [1, 2, 3]]` |
| `IS NULL` | `['deleted_at', 'IS NULL']` |
| `IS NOT NULL` | `['verified_at', 'IS NOT NULL']` |

### OR Conditions

```php
use function Sqlx\OR_;

// AND (status = 'active' OR status = 'pending')
[
    OR_([
        ['status', '=', 'active'],
        ['status', '=', 'pending']
    ])
]

// Complex: active AND (admin OR moderator)
[
    ['active', '=', true],
    OR_([
        ['role', '=', 'admin'],
        ['role', '=', 'moderator']
    ])
]
```

## Database-Specific Output

### Identifier Quoting

| Database | Style | Example |
|----------|-------|---------|
| PostgreSQL | Double quotes | `"table_name"` |
| MySQL | Backticks | `` `table_name` `` |
| MSSQL | Brackets | `[table_name]` |

### Parameter Markers

| Database | Style | Example |
|----------|-------|---------|
| PostgreSQL | Numbered | `$1, $2, $3` |
| MySQL | Question marks | `?, ?, ?` |
| MSSQL | Named | `@p1, @p2, @p3` |

### Boolean Values

| Database | TRUE | FALSE |
|----------|------|-------|
| PostgreSQL | `TRUE` | `FALSE` |
| MySQL | `TRUE` | `FALSE` |
| MSSQL | `1` | `0` |

### Unicode Strings

| Database | Style |
|----------|-------|
| PostgreSQL | `'text'` |
| MySQL | `'text'` |
| MSSQL | `N'text'` |

## Escaping Rules

### String Escaping

Single quotes are doubled:
```sql
'O''Brien'  -- Represents O'Brien
```

### LIKE Pattern Escaping

`metaQuoteLike()` escapes `%` and `_`:
```php
$pattern = $driver->metaQuoteLike("100%");
// Returns: '100\%'
```

### Identifier Escaping

Special characters in identifiers:
```php
$driver->quoteIdentifier("my-table");
// PostgreSQL: "my-table"
// MySQL: `my-table`
// MSSQL: [my-table]
```

## Examples

### Dynamic Search

```sql
SELECT * FROM products
WHERE 1=1
{{ AND category = $category }}
{{ AND price >= $min_price }}
{{ AND price <= $max_price }}
{{ AND name LIKE $search }}
ORDER BY {{ $sort_column }} {{ $sort_dir }}
LIMIT $limit OFFSET $offset
```

### Type-Safe User Input

```sql
SELECT * FROM users
WHERE age >= ?u
AND status IN (?sa)
AND balance >= ?ud
```

### Complex Filtering

```sql
SELECT u.*, COUNT(o.id) as order_count
FROM users u
LEFT JOIN orders o ON o.user_id = u.id
WHERE 1=1
{{ AND u.status = $status }}
{{ AND u.role IN ($roles) }}
{{ AND u.created_at >= $since }}
GROUP BY u.id
{{ HAVING COUNT(o.id) >= $min_orders }}
ORDER BY {{ $order_by }}
LIMIT ?u
```
