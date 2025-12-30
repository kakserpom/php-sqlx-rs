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

    public function testJsonPlaceholder(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_json');
        $this->driver->execute('
            CREATE TABLE test_json (
                id INT AUTO_INCREMENT PRIMARY KEY,
                data JSON
            ) ENGINE=InnoDB
        ');

        try {
            // Test with associative array (object)
            $data = ['name' => 'Alice', 'age' => 30, 'active' => true];
            $this->driver->execute('INSERT INTO test_json (data) VALUES (?j)', [$data]);

            $row = $this->driver->queryRow('SELECT * FROM test_json WHERE id = 1');
            $this->assertNotNull($row);
            // JSON columns are returned as decoded objects/arrays
            $this->assertEquals('Alice', $row->data->name);
            $this->assertEquals(30, $row->data->age);
            $this->assertTrue($row->data->active);

            // Test with sequential array
            $arrayData = [1, 2, 3, 'four'];
            $this->driver->execute('INSERT INTO test_json (data) VALUES (?j)', [$arrayData]);

            $row = $this->driver->queryRow('SELECT * FROM test_json WHERE id = 2');
            $this->assertEquals([1, 2, 3, 'four'], $row->data);

            // Test with Sqlx\JSON() wrapper and ?j placeholder
            $wrappedData = \Sqlx\JSON(['wrapped' => true, 'value' => 42]);
            $this->driver->execute('INSERT INTO test_json (data) VALUES (?j)', [$wrappedData]);

            $row = $this->driver->queryRow('SELECT * FROM test_json WHERE id = 3');
            $this->assertTrue($row->data->wrapped);
            $this->assertEquals(42, $row->data->value);

            // Test Sqlx\JSON() without type hint (should also work)
            $this->driver->execute('INSERT INTO test_json (data) VALUES (?)', [\Sqlx\JSON(['no_hint' => 'works'])]);

            $row = $this->driver->queryRow('SELECT * FROM test_json WHERE id = 4');
            $this->assertEquals('works', $row->data->no_hint);
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
            // LAST_INSERT_ID() requires a pinned connection to ensure the INSERT and SELECT
            // run on the same connection (connection pooling uses different connections otherwise)
            $lastId = $this->driver->withConnection(function ($driver) {
                $driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
                return $driver->queryValue('SELECT LAST_INSERT_ID()');
            });

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

    public function testUpsert(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_upsert');
        $this->driver->execute('
            CREATE TABLE test_upsert (
                email VARCHAR(255) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                login_count INT DEFAULT 1
            ) ENGINE=InnoDB
        ');

        try {
            // First insert - should create new row
            $affected = $this->driver->upsert(
                'test_upsert',
                ['email' => 'alice@example.com', 'name' => 'Alice', 'login_count' => 1],
                ['email'],
                ['name', 'login_count']
            );
            $this->assertEquals(1, $affected);

            $row = $this->driver->queryRow("SELECT * FROM test_upsert WHERE email = 'alice@example.com'");
            $this->assertEquals('Alice', $row->name);
            $this->assertEquals(1, $row->login_count);

            // Second upsert - should update existing row
            // MySQL returns 2 for ON DUPLICATE KEY UPDATE when row is updated
            $affected = $this->driver->upsert(
                'test_upsert',
                ['email' => 'alice@example.com', 'name' => 'Alice Updated', 'login_count' => 2],
                ['email'],
                ['name', 'login_count']
            );
            $this->assertGreaterThanOrEqual(1, $affected);

            $row = $this->driver->queryRow("SELECT * FROM test_upsert WHERE email = 'alice@example.com'");
            $this->assertEquals('Alice Updated', $row->name);
            $this->assertEquals(2, $row->login_count);

            // Test upsert with auto-determined update columns
            $this->driver->upsert(
                'test_upsert',
                ['email' => 'bob@example.com', 'name' => 'Bob', 'login_count' => 5],
                ['email']
            );

            $row = $this->driver->queryRow("SELECT * FROM test_upsert WHERE email = 'bob@example.com'");
            $this->assertEquals('Bob', $row->name);
            $this->assertEquals(5, $row->login_count);
        } finally {
            $this->driver->execute('DROP TABLE IF EXISTS test_upsert');
        }
    }

    public function testUpsertInsertIgnore(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_upsert_ignore');
        $this->driver->execute('
            CREATE TABLE test_upsert_ignore (
                id INT PRIMARY KEY
            ) ENGINE=InnoDB
        ');

        try {
            // First insert
            $this->driver->upsert('test_upsert_ignore', ['id' => 1], ['id']);

            // Second upsert with same key - should use INSERT IGNORE
            $affected = $this->driver->upsert('test_upsert_ignore', ['id' => 1], ['id']);
            $this->assertEquals(0, $affected);

            // Verify only one row exists
            $count = $this->driver->queryValue('SELECT COUNT(*) FROM test_upsert_ignore');
            $this->assertEquals(1, $count);
        } finally {
            $this->driver->execute('DROP TABLE IF EXISTS test_upsert_ignore');
        }
    }

    public function testInsertMany(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_insert_many');
        $this->driver->execute('
            CREATE TABLE test_insert_many (
                id INT AUTO_INCREMENT PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL
            ) ENGINE=InnoDB
        ');

        try {
            $affected = $this->driver->insertMany('test_insert_many', [
                ['name' => 'Alice', 'email' => 'alice@example.com'],
                ['name' => 'Bob', 'email' => 'bob@example.com'],
                ['name' => 'Carol', 'email' => 'carol@example.com'],
            ]);

            $this->assertEquals(3, $affected);

            $rows = $this->driver->queryAll('SELECT name, email FROM test_insert_many ORDER BY name');
            $this->assertCount(3, $rows);
            $this->assertEquals('Alice', $rows[0]->name);
            $this->assertEquals('Bob', $rows[1]->name);
            $this->assertEquals('Carol', $rows[2]->name);
        } finally {
            $this->driver->execute('DROP TABLE IF EXISTS test_insert_many');
        }
    }
}
