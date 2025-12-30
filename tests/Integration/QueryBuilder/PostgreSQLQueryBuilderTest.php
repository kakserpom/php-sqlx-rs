<?php

declare(strict_types=1);

namespace PhpSqlx\Tests\Integration\QueryBuilder;

/**
 * PostgreSQL-specific query builder tests.
 */
class PostgreSQLQueryBuilderTest extends AbstractQueryBuilderTest
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

    protected function createOrdersTable(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_orders');
        $this->driver->execute('
            CREATE TABLE test_orders (
                id SERIAL PRIMARY KEY,
                user_id INTEGER NOT NULL,
                amount DECIMAL(10,2) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        ');
    }

    // =========================================================================
    // PostgreSQL-Specific Tests
    // =========================================================================

    public function testOperatorILike(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['name', 'ILIKE', 'alice']])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Alice', $users[0]->name);
    }

    public function testOperatorIContains(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['name', 'ICONTAINS', 'LIC']])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Alice', $users[0]->name);
    }

    public function testOperatorNotIContains(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['name', 'NOT ICONTAINS', 'LIC']])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Bob', $users[0]->name);
    }

    public function testOperatorIStartsWith(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['name', 'ISTARTSWITH', 'ali']])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Alice', $users[0]->name);
    }

    public function testOperatorIEndsWith(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['name', 'IENDSWITH', 'ICE']])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Alice', $users[0]->name);
    }

    public function testOperatorNotIEndsWith(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['name', 'NOT IENDSWITH', 'ICE']])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Bob', $users[0]->name);
    }

    public function testOperatorIEq(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['name', 'IEQ', 'ALICE']])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Alice', $users[0]->name);
    }

    public function testReturning(): void
    {
        $this->createTestTable();

        $result = $this->driver->builder()
            ->insertInto('test_users')
            ->values(['name' => 'Alice', 'email' => 'alice@example.com'])
            ->returning('id, name')
            ->queryRow();

        $this->assertNotNull($result);
        $this->assertGreaterThan(0, $result->id);
        $this->assertEquals('Alice', $result->name);
    }

    public function testReturningArray(): void
    {
        $this->createTestTable();

        $result = $this->driver->builder()
            ->insertInto('test_users')
            ->values(['name' => 'Alice', 'email' => 'alice@example.com'])
            ->returning(['id', 'name'])
            ->queryRow();

        $this->assertNotNull($result);
        $this->assertEquals('Alice', $result->name);
    }

    public function testOnConflictDoNothing(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_unique');
        $this->driver->execute('
            CREATE TABLE test_unique (
                email VARCHAR(255) PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )
        ');

        try {
            // First insert
            $this->driver->builder()
                ->insertInto('test_unique')
                ->values(['email' => 'alice@example.com', 'name' => 'Alice'])
                ->execute();

            // Second insert with conflict - should do nothing
            $this->driver->builder()
                ->insertInto('test_unique')
                ->values(['email' => 'alice@example.com', 'name' => 'Alice Updated'])
                ->onConflict('email', null)
                ->execute();

            $row = $this->driver->queryRow("SELECT * FROM test_unique WHERE email = 'alice@example.com'");
            $this->assertEquals('Alice', $row->name); // Not updated
        } finally {
            $this->driver->execute('DROP TABLE IF EXISTS test_unique');
        }
    }

    public function testOnConflictDoUpdate(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_unique');
        $this->driver->execute('
            CREATE TABLE test_unique (
                email VARCHAR(255) PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )
        ');

        try {
            // First insert
            $this->driver->builder()
                ->insertInto('test_unique')
                ->values(['email' => 'alice@example.com', 'name' => 'Alice'])
                ->execute();

            // Second insert with conflict - should update
            $this->driver->builder()
                ->insertInto('test_unique')
                ->values(['email' => 'alice@example.com', 'name' => 'Alice Updated'])
                ->onConflict('email', ['name' => 'Alice Updated'])
                ->execute();

            $row = $this->driver->queryRow("SELECT * FROM test_unique WHERE email = 'alice@example.com'");
            $this->assertEquals('Alice Updated', $row->name);
        } finally {
            $this->driver->execute('DROP TABLE IF EXISTS test_unique');
        }
    }

    public function testCte(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Alice', 'alice@example.com', 30)");
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Bob', 'bob@example.com', 25)");

        $users = $this->driver->builder()
            ->with('young_users', "SELECT * FROM test_users WHERE age < :max_age", ['max_age' => 30])
            ->select('*')
            ->from('young_users')
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Bob', $users[0]->name);
    }

    public function testFullOuterJoin(): void
    {
        $this->createTestTable();
        $this->createOrdersTable();

        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");
        $this->driver->execute("INSERT INTO test_orders (user_id, amount) VALUES (1, 100)");
        $this->driver->execute("INSERT INTO test_orders (user_id, amount) VALUES (999, 200)"); // Non-existent user

        $results = $this->driver->builder()
            ->select('test_users.name, CAST(test_orders.amount AS INTEGER) AS amount')
            ->from('test_users')
            ->fullJoin('test_orders', 'test_users.id = test_orders.user_id')
            ->orderBy('test_orders.amount')
            ->queryAll();

        // Should include: Alice with 100, Bob with null, null with 200
        $this->assertCount(3, $results);
    }
}
