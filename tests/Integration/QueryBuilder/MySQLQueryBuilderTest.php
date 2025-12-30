<?php

declare(strict_types=1);

namespace PhpSqlx\Tests\Integration\QueryBuilder;

/**
 * MySQL-specific query builder tests.
 */
class MySQLQueryBuilderTest extends AbstractQueryBuilderTest
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

    protected function createOrdersTable(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_orders');
        $this->driver->execute('
            CREATE TABLE test_orders (
                id INT AUTO_INCREMENT PRIMARY KEY,
                user_id INT NOT NULL,
                amount DECIMAL(10,2) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            ) ENGINE=InnoDB
        ');
    }

    // =========================================================================
    // MySQL-Specific Tests
    // =========================================================================

    public function testOnDuplicateKeyUpdate(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_unique');
        $this->driver->execute('
            CREATE TABLE test_unique (
                email VARCHAR(255) PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            ) ENGINE=InnoDB
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
                ->onDuplicateKeyUpdate(['name' => 'Alice Updated'])
                ->execute();

            $row = $this->driver->queryRow("SELECT * FROM test_unique WHERE email = 'alice@example.com'");
            $this->assertEquals('Alice Updated', $row->name);
        } finally {
            $this->driver->execute('DROP TABLE IF EXISTS test_unique');
        }
    }

    public function testReplaceInto(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_unique');
        $this->driver->execute('
            CREATE TABLE test_unique (
                email VARCHAR(255) PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            ) ENGINE=InnoDB
        ');

        try {
            // First insert
            $this->driver->builder()
                ->insertInto('test_unique')
                ->values(['email' => 'alice@example.com', 'name' => 'Alice'])
                ->execute();

            // Replace - should replace existing row
            $this->driver->builder()
                ->replaceInto('test_unique')
                ->values(['email' => 'alice@example.com', 'name' => 'Alice Replaced'])
                ->execute();

            $row = $this->driver->queryRow("SELECT * FROM test_unique WHERE email = 'alice@example.com'");
            $this->assertEquals('Alice Replaced', $row->name);

            // Verify only one row exists
            $count = $this->driver->queryValue('SELECT COUNT(*) FROM test_unique');
            $this->assertEquals(1, $count);
        } finally {
            $this->driver->execute('DROP TABLE IF EXISTS test_unique');
        }
    }

    public function testCrossJoin(): void
    {
        $this->createTestTable();
        $this->createOrdersTable();

        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");
        $this->driver->execute("INSERT INTO test_orders (user_id, amount) VALUES (1, 100)");
        $this->driver->execute("INSERT INTO test_orders (user_id, amount) VALUES (2, 200)");

        $results = $this->driver->builder()
            ->select('test_users.name, CAST(test_orders.amount AS SIGNED) AS amount')
            ->from('test_users')
            ->crossJoin('test_orders', '1=1')
            ->orderBy('test_users.name, test_orders.amount')
            ->queryAll();

        // Cross join produces 2 users x 2 orders = 4 rows
        $this->assertCount(4, $results);
    }
}
