# Query Hooks

Query hooks let you intercept and observe all queries executed by a driver. They're useful for logging, profiling, debugging, and monitoring.

## Setting a Query Hook

```php
$driver->onQuery(function(
    string $sql,
    string $sqlInline,
    float $durationMs,
    ?int $rows,
    ?string $error,
) {
    echo "Query: $sqlInline\n";
    echo "Rows: " . ($rows ?? 'n/a') . "\n";
    echo "Duration: {$durationMs}ms\n";
    if ($error !== null) {
        echo "Error: $error\n";
    }
});
```

The callback receives:

- `$sql` – The rendered SQL with placeholders (`SELECT * FROM users WHERE status = $1`)
- `$sqlInline` – The SQL with inlined parameter values, for logging (`SELECT * FROM users WHERE status = 'active'`)
- `$durationMs` – Execution time in milliseconds (DB execution only, excluding row-to-PHP conversion)
- `$rows` – Rows affected (writes) or returned (reads), or `null` if unknown
- `$error` – The error message if the query failed, or `null` on success

The hook fires after **every** query, including failed ones. On failure, `$error` is set and `$rows` is `null`.

> **Note:** PHP ignores trailing arguments a closure doesn't declare, so a hook that only accepts `(string $sql, string $sqlInline, float $durationMs)` keeps working unchanged.

## Use Cases

### Logging

```php
$driver->onQuery(function(string $sql, string $sqlInline, float $durationMs, ?int $rows, ?string $error) use ($logger) {
    $logger->debug('SQL Query', [
        'sql' => $sqlInline,
        'rows' => $rows,
        'duration_ms' => $durationMs,
        'error' => $error,
    ]);
});
```

### Error Logging

The hook fires on failures too, so you can capture every error in one place:

```php
$driver->onQuery(function(string $sql, string $sqlInline, float $durationMs, ?int $rows, ?string $error) use ($logger) {
    if ($error !== null) {
        $logger->error('Query failed', [
            'sql' => $sqlInline,
            'duration_ms' => $durationMs,
            'error' => $error,
        ]);
    }
});
```

### Slow Query Detection

```php
$driver->onQuery(function(string $sql, string $sqlInline, float $durationMs, ?int $rows, ?string $error) use ($logger) {
    if ($durationMs > 1000) {  // > 1 second
        $logger->warning('Slow query detected', [
            'sql' => $sqlInline,
            'rows' => $rows,
            'duration_ms' => $durationMs,
        ]);
    }
});
```

### Large Result-Set Detection

`$rows` makes it easy to flag unbounded reads before they hurt:

```php
$driver->onQuery(function(string $sql, string $sqlInline, float $durationMs, ?int $rows, ?string $error) use ($logger) {
    if ($rows !== null && $rows > 10_000) {
        $logger->warning("Query returned {$rows} rows", ['sql' => $sqlInline]);
    }
});
```

### Query Counting

```php
$queryCount = 0;
$totalTime = 0;

$driver->onQuery(function(string $sql, string $sqlInline, float $durationMs) use (&$queryCount, &$totalTime) {
    $queryCount++;
    $totalTime += $durationMs;
});

// After request processing
echo "Executed $queryCount queries in {$totalTime}ms\n";
```

### N+1 Detection

```php
$queries = [];

$driver->onQuery(function(string $sql, string $sqlInline, float $durationMs) use (&$queries) {
    // The placeholder form already strips specific values, so it normalizes well
    $queries[$sql] = ($queries[$sql] ?? 0) + 1;
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
$driver->onQuery(function(string $sql, string $sqlInline, float $durationMs, ?int $rows, ?string $error) {
    APM::recordQuery($sql, $durationMs, rows: $rows, error: $error);
});
```

### Debug Output

```php
if (getenv('DEBUG_SQL')) {
    $driver->onQuery(function(string $sql, string $sqlInline, float $durationMs, ?int $rows, ?string $error) {
        $tag = $error !== null ? "\033[31mERR\033[0m" : "\033[36m{$durationMs}ms\033[0m";
        echo "[SQL $tag] $sqlInline\n";
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

$driver->onQuery(function(string $sql, string $sqlInline, float $durationMs, ?int $rows, ?string $error) {
    global $handlers;
    foreach ($handlers as $handler) {
        $handler($sql, $sqlInline, $durationMs, $rows, $error);
    }
});

addQueryHandler(function($sql, $sqlInline, $durationMs, $rows, $error) {
    // Logging
});

addQueryHandler(function($sql, $sqlInline, $durationMs, $rows, $error) {
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
$driver->onQuery(function(string $sql, string $sqlInline, float $durationMs) {
    if ($durationMs > 100) {  // Only log slow queries
        error_log("Slow query: $sqlInline ({$durationMs}ms)");
    }
});
```

## Testing with Hooks

Verify queries in tests:

```php
public function testUserCreation()
{
    $executedQueries = [];

    $this->driver->onQuery(function($sql, $sqlInline, $durationMs, $rows, $error) use (&$executedQueries) {
        $executedQueries[] = ['sql' => $sql, 'rows' => $rows, 'error' => $error];
    });

    $this->userService->createUser('Alice', 'alice@example.com');

    $this->assertCount(1, $executedQueries);
    $this->assertStringContains('INSERT INTO users', $executedQueries[0]['sql']);
    $this->assertSame(1, $executedQueries[0]['rows']);
    $this->assertNull($executedQueries[0]['error']);
}
```

## Request-Scoped Hooks

In web applications, set hooks per-request:

```php
// In middleware
public function handle($request, $next)
{
    $requestId = uniqid();

    $this->driver->onQuery(function($sql, $sqlInline, $durationMs) use ($requestId) {
        Log::debug("[$requestId] SQL: $sqlInline ({$durationMs}ms)");
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

        $driver->onQuery(function($sql, $sqlInline, $durationMs, $rows, $error) {
            $this->queries[] = [
                'sql' => $sqlInline,
                'rows' => $rows,
                'error' => $error,
                'duration' => $durationMs,
                'timestamp' => microtime(true) - $this->startTime,
            ];
        });
    }

    public function getReport(): array
    {
        $totalTime = array_sum(array_column($this->queries, 'duration'));

        return [
            'query_count' => count($this->queries),
            'error_count' => count(array_filter($this->queries, fn($q) => $q['error'] !== null)),
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
