# Augmented SQL Overview

php-sqlx extends standard SQL with powerful features that make dynamic queries safer and more ergonomic.

## What is Augmented SQL?

Augmented SQL is a superset of SQL that adds:

1. **Conditional Blocks** - Sections that are included/excluded based on parameter presence
2. **Type-Safe Placeholders** - Placeholders that validate parameter types
3. **Smart IN Clauses** - Automatic handling of arrays and empty sets
4. **Parameter Flexibility** - Mix positional and named parameters freely

These features are processed at the php-sqlx layer before the query reaches your database, so they work with any database backend.

## Quick Examples

### Conditional Blocks

Only include parts of a query when parameters are provided:

```php
$users = $driver->queryAll("
    SELECT * FROM users
    WHERE 1=1
    {{ AND status = $status }}
    {{ AND role = $role }}
    {{ AND created_at >= $since }}
", [
    'status' => 'active',
    // 'role' not provided - that block is omitted
    // 'since' not provided - that block is omitted
]);
// Executed: SELECT * FROM users WHERE 1=1 AND status = $1
```

### Type-Safe Placeholders

Ensure parameters match expected types:

```php
// Only accepts unsigned integers
$users = $driver->queryAll(
    "SELECT * FROM users WHERE age >= ?u",
    [18]  // OK
);

$users = $driver->queryAll(
    "SELECT * FROM users WHERE age >= ?u",
    [-5]  // Throws ParameterException!
);
```

### Smart IN Clauses

Arrays are automatically expanded:

```php
$users = $driver->queryAll(
    "SELECT * FROM users WHERE id IN (?)",
    [[1, 2, 3]]
);
// Executed: SELECT * FROM users WHERE id IN ($1, $2, $3)
```

Empty arrays become FALSE (configurable):

```php
$users = $driver->queryAll(
    "SELECT * FROM users WHERE id IN (?)",
    [[]]  // empty
);
// Executed: SELECT * FROM users WHERE FALSE
```

## How It Works

1. **Parse**: php-sqlx parses your SQL into an AST (Abstract Syntax Tree)
2. **Transform**: Conditional blocks and placeholders are resolved
3. **Render**: The final SQL is generated for your specific database
4. **Cache**: The AST is cached for repeated queries

The AST cache means that even complex augmented SQL has minimal overhead after the first execution.

## Database Compatibility

Augmented SQL features are translated to native syntax for each database:

| Feature | PostgreSQL | MySQL | MSSQL |
|---------|-----------|-------|-------|
| Placeholders | `$1, $2` | `?, ?` | `@p1, @p2` |
| Identifiers | `"name"` | `` `name` `` | `[name]` |
| Boolean | `TRUE/FALSE` | `TRUE/FALSE` | `1/0` |
| Unicode strings | `'text'` | `'text'` | `N'text'` |

## When to Use Augmented SQL

**Good use cases:**
- Dynamic filters based on user input
- Optional search criteria
- Building queries with variable conditions
- Type validation at the query level

**Consider alternatives for:**
- Very simple, static queries (no benefit)
- Extremely complex dynamic queries (use Query Builder instead)

## Next Steps

- [Conditional Blocks](./conditional-blocks.md) - Deep dive into `{{ }}` syntax
- [Type-Safe Placeholders](./type-safe-placeholders.md) - Learn all placeholder types
- [IN Clause Handling](./in-clause.md) - Array expansion and empty set handling
