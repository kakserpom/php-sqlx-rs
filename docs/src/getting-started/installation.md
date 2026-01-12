# Installation

php-sqlx is distributed as a PHP extension. You can install it from pre-built binaries or compile from source.

## Pre-built Binaries

### Linux (Ubuntu/Debian)

```bash
# Download the latest release
curl -LO https://github.com/kakserpom/php-sqlx-rs/releases/latest/download/sqlx-linux-x64.so

# Move to PHP extension directory
sudo mv sqlx-linux-x64.so $(php -r "echo ini_get('extension_dir');")/sqlx.so

# Enable the extension
echo "extension=sqlx.so" | sudo tee /etc/php/$(php -r 'echo PHP_MAJOR_VERSION.".".PHP_MINOR_VERSION;')/mods-available/sqlx.ini
sudo phpenmod sqlx
```

### macOS

```bash
# Download the latest release
curl -LO https://github.com/kakserpom/php-sqlx-rs/releases/latest/download/sqlx-macos-arm64.dylib

# Move to PHP extension directory
sudo mv sqlx-macos-arm64.dylib $(php -r "echo ini_get('extension_dir');")/sqlx.so

# Enable the extension
echo "extension=sqlx.so" >> $(php --ini | grep "Loaded Configuration" | cut -d: -f2 | xargs)
```

### Windows

1. Download `sqlx-windows-x64.dll` from the releases page
2. Copy to your PHP `ext` directory
3. Add `extension=sqlx` to your `php.ini`

## Building from Source

### Prerequisites

- Rust 1.70 or higher
- PHP 8.1+ development headers
- Clang/LLVM

### Build Steps

```bash
# Clone the repository
git clone https://github.com/kakserpom/php-sqlx-rs.git
cd php-sqlx-rs

# Build the extension
cd php-sqlx-cdylib
cargo build --release

# Install
sudo cp target/release/libphp_sqlx_cdylib.so $(php -r "echo ini_get('extension_dir');")/sqlx.so
echo "extension=sqlx.so" | sudo tee /etc/php/conf.d/99-sqlx.ini
```

## Verifying Installation

```bash
# Check if extension is loaded
php -m | grep sqlx

# Should output:
# sqlx
```

```php
<?php
// Test the extension
$driver = Sqlx\DriverFactory::make("postgres://localhost/test");
echo "php-sqlx is working!\n";
```

## Docker

```dockerfile
FROM php:8.3-cli

# Install build dependencies
RUN apt-get update && apt-get install -y \
    curl \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Build php-sqlx
COPY . /app
WORKDIR /app/php-sqlx-cdylib
RUN cargo build --release

# Install extension
RUN cp target/release/libphp_sqlx_cdylib.so $(php -r "echo ini_get('extension_dir');")/sqlx.so \
    && echo "extension=sqlx.so" > /usr/local/etc/php/conf.d/sqlx.ini
```

## Troubleshooting

### Extension not loading

Check that the extension file exists and has correct permissions:

```bash
ls -la $(php -r "echo ini_get('extension_dir');")/sqlx.so
```

### Symbol errors on load

Make sure you're using a compatible PHP version. The extension must be compiled for your specific PHP version.

### Connection errors

Verify your database server is running and accessible:

```bash
# PostgreSQL
pg_isready -h localhost -p 5432

# MySQL
mysqladmin ping -h localhost

# MSSQL
sqlcmd -S localhost -U sa -P password -Q "SELECT 1"
```
