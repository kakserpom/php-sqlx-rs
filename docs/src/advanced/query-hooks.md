# Query Hooks

Query hooks let you intercept and observe all queries executed by a driver. They're useful for logging, profiling, debugging, and monitoring.

## Setting a Query Hook

```php
$driver->onQuery(function(string $sql, array $params, float $duration) {
    echo "Query: $sql\n";
    echo "Params: " . json_encode($params) . "\n";
    echo "Duration: {$duration}ms\n";
});
```

The callback receives:
- `$sql` - The SQL query (with placeholders)
- `$params` - The bound parameters
- `$duration` - Execution time in milliseconds

## Use Cases

### Logging

```php
$driver->onQuery(function(string $sql, array $params, float $duration) {
    $logger->debug('SQL Query', [
        'sql' => $sql,
        'params' => $params,
        'duration_ms' => $duration,
    ]);
});
```

### Slow Query Detection

```php
$driver->onQuery(function(string $sql, array $params, float $duration) {
    if ($duration > 1000) {  // > 1 second
        $logger->warning('Slow query detected', [
            'sql' => $sql,
            'params' => $params,
            'duration_ms' => $duration,
        ]);
    }
});
```

### Query Counting

```php
$queryCount = 0;
$totalTime = 0;

$driver->onQuery(function(string $sql, array $params, float $duration) use (&$queryCount, &$totalTime) {
    $queryCount++;
    $totalTime += $duration;
});

// After request processing
echo "Executed $queryCount queries in {$totalTime}ms\n";
```

### N+1 Detection

```php
$queries = [];

$driver->onQuery(function(string $sql, array $params, float $duration) use (&$queries) {
    // Normalize query (remove specific values)
    $normalized = preg_replace('/\$\d+/', '?', $sql);
    $queries[$normalized] = ($queries[$normalized] ?? 0) + 1;
});

// After request, check for repeated queries
register_shutdown_function(function() use (&$queries) {
    foreach ($queries as $sql => $count) {
        if ($count > 10) {
            error_log("Possible N+1: '$sql' executed $count times");
        }
    }
});
```

### APM Integration

```php
// Example with a hypothetical APM library
$driver->onQuery(function(string $sql, array $params, float $duration) {
    APM::recordQuery($sql, $duration);
});
```

### Debug Output

```php
if (getenv('DEBUG_SQL')) {
    $driver->onQuery(function(string $sql, array $params, float $duration) {
        $paramStr = empty($params) ? '' : ' -- ' . json_encode($params);
        echo "\033[36m[SQL {$duration}ms]\033[0m $sql$paramStr\n";
    });
}
```

## Removing the Hook

Pass `null` to remove the hook:

```php
$driver->onQuery(null);
```

## Multiple Hooks

Only one hook can be active at a time. To call multiple handlers, compose them:

```php
$handlers = [];

function addQueryHandler(callable $handler) {
    global $handlers;
    $handlers[] = $handler;
}

$driver->onQuery(function(string $sql, array $params, float $duration) {
    global $handlers;
    foreach ($handlers as $handler) {
        $handler($sql, $params, $duration);
    }
});

addQueryHandler(function($sql, $params, $duration) {
    // Logging
});

addQueryHandler(function($sql, $params, $duration) {
    // Metrics
});
```

## Performance Considerations

Query hooks add overhead to every query. For production:

1. **Keep hooks lightweight**: Don't do heavy processing in the callback
2. **Use conditional hooks**: Only enable detailed logging when needed
3. **Buffer and batch**: Collect data and process periodically

```php
// Lightweight production hook
$driver->onQuery(function(string $sql, array $params, float $duration) {
    if ($duration > 100) {  // Only log slow queries
        error_log("Slow query: $sql ({$duration}ms)");
    }
});
```

## Testing with Hooks

Verify queries in tests:

```php
public function testUserCreation()
{
    $executedQueries = [];

    $this->driver->onQuery(function($sql, $params, $duration) use (&$executedQueries) {
        $executedQueries[] = ['sql' => $sql, 'params' => $params];
    });

    $this->userService->createUser('Alice', 'alice@example.com');

    $this->assertCount(1, $executedQueries);
    $this->assertStringContains('INSERT INTO users', $executedQueries[0]['sql']);
    $this->assertEquals(['Alice', 'alice@example.com'], $executedQueries[0]['params']);
}
```

## Request-Scoped Hooks

In web applications, set hooks per-request:

```php
// In middleware
public function handle($request, $next)
{
    $requestId = uniqid();

    $this->driver->onQuery(function($sql, $params, $duration) use ($requestId) {
        Log::debug("[$requestId] SQL: $sql ({$duration}ms)");
    });

    $response = $next($request);

    // Clean up
    $this->driver->onQuery(null);

    return $response;
}
```

## Example: Query Profiler

```php
class QueryProfiler
{
    private array $queries = [];
    private float $startTime;

    public function attach($driver): void
    {
        $this->startTime = microtime(true);
        $this->queries = [];

        $driver->onQuery(function($sql, $params, $duration) {
            $this->queries[] = [
                'sql' => $sql,
                'params' => $params,
                'duration' => $duration,
                'timestamp' => microtime(true) - $this->startTime,
            ];
        });
    }

    public function getReport(): array
    {
        $totalTime = array_sum(array_column($this->queries, 'duration'));

        return [
            'query_count' => count($this->queries),
            'total_time_ms' => $totalTime,
            'queries' => $this->queries,
        ];
    }

    public function getSlowest(int $n = 5): array
    {
        $sorted = $this->queries;
        usort($sorted, fn($a, $b) => $b['duration'] <=> $a['duration']);
        return array_slice($sorted, 0, $n);
    }
}

// Usage
$profiler = new QueryProfiler();
$profiler->attach($driver);

// ... execute queries ...

print_r($profiler->getReport());
print_r($profiler->getSlowest(3));
```
