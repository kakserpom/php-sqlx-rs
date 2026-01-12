# Connection Strings

php-sqlx uses URL-style connection strings to configure database connections.

## Basic Format

```
scheme://[user[:password]@]host[:port]/database[?options]
```

## PostgreSQL

**Scheme:** `postgres://`, `postgresql://`, or `pgsql://`

```php
// Basic connection
$driver = DriverFactory::make("postgres://localhost/mydb");

// With authentication
$driver = DriverFactory::make("postgres://user:password@localhost/mydb");

// With port
$driver = DriverFactory::make("postgres://user:password@localhost:5432/mydb");

// With SSL
$driver = DriverFactory::make("postgres://user:password@localhost/mydb?sslmode=require");
```

### PostgreSQL Options

| Option | Description | Example |
|--------|-------------|---------|
| `sslmode` | SSL mode: `disable`, `prefer`, `require` | `?sslmode=require` |
| `sslrootcert` | Path to CA certificate | `?sslrootcert=/path/to/ca.crt` |
| `application_name` | Application name for monitoring | `?application_name=myapp` |
| `options` | PostgreSQL options string | `?options=-c%20search_path%3Dmyschema` |

## MySQL

**Scheme:** `mysql://`

```php
// Basic connection
$driver = DriverFactory::make("mysql://localhost/mydb");

// With authentication
$driver = DriverFactory::make("mysql://root:password@localhost/mydb");

// With port
$driver = DriverFactory::make("mysql://root:password@localhost:3306/mydb");

// With SSL
$driver = DriverFactory::make("mysql://user:pass@localhost/mydb?ssl-mode=required");
```

### MySQL Options

| Option | Description | Example |
|--------|-------------|---------|
| `ssl-mode` | SSL mode: `disabled`, `preferred`, `required` | `?ssl-mode=required` |
| `ssl-ca` | Path to CA certificate | `?ssl-ca=/path/to/ca.pem` |
| `charset` | Connection charset | `?charset=utf8mb4` |

## Microsoft SQL Server

**Scheme:** `mssql://` or `sqlserver://`

```php
// Basic connection
$driver = DriverFactory::make("mssql://localhost/mydb");

// With authentication
$driver = DriverFactory::make("mssql://sa:Password123@localhost/mydb");

// With port
$driver = DriverFactory::make("mssql://sa:Password123@localhost:1433/mydb");

// Trust self-signed certificates
$driver = DriverFactory::make("mssql://sa:Password123@localhost/mydb?trust_server_certificate=true");
```

### MSSQL Options

| Option | Description | Example |
|--------|-------------|---------|
| `trust_server_certificate` | Trust self-signed certs | `?trust_server_certificate=true` |
| `encrypt` | Encryption mode: `strict`, `mandatory`, `optional` | `?encrypt=mandatory` |
| `instance` | Named instance | `?instance=SQLEXPRESS` |
| `app_name` | Application name | `?app_name=myapp` |
| `packet_size` | TDS packet size | `?packet_size=4096` |

## Special Characters

Special characters in usernames or passwords must be URL-encoded:

```php
// Password with @ symbol: "p@ssword" -> "p%40ssword"
$driver = DriverFactory::make("postgres://user:p%40ssword@localhost/mydb");

// Username with special chars
$driver = DriverFactory::make("mysql://admin%40company:password@localhost/mydb");
```

Common encodings:
- `@` → `%40`
- `:` → `%3A`
- `/` → `%2F`
- `#` → `%23`
- `?` → `%3F`

## Using Options Array

For complex configurations, use an options array instead:

```php
use Sqlx\DriverOptions;

$driver = DriverFactory::make([
    DriverOptions::OPT_URL => "postgres://localhost/mydb",
    DriverOptions::OPT_MAX_CONNECTIONS => 10,
    DriverOptions::OPT_ASSOC_ARRAYS => true,
    DriverOptions::OPT_IDLE_TIMEOUT => "5m",
]);
```

See [Driver Options](../configuration/driver-options.md) for all available options.

## Environment Variables

A common pattern is to use environment variables:

```php
$driver = DriverFactory::make(getenv('DATABASE_URL'));
```

```bash
export DATABASE_URL="postgres://user:pass@localhost/mydb"
```

## IPv6 Addresses

Wrap IPv6 addresses in brackets:

```php
$driver = DriverFactory::make("postgres://user:pass@[::1]:5432/mydb");
```

## Unix Sockets

### PostgreSQL

```php
$driver = DriverFactory::make("postgres://user@localhost/mydb?host=/var/run/postgresql");
```

### MySQL

```php
$driver = DriverFactory::make("mysql://user@localhost/mydb?socket=/var/run/mysqld/mysqld.sock");
```
