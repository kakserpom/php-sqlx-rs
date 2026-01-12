# Introduction

**php-sqlx** is a modern, high-performance SQL database driver for PHP, written in Rust. It provides a powerful and ergonomic API for working with PostgreSQL, MySQL, and Microsoft SQL Server databases.

## Why php-sqlx?

Traditional PHP database extensions like PDO are written in C and have remained largely unchanged for years. php-sqlx takes a different approach:

- **Written in Rust** - Memory-safe, fast, and modern
- **Async under the hood** - Uses SQLx and Tokio for efficient I/O
- **Augmented SQL** - Extends SQL with conditional blocks and type-safe placeholders
- **Query Builder** - Fluent API for building complex queries safely
- **Connection Pooling** - Built-in connection pool with read replica support
- **Better DX** - Intuitive methods like `queryRow()`, `queryAll()`, `queryDictionary()`

## Key Features

### Multi-Database Support

```php
// PostgreSQL
$pg = Sqlx\DriverFactory::make("postgres://user:pass@localhost/mydb");

// MySQL
$mysql = Sqlx\DriverFactory::make("mysql://user:pass@localhost/mydb");

// Microsoft SQL Server
$mssql = Sqlx\DriverFactory::make("mssql://user:pass@localhost/mydb");
```

### Intuitive Query API

```php
// Get a single row
$user = $driver->queryRow("SELECT * FROM users WHERE id = ?", [42]);

// Get all rows
$users = $driver->queryAll("SELECT * FROM users WHERE active = ?", [true]);

// Get a single value
$count = $driver->queryValue("SELECT COUNT(*) FROM users");

// Get a column as array
$emails = $driver->queryColumn("SELECT email FROM users");

// Get a dictionary (id => row)
$usersById = $driver->queryDictionary("SELECT id, * FROM users");
```

### Augmented SQL Syntax

```php
// Conditional blocks - omitted when parameter is missing/null
$users = $driver->queryAll("
    SELECT * FROM users
    WHERE 1=1
    {{ AND status = $status }}
    {{ AND role = $role }}
", ['status' => 'active']); // role block is omitted

// Type-safe placeholders
$driver->queryAll("SELECT * FROM users WHERE age >= ?u", [18]); // unsigned int
```

### Fluent Query Builder

```php
$users = $driver->builder()
    ->select(['id', 'name', 'email'])
    ->from('users')
    ->where([['status', '=', 'active']])
    ->orderBy(['created_at' => 'DESC'])
    ->limit(10)
    ->queryAll();
```

### Transaction Support

```php
$driver->begin(function($driver) {
    $driver->execute("INSERT INTO orders ...", [...]);
    $driver->execute("UPDATE inventory ...", [...]);
    return true; // commit
});
```

## Requirements

- PHP 8.1 or higher
- Linux, macOS, or Windows
- One of: PostgreSQL, MySQL, or Microsoft SQL Server

## What's Next?

- [Installation](./getting-started/installation.md) - Get php-sqlx up and running
- [Quick Start](./getting-started/quick-start.md) - Your first queries
- [Connection Strings](./getting-started/connection-strings.md) - Configure your database connection
