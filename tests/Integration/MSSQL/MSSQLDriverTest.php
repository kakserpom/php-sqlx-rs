<?php

declare(strict_types=1);

namespace PhpSqlx\Tests\Integration\MSSQL;

use PhpSqlx\Tests\Integration\AbstractDriverTest;

/**
 * MSSQL-specific integration tests.
 */
class MSSQLDriverTest extends AbstractDriverTest
{
    protected function getConnectionUrl(): string
    {
        return $_ENV['MSSQL_URL'] ?? '';
    }

    protected function createTestTable(): void
    {
        $this->driver->execute("
            IF OBJECT_ID('test_users', 'U') IS NOT NULL DROP TABLE test_users
        ");
        $this->driver->execute('
            CREATE TABLE test_users (
                id INT IDENTITY(1,1) PRIMARY KEY,
                name NVARCHAR(255) NOT NULL,
                email NVARCHAR(255) NOT NULL,
                age INT NULL,
                created_at DATETIME2 DEFAULT GETDATE()
            )
        ');
    }

    protected function dropTestTable(): void
    {
        try {
            $this->driver->execute("
                IF OBJECT_ID('test_users', 'U') IS NOT NULL DROP TABLE test_users
            ");
        } catch (\Exception $e) {
            // Ignore errors during cleanup
        }
    }

    // =========================================================================
    // MSSQL-Specific Tests
    // =========================================================================

    public function testOutputClause(): void
    {
        $this->createTestTable();

        try {
            $result = $this->driver->queryRow("
                INSERT INTO test_users (name, email)
                OUTPUT INSERTED.id, INSERTED.name
                VALUES ('Alice', 'alice@example.com')
            ");

            $this->assertNotNull($result);
            $this->assertGreaterThan(0, $result->id);
            $this->assertEquals('Alice', $result->name);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testTopClause(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Charlie', 'charlie@example.com')");

            $users = $this->driver->queryAll('SELECT TOP 2 * FROM test_users ORDER BY name');
            $this->assertCount(2, $users);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testNvarcharType(): void
    {
        $this->createTestTable();

        try {
            $unicode = '日本語テスト';
            $this->driver->execute(
                "INSERT INTO test_users (name, email) VALUES (?s, 'test@example.com')",
                [$unicode]
            );

            $user = $this->driver->queryRow('SELECT * FROM test_users WHERE id = 1');
            $this->assertEquals($unicode, $user->name);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testMergeStatement(): void
    {
        $this->driver->execute("
            IF OBJECT_ID('test_upsert', 'U') IS NOT NULL DROP TABLE test_upsert
        ");
        $this->driver->execute('
            CREATE TABLE test_upsert (
                id INT PRIMARY KEY,
                name NVARCHAR(255) NOT NULL,
                counter INT DEFAULT 1
            )
        ');

        try {
            $this->driver->execute("INSERT INTO test_upsert (id, name) VALUES (1, 'Alice')");
            $this->driver->execute("
                MERGE test_upsert AS target
                USING (SELECT 1 AS id, 'Alice' AS name) AS source
                ON target.id = source.id
                WHEN MATCHED THEN UPDATE SET counter = counter + 1
                WHEN NOT MATCHED THEN INSERT (id, name) VALUES (source.id, source.name);
            ");

            $row = $this->driver->queryRow('SELECT * FROM test_upsert WHERE id = 1');
            $this->assertEquals(2, $row->counter);
        } finally {
            $this->driver->execute("
                IF OBJECT_ID('test_upsert', 'U') IS NOT NULL DROP TABLE test_upsert
            ");
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
