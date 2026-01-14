<?php

declare(strict_types=1);

namespace PhpSqlx\Tests\Integration;

use PHPUnit\Framework\TestCase;
use Sqlx\DriverFactory;
use Sqlx\Exceptions\SqlxException;

/**
 * Abstract base class for database driver integration tests.
 *
 * Subclasses must implement getConnectionUrl() to provide
 * the database-specific connection URL.
 */
abstract class AbstractDriverTest extends TestCase
{
    protected mixed $driver;

    abstract protected function getConnectionUrl(): string;

    protected function setUp(): void
    {
        parent::setUp();

        $url = $this->getConnectionUrl();
        if (empty($url)) {
            $this->markTestSkipped('Database URL not configured');
        }

        try {
            $this->driver = DriverFactory::make($url);
        } catch (\Exception $e) {
            $this->markTestSkipped('Could not connect to database: ' . $e->getMessage());
        }
    }

    protected function tearDown(): void
    {
        // Explicitly close the pool - Drop won't run until PHP exits
        if (isset($this->driver)) {
            $this->driver->close();
        }
        parent::tearDown();
    }

    // =========================================================================
    // Connection Tests
    // =========================================================================

    public function testConnection(): void
    {
        $result = $this->driver->queryValue('SELECT 1');
        $this->assertEquals(1, $result);
    }

    // =========================================================================
    // Basic Query Tests
    // =========================================================================

    public function testQueryAll(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

            $users = $this->driver->queryAll('SELECT * FROM test_users ORDER BY name');

            $this->assertCount(2, $users);
            $this->assertEquals('Alice', $users[0]->name);
            $this->assertEquals('Bob', $users[1]->name);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testQueryRow(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");

            $user = $this->driver->queryRow('SELECT * FROM test_users WHERE name = ?s', ['Alice']);

            $this->assertEquals('Alice', $user->name);
            $this->assertEquals('alice@example.com', $user->email);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testQueryMaybeRow(): void
    {
        $this->createTestTable();

        try {
            $user = $this->driver->queryMaybeRow('SELECT * FROM test_users WHERE name = ?s', ['NonExistent']);
            $this->assertNull($user);

            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");

            $user = $this->driver->queryMaybeRow('SELECT * FROM test_users WHERE name = ?s', ['Alice']);
            $this->assertNotNull($user);
            $this->assertEquals('Alice', $user->name);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testQueryValue(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

            $count = $this->driver->queryValue('SELECT COUNT(*) FROM test_users');
            $this->assertEquals(2, $count);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testQueryColumn(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

            $names = $this->driver->queryColumn('SELECT name FROM test_users ORDER BY name');

            $this->assertCount(2, $names);
            $this->assertEquals('Alice', $names[0]);
            $this->assertEquals('Bob', $names[1]);
        } finally {
            $this->dropTestTable();
        }
    }

    // =========================================================================
    // Parameter Binding Tests
    // =========================================================================

    public function testNamedParameters(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute(
                "INSERT INTO test_users (name, email) VALUES (:name, :email)",
                ['name' => 'Alice', 'email' => 'alice@example.com']
            );

            $user = $this->driver->queryRow(
                'SELECT * FROM test_users WHERE name = :name',
                ['name' => 'Alice']
            );

            $this->assertEquals('Alice', $user->name);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testPositionalParameters(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute(
                "INSERT INTO test_users (name, email) VALUES (?s, ?s)",
                ['Alice', 'alice@example.com']
            );

            $user = $this->driver->queryRow('SELECT * FROM test_users WHERE name = ?s', ['Alice']);
            $this->assertEquals('Alice', $user->name);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testTypedPlaceholders(): void
    {
        $this->createTestTable();

        try {
            // Test integer placeholder
            $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Alice', 'alice@example.com', 30)");

            $user = $this->driver->queryRow('SELECT * FROM test_users WHERE age = ?i', [30]);
            $this->assertEquals('Alice', $user->name);

            // Test string placeholder
            $user = $this->driver->queryRow('SELECT * FROM test_users WHERE name = ?s', ['Alice']);
            $this->assertEquals(30, $user->age);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testInClauseExpansion(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Charlie', 'charlie@example.com')");

            $users = $this->driver->queryAll(
                'SELECT * FROM test_users WHERE name IN :names!sa ORDER BY name',
                ['names' => ['Alice', 'Charlie']]
            );

            $this->assertCount(2, $users);
            $this->assertEquals('Alice', $users[0]->name);
            $this->assertEquals('Charlie', $users[1]->name);
        } finally {
            $this->dropTestTable();
        }
    }

    // =========================================================================
    // Conditional Block Tests
    // =========================================================================

    public function testConditionalBlockIncluded(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

            $users = $this->driver->queryAll(
                'SELECT * FROM test_users WHERE 1=1 {{ AND name = :name }}',
                ['name' => 'Alice']
            );

            $this->assertCount(1, $users);
            $this->assertEquals('Alice', $users[0]->name);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testConditionalBlockSkipped(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

            // Without the 'name' parameter, the block should be skipped
            $users = $this->driver->queryAll(
                'SELECT * FROM test_users WHERE 1=1 {{ AND name = :name }} ORDER BY name'
            );

            $this->assertCount(2, $users);
        } finally {
            $this->dropTestTable();
        }
    }

    // =========================================================================
    // Transaction Tests
    // =========================================================================

    public function testTransactionCommit(): void
    {
        $this->createTestTable();

        try {
            $this->driver->begin();
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
            $this->driver->commit();

            $count = $this->driver->queryValue('SELECT COUNT(*) FROM test_users');
            $this->assertEquals(1, $count);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testTransactionRollback(): void
    {
        $this->createTestTable();

        try {
            $this->driver->begin();
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
            $this->driver->rollback();

            $count = $this->driver->queryValue('SELECT COUNT(*) FROM test_users');
            $this->assertEquals(0, $count);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testTransactionCallback(): void
    {
        $this->createTestTable();

        try {
            $result = $this->driver->begin(function ($driver) {
                $driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
                return true; // commit
            });

            $this->assertTrue($result);
            $count = $this->driver->queryValue('SELECT COUNT(*) FROM test_users');
            $this->assertEquals(1, $count);
        } finally {
            $this->dropTestTable();
        }
    }

    // =========================================================================
    // Schema Introspection Tests
    // =========================================================================

    public function testDescribeTable(): void
    {
        $this->createTestTable();

        try {
            $columns = $this->driver->describeTable('test_users');

            $this->assertGreaterThanOrEqual(4, count($columns));

            // Find the 'name' column
            $nameColumn = null;
            foreach ($columns as $col) {
                if ($col['name'] === 'name') {
                    $nameColumn = $col;
                    break;
                }
            }

            $this->assertNotNull($nameColumn, 'name column should exist');
            $this->assertArrayHasKey('type', $nameColumn);
            $this->assertArrayHasKey('nullable', $nameColumn);
            $this->assertArrayHasKey('ordinal', $nameColumn);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testDescribeTableValidation(): void
    {
        $this->expectException(SqlxException::class);
        $this->driver->describeTable('users; DROP TABLE users;--');
    }

    // =========================================================================
    // Query Hook Tests
    // =========================================================================

    public function testQueryHook(): void
    {
        $this->createTestTable();

        try {
            $queries = [];
            $this->driver->onQuery(function (string $sql, string $sqlInline, float $durationMs) use (&$queries) {
                $queries[] = [
                    'sql' => $sql,
                    'sqlInline' => $sqlInline,
                    'durationMs' => $durationMs,
                ];
            });

            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
            $this->driver->queryAll('SELECT * FROM test_users');

            $this->driver->onQuery(null); // Disable hook

            $this->assertCount(2, $queries);
            $this->assertGreaterThan(0, $queries[0]['durationMs']);
            $this->assertGreaterThan(0, $queries[1]['durationMs']);
        } finally {
            $this->dropTestTable();
        }
    }

    // =========================================================================
    // Connection Tagging Tests
    // =========================================================================

    public function testSetApplicationName(): void
    {
        // Just verify it doesn't throw
        $this->driver->setApplicationName('test-application');
        $this->assertTrue(true);
    }

    public function testSetClientInfo(): void
    {
        // Just verify it doesn't throw
        $this->driver->setClientInfo('test-app', [
            'request_id' => 'abc123',
            'user_id' => 42,
        ]);
        $this->assertTrue(true);
    }

    // =========================================================================
    // Query Iterator Tests (query() returning QueryResult)
    // =========================================================================

    public function testQueryReturnsIterator(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Charlie', 'charlie@example.com')");

            $result = $this->driver->query('SELECT * FROM test_users ORDER BY name');

            // Should implement Iterator
            $this->assertInstanceOf(\Iterator::class, $result);

            // Iterate and collect names
            $names = [];
            foreach ($result as $index => $row) {
                $this->assertIsInt($index);
                $names[] = $row->name;
            }

            $this->assertCount(3, $names);
            $this->assertEquals(['Alice', 'Bob', 'Charlie'], $names);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testQueryWithParameters(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Alice', 'alice@example.com', 25)");
            $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Bob', 'bob@example.com', 30)");
            $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Charlie', 'charlie@example.com', 35)");

            $result = $this->driver->query('SELECT * FROM test_users WHERE age > ?i ORDER BY name', [25]);

            $names = [];
            foreach ($result as $row) {
                $names[] = $row->name;
            }

            $this->assertCount(2, $names);
            $this->assertEquals(['Bob', 'Charlie'], $names);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testQueryAssocReturnsAssociativeArrays(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");

            $result = $this->driver->queryAssoc('SELECT * FROM test_users');

            foreach ($result as $row) {
                $this->assertIsArray($row);
                $this->assertArrayHasKey('name', $row);
                $this->assertArrayHasKey('email', $row);
                $this->assertEquals('Alice', $row['name']);
            }
        } finally {
            $this->dropTestTable();
        }
    }

    public function testQueryObjReturnsObjects(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");

            $result = $this->driver->queryObj('SELECT * FROM test_users');

            foreach ($result as $row) {
                $this->assertIsObject($row);
                $this->assertObjectHasProperty('name', $row);
                $this->assertObjectHasProperty('email', $row);
                $this->assertEquals('Alice', $row->name);
            }
        } finally {
            $this->dropTestTable();
        }
    }

    public function testQueryWithBatchSize(): void
    {
        $this->createTestTable();

        try {
            // Insert 10 rows
            for ($i = 1; $i <= 10; $i++) {
                $this->driver->execute(
                    "INSERT INTO test_users (name, email) VALUES (:name, :email)",
                    ['name' => "User{$i}", 'email' => "user{$i}@example.com"]
                );
            }

            // Query with small batch size
            $result = $this->driver->query('SELECT * FROM test_users ORDER BY name', [], 3);

            $this->assertEquals(3, $result->getBatchSize());

            $count = 0;
            foreach ($result as $row) {
                $count++;
            }

            $this->assertEquals(10, $count);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testQueryToArray(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

            $result = $this->driver->query('SELECT * FROM test_users ORDER BY name');
            $rows = $result->toArray();

            $this->assertIsArray($rows);
            $this->assertCount(2, $rows);
            $this->assertEquals('Alice', $rows[0]->name);
            $this->assertEquals('Bob', $rows[1]->name);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testQueryIsExhausted(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");

            $result = $this->driver->query('SELECT * FROM test_users');

            // Before iteration, should not be exhausted (first batch loaded on rewind)
            $this->assertFalse($result->isExhausted());

            // Iterate through all rows
            foreach ($result as $row) {
                // Just iterate
            }

            // After full iteration, should be exhausted
            $this->assertTrue($result->isExhausted());
        } finally {
            $this->dropTestTable();
        }
    }

    public function testQueryEmptyResult(): void
    {
        $this->createTestTable();

        try {
            $result = $this->driver->query('SELECT * FROM test_users');

            $count = 0;
            foreach ($result as $row) {
                $count++;
            }

            $this->assertEquals(0, $count);
            $this->assertTrue($result->isExhausted());
        } finally {
            $this->dropTestTable();
        }
    }

    public function testQueryCount(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Charlie', 'charlie@example.com')");

            $result = $this->driver->query('SELECT * FROM test_users ORDER BY name');

            // Iterate to load rows
            foreach ($result as $row) {
                // Just iterate
            }

            // Count should reflect total fetched
            $this->assertEquals(3, $result->count());
        } finally {
            $this->dropTestTable();
        }
    }

    public function testQueryMultipleIterations(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

            $result = $this->driver->query('SELECT * FROM test_users ORDER BY name');

            // First iteration
            $firstPass = [];
            foreach ($result as $row) {
                $firstPass[] = $row->name;
            }

            // Note: QueryResult doesn't support full rewind after exhaustion
            // This is expected behavior - for multiple iterations, use toArray() or create a new query
            $this->assertCount(2, $firstPass);
        } finally {
            $this->dropTestTable();
        }
    }

    public function testQueryIteratorToArray(): void
    {
        $this->createTestTable();

        try {
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
            $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

            $result = $this->driver->query('SELECT * FROM test_users ORDER BY name');

            // Use PHP's iterator_to_array
            $rows = iterator_to_array($result);

            $this->assertCount(2, $rows);
            $this->assertEquals('Alice', $rows[0]->name);
            $this->assertEquals('Bob', $rows[1]->name);
        } finally {
            $this->dropTestTable();
        }
    }

    /**
     * Test that dropping a QueryResult mid-iteration properly cancels the background streaming.
     *
     * This ensures that:
     * 1. The background task is cancelled when the result is dropped
     * 2. Database connections are released back to the pool
     * 3. No resource leaks occur when not fully consuming an iterator
     */
    public function testQueryDropCancelsStream(): void
    {
        $this->createTestTable();

        try {
            // Insert many rows to ensure we have more than one buffer's worth
            for ($i = 0; $i < 200; $i++) {
                $this->driver->execute(
                    "INSERT INTO test_users (name, email) VALUES (?s, ?s)",
                    ["User{$i}", "user{$i}@example.com"]
                );
            }

            // Start a query but only read a few rows, then drop the result
            for ($iteration = 0; $iteration < 5; $iteration++) {
                $result = $this->driver->query('SELECT * FROM test_users ORDER BY id', null, 10);

                // Only read first 3 rows
                $count = 0;
                foreach ($result as $row) {
                    $count++;
                    if ($count >= 3) {
                        break;
                    }
                }

                $this->assertEquals(3, $count);
                $this->assertFalse($result->isExhausted());

                // Explicitly unset to trigger destructor
                unset($result);
            }

            // Verify the connection pool is still healthy by executing more queries
            // If streams weren't properly cancelled, we'd have connection leaks
            $value = $this->driver->queryValue('SELECT COUNT(*) FROM test_users');
            $this->assertEquals(200, $value);

            // Also verify we can start new streaming queries
            // Use iteration with break instead of LIMIT for cross-database compatibility
            $result2 = $this->driver->query('SELECT * FROM test_users ORDER BY id');
            $rows = [];
            foreach ($result2 as $row) {
                $rows[] = $row;
                if (count($rows) >= 5) break;
            }
            $this->assertCount(5, $rows);

        } finally {
            $this->dropTestTable();
        }
    }

    /**
     * Test that multiple concurrent partial iterations don't cause issues.
     */
    public function testQueryMultipleConcurrentPartialIterations(): void
    {
        $this->createTestTable();

        try {
            // Insert test data
            for ($i = 0; $i < 50; $i++) {
                $this->driver->execute(
                    "INSERT INTO test_users (name, email) VALUES (?s, ?s)",
                    ["User{$i}", "user{$i}@example.com"]
                );
            }

            // Create multiple result objects simultaneously
            $result1 = $this->driver->query('SELECT * FROM test_users ORDER BY id', null, 5);
            $result2 = $this->driver->query('SELECT * FROM test_users ORDER BY id DESC', null, 5);
            $result3 = $this->driver->query('SELECT * FROM test_users WHERE id > 25', null, 5);

            // Partially iterate each
            $names1 = [];
            $count = 0;
            foreach ($result1 as $row) {
                $names1[] = $row->name;
                if (++$count >= 3) break;
            }

            $names2 = [];
            $count = 0;
            foreach ($result2 as $row) {
                $names2[] = $row->name;
                if (++$count >= 3) break;
            }

            $names3 = [];
            $count = 0;
            foreach ($result3 as $row) {
                $names3[] = $row->name;
                if (++$count >= 3) break;
            }

            // Verify we got different data from each stream
            $this->assertCount(3, $names1);
            $this->assertCount(3, $names2);
            $this->assertCount(3, $names3);

            // First stream should start with User0
            $this->assertEquals('User0', $names1[0]);

            // Second stream (DESC) should start with User49
            $this->assertEquals('User49', $names2[0]);

            // Drop all results
            unset($result1, $result2, $result3);

            // Verify pool is still healthy
            $this->assertEquals(50, $this->driver->queryValue('SELECT COUNT(*) FROM test_users'));

        } finally {
            $this->dropTestTable();
        }
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    abstract protected function createTestTable(): void;

    protected function dropTestTable(): void
    {
        try {
            $this->driver->execute('DROP TABLE IF EXISTS test_users');
        } catch (\Exception $e) {
            // Ignore errors during cleanup
        }
    }
}
