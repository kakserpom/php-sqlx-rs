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
}
