# PHP-SQLx PHPStorm/IntelliJ Support

SQL syntax highlighting and language injection for PHP-SQLx in PHPStorm and other JetBrains IDEs.

## Language Injection

PHPStorm can automatically highlight SQL syntax inside PHP strings. Import the provided configuration:

1. Open **Settings** > **Editor** > **Language Injections**
2. Click the **Import** button (arrow icon)
3. Select `IntelliLang.xml` from this directory
4. Click **OK**

This enables SQL highlighting in:

- `$driver->queryAll('SELECT ...')`
- `$driver->execute('INSERT ...')`
- Any string starting with `SELECT`, `INSERT`, `UPDATE`, `DELETE`, etc.

## Manual Injection (Alternative)

If the import doesn't work, add injections manually:

1. Open **Settings** > **Editor** > **Language Injections**
2. Click **+** > **PHP Parameter**
3. Configure:
    - **Language**: SQL
    - **Parameter index**: 0
    - **Method name regex**: `queryAll|queryRow|queryMaybeRow|queryValue|execute|prepare`

## Highlighting Conditional Blocks

PHPStorm's built-in SQL doesn't recognize `{{ }}` blocks. Options:

### Option 1: Custom Inspection Suppression

Add this comment before SQLx queries to suppress "unknown token" warnings:

```php
/** @lang SQL */
$sql = 'SELECT * FROM users WHERE 1=1 {{ AND status = $status }}';
```

### Option 2: Use Heredoc with SQL Tag

```php
$sql = <<<SQL
    SELECT * FROM users
    WHERE 1=1
    {{ AND status = $status }}
    {{ AND created_at > $since }}
SQL;
```

## Live Templates

Import `live-templates.xml` for useful snippets:

1. Open **Settings** > **Editor** > **Live Templates**
2. Click **Import** (gear icon)
3. Select `live-templates.xml`

Available templates:

- `sqlxq` - SQLx query with parameters
- `sqlxcond` - Conditional block `{{ }}`
- `sqlxin` - IN clause with typed array
- `sqlxins` - INSERT statement
- `sqlxupd` - UPDATE statement

## Type-Safe Placeholder Reference

PHPStorm won't validate these, but here's a quick reference:

| Positional | Named        | Description      |
|------------|--------------|------------------|
| `?i`       | `$id!i`      | Integer          |
| `?u`       | `$age!u`     | Unsigned integer |
| `?d`       | `$price!d`   | Decimal          |
| `?ud`      | `$amount!ud` | Unsigned decimal |
| `?s`       | `$name!s`    | String           |
| `?ni`      | `$id!ni`     | Nullable integer |
| `?ia`      | `$ids!ia`    | Integer array    |
| `?sa`      | `$names!sa`  | String array     |
