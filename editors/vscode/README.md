# PHP-SQLx VS Code Extension

Syntax highlighting for PHP-SQLx augmented SQL syntax.

## Features

- **Conditional blocks**: `{{ AND status = $status }}`
- **Type-safe placeholders**: `?i`, `?s`, `?ni`, `?ia`, `?nuda`, etc.
- **Named placeholders**: `$name`, `:name`, `$id!i`, `:status!ns`
- **SQL injection into PHP strings**: Automatically highlights SQL in PHP string literals

## Installation

### From VSIX (local)

1. Package the extension:
   ```bash
   cd editors/vscode
   npx vsce package
   ```

2. Install in VS Code:
    - Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
    - Type "Install from VSIX"
    - Select the generated `.vsix` file

### Manual (development)

1. Copy or symlink this folder to your VS Code extensions directory:
    - **Mac**: `~/.vscode/extensions/php-sqlx`
    - **Windows**: `%USERPROFILE%\.vscode\extensions\php-sqlx`
    - **Linux**: `~/.vscode/extensions/php-sqlx`

2. Reload VS Code

## Highlighted Syntax

### Conditional Blocks

```sql
SELECT *
FROM users
WHERE 1 = 1 {{ AND status = $status }}
    {{
  AND created_at
    > $since }}
```

The `{{ }}` delimiters are highlighted as control keywords.

### Type-Safe Placeholders

| Syntax                        | Description      |
|-------------------------------|------------------|
| `?i`, `?s`, `?d`, `?u`, `?ud` | Typed positional |
| `?ni`, `?ns`, `?nd`           | Nullable typed   |
| `?ia`, `?sa`, `?da`           | Typed arrays     |
| `$name!i`, `:id!s`            | Named with type  |
| `$name!ni`, `:id!ns`          | Named nullable   |

### Named Placeholders

```sql
SELECT *
FROM users
WHERE id = $id
  AND name = :name
```

## Color Customization

Add to your `settings.json` to customize colors:

```json
{
  "editor.tokenColorCustomizations": {
    "textMateRules": [
      {
        "scope": "keyword.control.conditional.sqlx",
        "settings": {
          "foreground": "#C586C0",
          "fontStyle": "bold"
        }
      },
      {
        "scope": "variable.parameter.placeholder.typed.sqlx",
        "settings": {
          "foreground": "#4EC9B0",
          "fontStyle": "bold"
        }
      },
      {
        "scope": "variable.parameter.placeholder.sqlx",
        "settings": {
          "foreground": "#9CDCFE"
        }
      }
    ]
  }
}
```
