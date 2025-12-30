<?php

declare(strict_types=1);

namespace PhpSqlx\Tests\Integration\QueryBuilder;

use PHPUnit\Framework\TestCase;
use Sqlx\DriverFactory;
use Sqlx\SqlxException;

use function Sqlx\OR_;

/**
 * Abstract base class for query builder integration tests.
 */
abstract class AbstractQueryBuilderTest extends TestCase
{
    protected mixed $driver;

    abstract protected function getConnectionUrl(): string;
    abstract protected function createTestTable(): void;

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
        $this->dropTestTable();
        // Explicitly close the pool - Drop won't run until PHP exits
        if (isset($this->driver)) {
            $this->driver->close();
        }
        parent::tearDown();
    }

    protected function dropTestTable(): void
    {
        try {
            $this->driver->execute('DROP TABLE IF EXISTS test_users');
            $this->driver->execute('DROP TABLE IF EXISTS test_orders');
        } catch (\Exception $e) {
            // Ignore errors during cleanup
        }
    }

    // =========================================================================
    // Basic Query Builder Tests
    // =========================================================================

    public function testSelectFromWhere(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['name', '=', 'Alice']])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Alice', $users[0]->name);
    }

    public function testSelectWithArray(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");

        $users = $this->driver->builder()
            ->select(['name', 'email'])
            ->from('test_users')
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Alice', $users[0]->name);
    }

    public function testSelectWithAliases(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");

        $users = $this->driver->builder()
            ->select(['user_name' => 'name', 'user_email' => 'email'])
            ->from('test_users')
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Alice', $users[0]->user_name);
        $this->assertEquals('alice@example.com', $users[0]->user_email);
    }

    public function testWhereWithAssociativeArray(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where(['name' => 'Alice'])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Alice', $users[0]->name);
    }

    public function testWhereWithString(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where("name = :name", ['name' => 'Alice'])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Alice', $users[0]->name);
    }

    // =========================================================================
    // Comparison Operators Tests
    // =========================================================================

    public function testOperatorEquals(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Alice', 'alice@example.com', 30)");
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Bob', 'bob@example.com', 25)");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['age', '=', 30]])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Alice', $users[0]->name);
    }

    public function testOperatorNotEquals(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Alice', 'alice@example.com', 30)");
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Bob', 'bob@example.com', 25)");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['age', '!=', 30]])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Bob', $users[0]->name);
    }

    public function testOperatorGreaterThan(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Alice', 'alice@example.com', 30)");
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Bob', 'bob@example.com', 25)");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['age', '>', 26]])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Alice', $users[0]->name);
    }

    public function testOperatorLessThan(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Alice', 'alice@example.com', 30)");
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Bob', 'bob@example.com', 25)");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['age', '<', 26]])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Bob', $users[0]->name);
    }

    public function testOperatorIn(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Charlie', 'charlie@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['name', 'IN', ['Alice', 'Charlie']]])
            ->orderBy('name')
            ->queryAll();

        $this->assertCount(2, $users);
        $this->assertEquals('Alice', $users[0]->name);
        $this->assertEquals('Charlie', $users[1]->name);
    }

    public function testOperatorNotIn(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Charlie', 'charlie@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['name', 'NOT IN', ['Alice', 'Charlie']]])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Bob', $users[0]->name);
    }

    public function testOperatorLike(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['name', 'LIKE', 'Ali%']])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Alice', $users[0]->name);
    }

    public function testOperatorContains(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['name', 'CONTAINS', 'lic']])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Alice', $users[0]->name);
    }

    public function testOperatorNotContains(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['name', 'NOT CONTAINS', 'lic']])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Bob', $users[0]->name);
    }

    public function testOperatorStartsWith(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['name', 'STARTSWITH', 'Ali']])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Alice', $users[0]->name);
    }

    public function testOperatorEndsWith(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['name', 'ENDSWITH', 'ice']])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Alice', $users[0]->name);
    }

    public function testOperatorNotEndsWith(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['name', 'NOT ENDSWITH', 'ice']])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Bob', $users[0]->name);
    }

    public function testOperatorIsNull(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Alice', 'alice@example.com', 30)");
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Bob', 'bob@example.com', NULL)");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['age', 'IS NULL']])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Bob', $users[0]->name);
    }

    public function testOperatorIsNotNull(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Alice', 'alice@example.com', 30)");
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Bob', 'bob@example.com', NULL)");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['age', 'IS NOT NULL']])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Alice', $users[0]->name);
    }

    // =========================================================================
    // OR Clause Tests
    // =========================================================================

    public function testOrClause(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Alice', 'alice@example.com', 30)");
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Bob', 'bob@example.com', 25)");
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Charlie', 'charlie@example.com', 35)");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([
                OR_([
                    ['name', '=', 'Alice'],
                    ['name', '=', 'Charlie'],
                ])
            ])
            ->orderBy('name')
            ->queryAll();

        $this->assertCount(2, $users);
        $this->assertEquals('Alice', $users[0]->name);
        $this->assertEquals('Charlie', $users[1]->name);
    }

    public function testNestedOrClause(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Alice', 'alice@example.com', 30)");
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Bob', 'bob@example.com', 25)");
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Charlie', 'charlie@example.com', 35)");

        // age > 28 AND (name = 'Alice' OR name = 'Charlie')
        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([
                ['age', '>', 28],
                OR_([
                    ['name', '=', 'Alice'],
                    ['name', '=', 'Charlie'],
                ])
            ])
            ->orderBy('name')
            ->queryAll();

        $this->assertCount(2, $users);
        $this->assertEquals('Alice', $users[0]->name);
        $this->assertEquals('Charlie', $users[1]->name);
    }

    // =========================================================================
    // ORDER BY Tests
    // =========================================================================

    public function testOrderByString(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Charlie', 'charlie@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->orderBy('name ASC')
            ->queryAll();

        $this->assertCount(3, $users);
        $this->assertEquals('Alice', $users[0]->name);
        $this->assertEquals('Bob', $users[1]->name);
        $this->assertEquals('Charlie', $users[2]->name);
    }

    public function testOrderByArray(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Charlie', 'charlie@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->orderBy(['name' => 'DESC'])
            ->queryAll();

        $this->assertCount(3, $users);
        $this->assertEquals('Charlie', $users[0]->name);
        $this->assertEquals('Bob', $users[1]->name);
        $this->assertEquals('Alice', $users[2]->name);
    }

    // =========================================================================
    // LIMIT and OFFSET Tests
    // =========================================================================

    public function testLimit(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Charlie', 'charlie@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->orderBy('name')
            ->limit(2)
            ->queryAll();

        $this->assertCount(2, $users);
    }

    public function testLimitWithOffset(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Charlie', 'charlie@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->orderBy('name')
            ->limit(2, 1)
            ->queryAll();

        $this->assertCount(2, $users);
        $this->assertEquals('Bob', $users[0]->name);
        $this->assertEquals('Charlie', $users[1]->name);
    }

    public function testOffset(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Charlie', 'charlie@example.com')");

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->orderBy('name')
            ->limit(10)
            ->offset(1)
            ->queryAll();

        $this->assertCount(2, $users);
        $this->assertEquals('Bob', $users[0]->name);
    }

    // =========================================================================
    // JOIN Tests
    // =========================================================================

    public function testInnerJoin(): void
    {
        $this->createTestTable();
        $this->createOrdersTable();

        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");
        $this->driver->execute("INSERT INTO test_orders (user_id, amount) VALUES (1, 100)");
        $this->driver->execute("INSERT INTO test_orders (user_id, amount) VALUES (1, 200)");

        $results = $this->driver->builder()
            ->select('test_users.name, test_orders.user_id AS amount')
            ->from('test_users')
            ->innerJoin('test_orders', 'test_users.id = test_orders.user_id')
            ->orderBy('test_orders.amount')
            ->queryAll();

        $this->assertCount(2, $results);
        $this->assertEquals('Alice', $results[0]->name);
        // user_id is 1 for both orders
        $this->assertEquals(1, $results[0]->amount);
    }

    public function testLeftJoin(): void
    {
        $this->createTestTable();
        $this->createOrdersTable();

        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");
        $this->driver->execute("INSERT INTO test_orders (user_id, amount) VALUES (1, 100)");

        $results = $this->driver->builder()
            ->select('test_users.name, test_orders.user_id')
            ->from('test_users')
            ->leftJoin('test_orders', 'test_users.id = test_orders.user_id')
            ->orderBy('test_users.name')
            ->queryAll();

        $this->assertCount(2, $results);
        $this->assertEquals('Alice', $results[0]->name);
        $this->assertEquals(1, $results[0]->user_id);
        $this->assertEquals('Bob', $results[1]->name);
        $this->assertNull($results[1]->user_id);
    }

    // =========================================================================
    // GROUP BY and HAVING Tests
    // =========================================================================

    public function testGroupBy(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Alice', 'alice@example.com', 30)");
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Bob', 'bob@example.com', 30)");
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Charlie', 'charlie@example.com', 25)");

        $results = $this->driver->builder()
            ->select('age, COUNT(*) as cnt')
            ->from('test_users')
            ->groupBy('age')
            ->orderBy('age')
            ->queryAll();

        $this->assertCount(2, $results);
        $this->assertEquals(25, $results[0]->age);
        $this->assertEquals(1, $results[0]->cnt);
        $this->assertEquals(30, $results[1]->age);
        $this->assertEquals(2, $results[1]->cnt);
    }

    public function testHaving(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Alice', 'alice@example.com', 30)");
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Bob', 'bob@example.com', 30)");
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Charlie', 'charlie@example.com', 25)");

        $results = $this->driver->builder()
            ->select('age, COUNT(*) as cnt')
            ->from('test_users')
            ->groupBy('age')
            ->having([['COUNT(*)', '>', 1]])
            ->queryAll();

        $this->assertCount(1, $results);
        $this->assertEquals(30, $results[0]->age);
    }

    // =========================================================================
    // INSERT Tests
    // =========================================================================

    public function testInsertWithValues(): void
    {
        $this->createTestTable();

        $this->driver->builder()
            ->insertInto('test_users')
            ->values(['name' => 'Alice', 'email' => 'alice@example.com'])
            ->execute();

        $count = $this->driver->queryValue('SELECT COUNT(*) FROM test_users');
        $this->assertEquals(1, $count);
    }

    public function testInsertValuesMany(): void
    {
        $this->createTestTable();

        $this->driver->builder()
            ->insertInto('test_users')
            ->valuesMany([
                ['name' => 'Alice', 'email' => 'alice@example.com'],
                ['name' => 'Bob', 'email' => 'bob@example.com'],
            ])
            ->execute();

        $count = $this->driver->queryValue('SELECT COUNT(*) FROM test_users');
        $this->assertEquals(2, $count);
    }

    // =========================================================================
    // UPDATE Tests
    // =========================================================================

    public function testUpdate(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");

        $this->driver->builder()
            ->update('test_users')
            ->set(['name' => 'Alice Updated'])
            ->where([['email', '=', 'alice@example.com']])
            ->execute();

        $user = $this->driver->queryRow('SELECT * FROM test_users WHERE email = ?s', ['alice@example.com']);
        $this->assertEquals('Alice Updated', $user->name);
    }

    // =========================================================================
    // DELETE Tests
    // =========================================================================

    public function testDelete(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $this->driver->builder()
            ->deleteFrom('test_users')
            ->where([['name', '=', 'Alice']])
            ->execute();

        $count = $this->driver->queryValue('SELECT COUNT(*) FROM test_users');
        $this->assertEquals(1, $count);
    }

    // =========================================================================
    // Subquery Tests
    // =========================================================================

    public function testSubqueryInWhere(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Alice', 'alice@example.com', 30)");
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Bob', 'bob@example.com', 25)");
        $this->driver->execute("INSERT INTO test_users (name, email, age) VALUES ('Charlie', 'charlie@example.com', 35)");

        $subquery = $this->driver->builder()
            ->select('MAX(age)')
            ->from('test_users');

        $users = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where("age = ?", [$subquery])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Charlie', $users[0]->name);
    }

    // =========================================================================
    // UNION Tests
    // =========================================================================

    public function testUnion(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $results = $this->driver->builder()
            ->select('name')
            ->from('test_users')
            ->where([['name', '=', 'Alice']])
            ->union("SELECT name FROM test_users WHERE name = 'Bob'")
            ->queryAll();

        $this->assertCount(2, $results);
    }

    // =========================================================================
    // Query Execution Methods Tests
    // =========================================================================

    public function testRow(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");

        $user = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->queryRow();

        $this->assertEquals('Alice', $user->name);
    }

    public function testMaybeRow(): void
    {
        $this->createTestTable();

        $user = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->queryMaybeRow();

        $this->assertNull($user);
    }

    public function testQueryColumn(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        // Use queryColumn to get a single value
        $counts = $this->driver->builder()
            ->select('COUNT(*)')
            ->from('test_users')
            ->queryColumn();

        $this->assertEquals(2, $counts[0]);
    }

    public function testQueryColumnMultipleRows(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Bob', 'bob@example.com')");

        $names = $this->driver->builder()
            ->select('name')
            ->from('test_users')
            ->orderBy('name')
            ->queryColumn();

        $this->assertCount(2, $names);
        $this->assertEquals('Alice', $names[0]);
        $this->assertEquals('Bob', $names[1]);
    }

    // =========================================================================
    // Raw Query Tests
    // =========================================================================

    public function testRaw(): void
    {
        $this->createTestTable();
        $this->driver->execute("INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@example.com')");

        $users = $this->driver->builder()
            ->raw('SELECT * FROM test_users WHERE name = :name', ['name' => 'Alice'])
            ->queryAll();

        $this->assertCount(1, $users);
        $this->assertEquals('Alice', $users[0]->name);
    }

    // =========================================================================
    // Quote Tests
    // =========================================================================

    public function testQuote(): void
    {
        $builder = $this->driver->builder();

        $quoted = $builder->quote("O'Reilly");
        $this->assertStringContainsString("O''Reilly", $quoted);
    }

    public function testMetaQuoteLike(): void
    {
        $builder = $this->driver->builder();

        $escaped = $builder->metaQuoteLike("100%_safe");
        $this->assertEquals('100\\%\\_safe', $escaped);
    }

    // =========================================================================
    // __toString Tests
    // =========================================================================

    public function testToString(): void
    {
        $this->createTestTable();

        $builder = $this->driver->builder()
            ->select('*')
            ->from('test_users')
            ->where([['name', '=', 'Alice']]);

        $sql = (string)$builder;
        $this->assertStringContainsString('SELECT', $sql);
        $this->assertStringContainsString('test_users', $sql);
        $this->assertStringContainsString('Alice', $sql);
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    abstract protected function createOrdersTable(): void;
}
