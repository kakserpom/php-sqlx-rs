<?php

declare(strict_types=1);

namespace PhpSqlx\Tests\Integration\MySQL;

use PhpSqlx\Tests\Integration\AbstractDriverTest;

/**
 * MySQL-specific integration tests.
 */
class MySQLDriverTest extends AbstractDriverTest
{
    protected function getConnectionUrl(): string
    {
        return $_ENV['MYSQL_URL'] ?? '';
    }

    protected function createTestTable(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_users');
        $this->driver->execute('
            CREATE TABLE test_users (
                id INT AUTO_INCREMENT PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL,
                age INT DEFAULT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            ) ENGINE=InnoDB
        ');
    }

    // =========================================================================
    // MySQL-Specific Tests
    // =========================================================================

    public function testJsonType(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_json');
        $this->driver->execute('
            CREATE TABLE test_json (
                id INT AUTO_INCREMENT PRIMARY KEY,
                data JSON
            ) ENGINE=InnoDB
        ');

        try {
            $this->driver->execute('INSERT INTO test_json (data) VALUES (\'{"key": "value"}\')');

            $row = $this->driver->queryRow('SELECT * FROM test_json WHERE id = 1');
            $this->assertNotNull($row);
        } finally {
            $this->driver->execute('DROP TABLE IF EXISTS test_json');
        }
    }

    public function testEnumType(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_enum');
        $this->driver->execute("
            CREATE TABLE test_enum (
                id INT AUTO_INCREMENT PRIMARY KEY,
                status ENUM('pending', 'active', 'completed') NOT NULL
            ) ENGINE=InnoDB
        ");

        try {
            $this->driver->execute("INSERT INTO test_enum (status) VALUES ('active')");

            $row = $this->driver->queryRow('SELECT * FROM test_enum WHERE id = 1');
            $this->assertNotNull($row);
            $this->assertEquals('active', $row->status);
        } finally {
            $this->driver->execute('DROP TABLE IF EXISTS test_enum');
        }
    }

    public function testLastInsertId(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");

            $lastId = $this->driver->queryValue('SELECT LAST_INSERT_ID()');
            $this->assertGreaterThan(0, $lastId);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testOnDuplicateKey(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_upsert');
        $this->driver->execute('
            CREATE TABLE test_upsert (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                counter INT DEFAULT 1
            ) ENGINE=InnoDB
        ');

        try {
            $this->driver->execute("INSERT INTO test_upsert (id, name) VALUES (1, 'Alice')");
            $this->driver->execute("
                INSERT INTO test_upsert (id, name, counter)
                VALUES (1, 'Alice', 1)
                ON DUPLICATE KEY UPDATE counter = counter + 1
            ");

            $row = $this->driver->queryRow('SELECT * FROM test_upsert WHERE id = 1');
            $this->assertEquals(2, $row->counter);
        } finally {
            $this->driver->execute('DROP TABLE IF EXISTS test_upsert');
        }
    }

    public function testUtf8mb4(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_emoji');
        $this->driver->execute('
            CREATE TABLE test_emoji (
                id INT AUTO_INCREMENT PRIMARY KEY,
                content VARCHAR(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci
            ) ENGINE=InnoDB
        ');

        try {
            $emoji = '🎉🚀💻';
            $this->driver->execute(
                'INSERT INTO test_emoji (content) VALUES (?s)',
                [$emoji]
            );

            $row = $this->driver->queryRow('SELECT * FROM test_emoji WHERE id = 1');
            $this->assertEquals($emoji, $row->content);
        } finally {
            $this->driver->execute('DROP TABLE IF EXISTS test_emoji');
        }
    }
}
