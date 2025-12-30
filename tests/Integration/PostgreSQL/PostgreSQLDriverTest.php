<?php

declare(strict_types=1);

namespace PhpSqlx\Tests\Integration\PostgreSQL;

use PhpSqlx\Tests\Integration\AbstractDriverTest;

/**
 * PostgreSQL-specific integration tests.
 */
class PostgreSQLDriverTest extends AbstractDriverTest
{
    protected function getConnectionUrl(): string
    {
        return $_ENV['POSTGRES_URL'] ?? '';
    }

    protected function createTestTable(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_users');
        $this->driver->execute('
            CREATE TABLE test_users (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL,
                age INTEGER DEFAULT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        ');
    }

    // =========================================================================
    // PostgreSQL-Specific Tests
    // =========================================================================

    public function testArrayType(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_arrays');
        $this->driver->execute('
            CREATE TABLE test_arrays (
                id SERIAL PRIMARY KEY,
                tags TEXT[]
            )
        ');

        try {
            $this->driver->execute("INSERT INTO test_arrays (tags) VALUES (ARRAY['php', 'rust'])");

            $row = $this->driver->queryRow('SELECT * FROM test_arrays WHERE id = 1');
            $this->assertNotNull($row);
        } finally {
            $this->driver->execute('DROP TABLE IF EXISTS test_arrays');
        }
    }

    public function testJsonType(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_json');
        $this->driver->execute('
            CREATE TABLE test_json (
                id SERIAL PRIMARY KEY,
                data JSONB
            )
        ');

        try {
            $this->driver->execute("INSERT INTO test_json (data) VALUES ('{\"key\": \"value\"}'::jsonb)");

            $row = $this->driver->queryRow('SELECT * FROM test_json WHERE id = 1');
            $this->assertNotNull($row);
        } finally {
            $this->driver->execute('DROP TABLE IF EXISTS test_json');
        }
    }

    public function testJsonArrayPlaceholder(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_json_array');
        $this->driver->execute('
            CREATE TABLE test_json_array (
                id SERIAL PRIMARY KEY,
                items JSONB[]
            )
        ');

        try {
            // Test with array of objects using ?ja placeholder
            $items = [
                ['name' => 'Item 1', 'price' => 10],
                ['name' => 'Item 2', 'price' => 20],
            ];
            $this->driver->execute('INSERT INTO test_json_array (items) VALUES (ARRAY[?ja]::jsonb[])', [$items]);

            // Read back the JSONB[] column and verify decoding works
            $row = $this->driver->queryRow('SELECT * FROM test_json_array WHERE id = 1');
            $this->assertNotNull($row);
            $this->assertCount(2, $row->items);
            $this->assertEquals('Item 1', $row->items[0]->name);
            $this->assertEquals(10, $row->items[0]->price);
            $this->assertEquals('Item 2', $row->items[1]->name);
            $this->assertEquals(20, $row->items[1]->price);
        } finally {
            $this->driver->execute('DROP TABLE IF EXISTS test_json_array');
        }
    }

    public function testUuidType(): void
    {
        $this->driver->execute('CREATE EXTENSION IF NOT EXISTS "uuid-ossp"');
        $this->driver->execute('DROP TABLE IF EXISTS test_uuid');
        $this->driver->execute('
            CREATE TABLE test_uuid (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                name VARCHAR(255)
            )
        ');

        try {
            $this->driver->execute("INSERT INTO test_uuid (name) VALUES ('test')");

            $row = $this->driver->queryRow('SELECT * FROM test_uuid LIMIT 1');
            $this->assertNotNull($row);
            $this->assertMatchesRegularExpression(
                '/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i',
                $row->id
            );
        } finally {
            $this->driver->execute('DROP TABLE IF EXISTS test_uuid');
        }
    }

    public function testReturningClause(): void
    {
        $this->createTestTable();

        try {
            $result = $this->driver->queryRow(
                "INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com') RETURNING id, name"
            );

            $this->assertNotNull($result);
            $this->assertGreaterThan(0, $result->id);
            $this->assertEquals('Alice', $result->name);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testCte(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Alice', 'alice@example.com', 30)");
            $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Bob', 'bob@example.com', 25)");

            $users = $this->driver->queryAll('
                WITH young_users AS (
                    SELECT * FROM test_users WHERE age < 30
                )
                SELECT * FROM young_users
            ');

            $this->assertCount(1, $users);
            $this->assertEquals('Bob', $users[0]->name);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testUpsert(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_upsert');
        $this->driver->execute('
            CREATE TABLE test_upsert (
                email VARCHAR(255) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                login_count INTEGER DEFAULT 1
            )
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
            $affected = $this->driver->upsert(
                'test_upsert',
                ['email' => 'alice@example.com', 'name' => 'Alice Updated', 'login_count' => 2],
                ['email'],
                ['name', 'login_count']
            );
            $this->assertEquals(1, $affected);

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

    public function testUpsertDoNothing(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_upsert_nothing');
        $this->driver->execute('
            CREATE TABLE test_upsert_nothing (
                id INTEGER PRIMARY KEY
            )
        ');

        try {
            // First insert
            $this->driver->upsert('test_upsert_nothing', ['id' => 1], ['id']);

            // Second upsert with same key - should do nothing (ON CONFLICT DO NOTHING)
            $affected = $this->driver->upsert('test_upsert_nothing', ['id' => 1], ['id']);
            $this->assertEquals(0, $affected);

            // Verify only one row exists
            $count = $this->driver->queryValue('SELECT COUNT(*) FROM test_upsert_nothing');
            $this->assertEquals(1, $count);
        } finally {
            $this->driver->execute('DROP TABLE IF EXISTS test_upsert_nothing');
        }
    }

    public function testInsertMany(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_insert_many');
        $this->driver->execute('
            CREATE TABLE test_insert_many (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL
            )
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
