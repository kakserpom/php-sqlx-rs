<?php

declare(strict_types=1);

/**
 * PHPUnit bootstrap file for php-sqlx integration tests.
 *
 * Ensures the sqlx extension is loaded before running tests.
 */

if (!extension_loaded('sqlx')) {
    echo "ERROR: The sqlx extension is not loaded.\n";
    echo "Build and install it first:\n";
    echo "  cd php-sqlx-cdylib && cargo build --release\n";
    echo "  cargo php install --release --yes\n";
    exit(1);
}

echo "sqlx extension loaded successfully.\n";
echo "PHP version: " . PHP_VERSION . "\n";
