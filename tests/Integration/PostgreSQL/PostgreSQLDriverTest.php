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

    /**
     * Test that LazyRow works correctly with json_encode.
     *
     * LazyRow is used when JSON data exceeds 4096 bytes (LAZY_ROW_JSON_SIZE_THRESHOLD).
     * This test verifies that json_encode works properly on LazyRow objects.
     */
    public function testLazyRowJsonEncode(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_lazy_json');
        $this->driver->execute('
            CREATE TABLE test_lazy_json (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255),
                large_data JSONB
            )
        ');

        try {
            // Create JSON data larger than 4096 bytes to trigger LazyRow
            $largeArray = [];
            for ($i = 0; $i < 200; $i++) {
                $largeArray[] = [
                    'index' => $i,
                    'name' => "Item number {$i} with some extra text to increase size",
                    'description' => str_repeat("Description text for item {$i}. ", 5),
                    'tags' => ['tag1', 'tag2', 'tag3'],
                ];
            }
            $largeJson = json_encode($largeArray);

            // Verify our test data is actually large enough
            $this->assertGreaterThan(4096, strlen($largeJson), 'Test JSON should exceed lazy threshold');

            $this->driver->execute(
                "INSERT INTO test_lazy_json (name, large_data) VALUES (:name, :data::jsonb)",
                ['name' => 'Test Row', 'data' => $largeJson]
            );

            // Query the row - this should return a LazyRow if the JSON is large enough
            $row = $this->driver->queryRow('SELECT id, name, large_data FROM test_lazy_json WHERE id = 1');
            $this->assertNotNull($row);

            // Debug: check the row type and content
            $rowClass = get_class($row);

            // Check if row is a LazyRow
            $isLazyRow = $row instanceof \Sqlx\LazyRow;

            // Access large_data directly
            $largeDataDirect = $row->large_data;
            $largeDataType = gettype($largeDataDirect);
            $largeDataClass = is_object($largeDataDirect) ? get_class($largeDataDirect) : 'N/A';
            $isLazyRowJson = $largeDataDirect instanceof \Sqlx\LazyRowJson;

            // Test json_encode on the row - this should trigger JsonSerializable on LazyRowJson
            $encodedRow = json_encode($row);
            $this->assertNotFalse($encodedRow, "json_encode should not fail on row with large JSON (row class: {$rowClass}, isLazyRow: " . ($isLazyRow ? 'yes' : 'no') . ", largeDataType: {$largeDataType}, largeDataClass: {$largeDataClass}, isLazyRowJson: " . ($isLazyRowJson ? 'yes' : 'no') . ")");

            $decodedRow = json_decode($encodedRow, true);
            $this->assertIsArray($decodedRow, "Decoded row should be array, got: " . var_export($decodedRow, true));
            $this->assertEquals('Test Row', $decodedRow['name']);
            $this->assertArrayHasKey('large_data', $decodedRow, "Row should have large_data key. Keys: " . implode(', ', array_keys($decodedRow)) . ". Row class: {$rowClass}. Encoded: " . substr($encodedRow, 0, 500));
            $this->assertIsArray($decodedRow['large_data'], "large_data should be array after json_encode/decode, got: " . gettype($decodedRow['large_data']) . " = " . var_export($decodedRow['large_data'], true));
            $this->assertCount(200, $decodedRow['large_data'], "large_data should have 200 items after json_encode/decode, got: " . count($decodedRow['large_data']) . ". Type: " . gettype($decodedRow['large_data']));
            $this->assertEquals(0, $decodedRow['large_data'][0]['index']);

            // Also test json_encode directly on the LazyRowJson object
            $largeData = $row->large_data;
            $encodedData = json_encode($largeData);
            $this->assertNotFalse($encodedData, 'json_encode should work on LazyRowJson via JsonSerializable');

            $decodedData = json_decode($encodedData, true);
            $this->assertCount(200, $decodedData, 'Decoded LazyRowJson should have 200 items');
        } finally {
            $this->driver->execute('DROP TABLE IF EXISTS test_lazy_json');
        }
    }

    /**
     * Test that queryAll with large JSON returns properly json_encode-able results.
     */
    public function testLazyRowQueryAllJsonEncode(): void
    {
        $this->driver->execute('DROP TABLE IF EXISTS test_lazy_json_all');
        $this->driver->execute('
            CREATE TABLE test_lazy_json_all (
                id SERIAL PRIMARY KEY,
                data JSONB
            )
        ');

        try {
            // Create large JSON data
            $largeData = ['items' => array_fill(0, 100, ['key' => str_repeat('value', 100)])];
            $largeJson = json_encode($largeData);
            $this->assertGreaterThan(4096, strlen($largeJson));

            $this->driver->execute(
                "INSERT INTO test_lazy_json_all (data) VALUES (:data::jsonb)",
                ['data' => $largeJson]
            );
            $this->driver->execute(
                "INSERT INTO test_lazy_json_all (data) VALUES (:data::jsonb)",
                ['data' => $largeJson]
            );

            $rows = $this->driver->queryAll('SELECT * FROM test_lazy_json_all ORDER BY id');
            $this->assertCount(2, $rows);

            // Test json_encode on the entire result set
            $encoded = json_encode($rows);
            $this->assertNotFalse($encoded, 'json_encode should work on queryAll result with large JSON');

            $decoded = json_decode($encoded, true);
            $this->assertIsArray($decoded);
            $this->assertCount(2, $decoded);
            $this->assertArrayHasKey('data', $decoded[0]);
            $this->assertArrayHasKey('items', $decoded[0]['data']);
        } finally {
            $this->driver->execute('DROP TABLE IF EXISTS test_lazy_json_all');
        }
    }

    /**
     * Test that JSON numbers are converted to proper PHP types (int/float) not strings.
     */
    public function testJsonNumberTypes(): void
    {
        $row = $this->driver->queryRow("SELECT '{\"int\": 42, \"float\": 3.14, \"str\": \"hello\", \"bool\": true, \"null\": null}'::jsonb as data");
        $this->assertNotNull($row);

        $data = $row->data;
        $this->assertIsObject($data, 'JSON object should decode to PHP object');

        // Verify integer type - should be int, not string
        $this->assertIsInt($data->int, 'JSON integer should be PHP int, got: ' . gettype($data->int));
        $this->assertEquals(42, $data->int);

        // Verify float type - should be float, not string
        $this->assertIsFloat($data->float, 'JSON float should be PHP float, got: ' . gettype($data->float));
        $this->assertEquals(3.14, $data->float);

        // Verify string type
        $this->assertIsString($data->str);
        $this->assertEquals('hello', $data->str);

        // Verify boolean type
        $this->assertIsBool($data->bool);
        $this->assertTrue($data->bool);

        // Verify null
        $this->assertNull($data->null);

        // Also test via json_encode/decode round-trip
        $encoded = json_encode($row);
        $decoded = json_decode($encoded);
        $this->assertIsInt($decoded->data->int, 'After json_encode/decode: integer should remain int');
        $this->assertIsFloat($decoded->data->float, 'After json_encode/decode: float should remain float');
    }

    /**
     * Test JSON number types with large JSON (LazyRow path).
     */
    public function testLazyRowJsonNumberTypes(): void
    {
        // Create JSON larger than 4096 bytes to trigger LazyRow
        $padding = str_repeat('x', 4100);
        $json = json_encode([
            'int' => 12345,
            'float' => 123.456,
            'negative_int' => -999,
            'negative_float' => -1.5,
            'zero' => 0,
            'padding' => $padding,
        ]);

        // Use a parameter to pass the JSON - avoids dollar-quoting issues
        $row = $this->driver->queryRow("SELECT :json::jsonb as data", ['json' => $json]);
        $this->assertNotNull($row);

        $data = $row->data;
        $this->assertIsObject($data, 'JSON object should decode to PHP object');

        $this->assertIsInt($data->int, 'Large JSON: integer should be PHP int');
        $this->assertEquals(12345, $data->int);

        $this->assertIsFloat($data->float, 'Large JSON: float should be PHP float');
        $this->assertEquals(123.456, $data->float);

        $this->assertIsInt($data->negative_int, 'Large JSON: negative integer should be PHP int');
        $this->assertEquals(-999, $data->negative_int);

        $this->assertIsFloat($data->negative_float, 'Large JSON: negative float should be PHP float');
        $this->assertEquals(-1.5, $data->negative_float);

        $this->assertIsInt($data->zero, 'Large JSON: zero should be PHP int');
        $this->assertEquals(0, $data->zero);
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
