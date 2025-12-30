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
