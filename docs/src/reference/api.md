# API Reference

Complete reference for all php-sqlx classes, interfaces, and methods.

## Interfaces Overview

php-sqlx provides interfaces for dependency injection and testing:

| Interface | Description |
|-----------|-------------|
| `DriverInterface` | Common driver methods (PostgreSQL, MSSQL) |
| `SqlDriverInterface` | MySQL driver methods |
| `PreparedQueryInterface` | Prepared statement methods |
| `ReadQueryBuilderInterface` | Read query builder methods |
| `WriteQueryBuilderInterface` | Write query builder methods |
| `RowInterface` | Result row with ArrayAccess |
| `RowJsonInterface` | JSON column access |

## DriverFactory

```php
namespace Sqlx;

class DriverFactory implements FactoryInterface
{
    /**
     * Create a driver instance
     * @param string|array $urlOrOptions Connection URL or options array
     * @return DriverInterface
     */
    public static function make(string|array $urlOrOptions): DriverInterface;
}
```

## DriverInterface

All drivers (`PgDriver`, `MySqlDriver`, `MssqlDriver`) implement this interface:

### Constructor

```php
public function __construct(string|array $urlOrOptions);
```

### Query Methods - Single Row

```php
// Get single row, throws if not found
public function queryRow(string $query, ?array $params = null): object;
public function queryRowAssoc(string $query, ?array $params = null): array;
public function queryRowObj(string $query, ?array $params = null): object;

// Get single row or null
public function queryMaybeRow(string $query, ?array $params = null): ?object;
public function queryMaybeRowAssoc(string $query, ?array $params = null): ?array;
public function queryMaybeRowObj(string $query, ?array $params = null): ?object;
```

### Query Methods - Multiple Rows

```php
public function queryAll(string $query, ?array $params = null): array;
public function queryAllAssoc(string $query, ?array $params = null): array;
public function queryAllObj(string $query, ?array $params = null): array;
```

### Query Methods - Single Value

```php
// Get single value, throws if not found
public function queryValue(string $query, ?array $params = null, mixed $column = 0): mixed;
public function queryValueAssoc(string $query, ?array $params = null, mixed $column = 0): mixed;
public function queryValueObj(string $query, ?array $params = null, mixed $column = 0): mixed;

// Get single value or null
public function queryMaybeValue(string $query, ?array $params = null, mixed $column = 0): mixed;
public function queryMaybeValueAssoc(string $query, ?array $params = null, mixed $column = 0): mixed;
public function queryMaybeValueObj(string $query, ?array $params = null, mixed $column = 0): mixed;
```

### Query Methods - Column

```php
public function queryColumn(string $query, ?array $params = null, mixed $column = 0): array;
public function queryColumnAssoc(string $query, ?array $params = null, mixed $column = 0): array;
public function queryColumnObj(string $query, ?array $params = null, mixed $column = 0): array;
```

### Query Methods - Dictionary

```php
// Map first column to entire row
public function queryDictionary(string $query, ?array $params = null): array;
public function queryDictionaryAssoc(string $query, ?array $params = null): array;
public function queryDictionaryObj(string $query, ?array $params = null): array;

// Map first column to second column
public function queryColumnDictionary(string $query, ?array $params = null): array;
public function queryColumnDictionaryAssoc(string $query, ?array $params = null): array;
public function queryColumnDictionaryObj(string $query, ?array $params = null): array;

// Group rows by first column
public function queryGroupedDictionary(string $query, ?array $params = null): array;
public function queryGroupedDictionaryAssoc(string $query, ?array $params = null): array;
public function queryGroupedDictionaryObj(string $query, ?array $params = null): array;

// Group second column by first column
public function queryGroupedColumnDictionary(string $query, ?array $params = null): array;
public function queryGroupedColumnDictionaryAssoc(string $query, ?array $params = null): array;
public function queryGroupedColumnDictionaryObj(string $query, ?array $params = null): array;
```

### Data Modification

```php
// Execute statement, return affected rows
public function execute(string $query, ?array $params = null): int;

// Insert single row, returns affected rows
public function insert(string $table, array $row): int;

// Insert multiple rows, returns affected rows
public function insertMany(string $table, array $rows): int;

// Insert or update on conflict
public function upsert(string $table, array $row, array $conflictCols, ?array $updateCols = null): void;
```

### Transactions

```php
// Start transaction (callback or imperative)
public function begin(?callable $callback = null): mixed;

// Commit current transaction
public function commit(): void;

// Rollback current transaction
public function rollback(): void;

// Savepoint operations
public function savepoint(string $name): void;
public function rollbackToSavepoint(string $name): void;
public function releaseSavepoint(string $name): void;
```

### Utilities

```php
// Quote value for SQL
public function quote(mixed $param): string;

// Quote string for LIKE pattern (escapes % and _)
public function metaQuoteLike(string $param): string;

// Quote identifier (table/column name)
public function quoteIdentifier(string $name): string;

// Render query without executing
public function dry(string $query, ?array $params = null): array; // [sql, params]

// Describe table columns
public function describeTable(string $table, ?string $schema = null): array;
```

### Prepared Queries

```php
public function prepare(string $query): PreparedQuery;
```

### Query Builder

```php
public function builder(): WriteQueryBuilder;
public function readBuilder(): ReadQueryBuilder;
```

### Configuration

```php
// Set application name
public function setApplicationName(string $name): void;

// Set client info
public function setClientInfo(string $appName, array $info): void;

// Check if returning arrays
public function assocArrays(): bool;

// Check for read replicas
public function hasReadReplicas(): bool;
```

### Lifecycle

```php
// Close all connections
public function close(): void;

// Check if closed
public function isClosed(): bool;

// Pin to single connection for callback
public function withConnection(callable $callback): mixed;

// Set query hook
public function onQuery(?callable $callback): void;
```

## PreparedQueryInterface

Prepared statements implement `PreparedQueryInterface`:

```php
public function execute(?array $params = null): int;

public function queryRow(?array $params = null): object;
public function queryRowAssoc(?array $params = null): array;
public function queryRowObj(?array $params = null): object;

public function queryMaybeRow(?array $params = null): ?object;
public function queryMaybeRowAssoc(?array $params = null): ?array;
public function queryMaybeRowObj(?array $params = null): ?object;

public function queryAll(?array $params = null): array;
public function queryAllAssoc(?array $params = null): array;
public function queryAllObj(?array $params = null): array;

// ... same dictionary methods as driver
```

## Query Builder Interfaces

Query builders implement `ReadQueryBuilderInterface` or `WriteQueryBuilderInterface`.

### Clause Methods

```php
public function select(mixed $fields): static;
public function from(mixed $table, ?array $params = null): static;
public function where(mixed $conditions, ?array $params = null): static;
public function having(mixed $conditions, ?array $params = null): static;
public function orderBy(mixed $order): static;
public function groupBy(mixed $columns): static;
public function limit(int $limit, ?int $offset = null): static;
public function offset(int $offset): static;
public function paginate(mixed $paginate): static;
public function raw(string $part, ?array $params = null): static;
```

### Join Methods

```php
public function join(string $table, string $on, ?array $params = null): static;
public function innerJoin(string $table, string $on, ?array $params = null): static;
public function leftJoin(string $table, string $on, ?array $params = null): static;
public function rightJoin(string $table, string $on, ?array $params = null): static;
public function fullJoin(string $table, string $on, ?array $params = null): static;
public function crossJoin(string $table, ?string $on = null, ?array $params = null): static;
public function naturalJoin(string $table): static;
```

### CTE and Union

```php
public function with(string $name, string|Builder $as, ?array $params = null): static;
public function withRecursive(string $tableFields, string|Builder $as, ?array $params = null): static;
public function union(string|Builder $query, ?array $params = null): static;
public function unionAll(string|Builder $query, ?array $params = null): static;
```

### Locking

```php
public function forUpdate(): static;
public function forShare(): static;
```

### Write Operations

```php
public function insertInto(string $table): static;
public function replaceInto(string $table): static;  // MySQL REPLACE INTO
public function update(string $table): static;
public function deleteFrom(string $table): static;
public function truncateTable(string $table): static;
public function set(array $assignments): static;
public function values(array $row): static;
public function valuesMany(array $rows): static;
public function returning(string|array $columns): static;
public function onConflict(string|array $target, ?array $set = null): static;
public function onDuplicateKeyUpdate(array $set): static;
public function using(string|Builder $from, ?array $params = null): static;
```

### Execution

```php
public function execute(): int;
public function queryRow(): object;
public function queryAll(): array;
// ... all query methods (same as Driver)

// Get rendered SQL and parameters without executing
public function dry(): array;  // [sql, params]
public function dryInline(): string;  // SQL with params inlined

// Get current parameters
public function parameters(): array;

// Get SQL string
public function __toString(): string;

// Return to parent builder (for subqueries)
public function end(): static;
```

## DriverOptions

```php
namespace Sqlx;

class DriverOptions
{
    // Core
    const OPT_URL = 'url';
    const OPT_ASSOC_ARRAYS = 'assoc_arrays';
    const OPT_READONLY = 'readonly';
    const OPT_PERSISTENT_NAME = 'persistent_name';
    const OPT_COLLAPSIBLE_IN = 'collapsible_in';

    // AST Cache
    const OPT_AST_CACHE_SHARD_COUNT = 'ast_cache_shard_count';
    const OPT_AST_CACHE_SHARD_SIZE = 'ast_cache_shard_size';

    // Connection Pool
    const OPT_MAX_CONNECTIONS = 'max_connections';
    const OPT_MIN_CONNECTIONS = 'min_connections';
    const OPT_MAX_LIFETIME = 'max_lifetime';
    const OPT_IDLE_TIMEOUT = 'idle_timeout';
    const OPT_ACQUIRE_TIMEOUT = 'acquire_timeout';
    const OPT_TEST_BEFORE_ACQUIRE = 'test_before_acquire';

    // Read Replicas
    const OPT_READ_REPLICAS = 'read_replicas';

    // Retry Policy
    const OPT_RETRY_MAX_ATTEMPTS = 'retry_max_attempts';
    const OPT_RETRY_INITIAL_BACKOFF = 'retry_initial_backoff';
    const OPT_RETRY_MAX_BACKOFF = 'retry_max_backoff';
    const OPT_RETRY_MULTIPLIER = 'retry_multiplier';
}
```

## Clause Helpers

```php
namespace Sqlx;

class SelectClause
{
    public function __construct(array $allowed = []);
    public function allowed(array $cols): static;
    public function input(array $cols): SelectClauseRendered;
}

class ByClause
{
    public function __construct(array $allowed = []);
    public function allowed(array $cols): static;
    public function input(array $cols): ByClauseRendered;
}

class PaginateClause
{
    public function __construct();
    public function perPage(int $count): static;
    public function minPerPage(int $min): static;
    public function maxPerPage(int $max): static;
    public function __invoke(?int $page, ?int $perPage): PaginateClauseRendered;
}
```

## Exceptions

```php
namespace Sqlx\Exceptions;

class SqlxException extends \Exception
{
    const GENERAL = 0;
    const CONNECTION = 1;
    const QUERY = 2;
    const TRANSACTION = 3;
    const PARSE = 4;
    const PARAMETER = 5;
    const CONFIGURATION = 6;
    const VALIDATION = 7;
    const NOT_PERMITTED = 8;
    const TIMEOUT = 9;
    const POOL_EXHAUSTED = 10;

    public function isTransient(): bool;
    public function getSql(): ?string;
}

class ConnectionException extends SqlxException {}
class QueryException extends SqlxException {}
class TransactionException extends SqlxException {}
class ParseException extends SqlxException {}
class ParameterException extends SqlxException {}
class ConfigurationException extends SqlxException {}
class ValidationException extends SqlxException {}
class NotPermittedException extends SqlxException {}
class TimeoutException extends SqlxException {}
class PoolExhaustedException extends SqlxException {}
```

## Functions

```php
namespace Sqlx;

// Create OR condition for where clauses
function OR_(array $conditions): OrClause;
```

## Row Interfaces

### RowInterface

Result rows implement `ArrayAccess` for property access:

```php
namespace Sqlx;

interface RowInterface extends \ArrayAccess
{
    public function offsetExists(mixed $offset): bool;
    public function offsetGet(mixed $offset): mixed;
    public function offsetSet(mixed $offset, mixed $value): void;
    public function offsetUnset(mixed $offset): void;
}
```

Usage:

```php
$user = $driver->queryRow("SELECT * FROM users WHERE id = ?", [1]);

// Object property access
echo $user->name;

// Array access (via ArrayAccess)
echo $user['name'];
```

### RowJsonInterface

For rows with JSON columns, provides automatic JSON decoding:

```php
namespace Sqlx;

interface RowJsonInterface
{
    // Get JSON column as decoded PHP array/object
    public function json(string $column): mixed;
}
```

Usage:

```php
$event = $driver->queryRow("SELECT * FROM events WHERE id = ?", [1]);

// Get JSON column as PHP array
$data = $event->json('payload');

// Access nested data
echo $data['user']['name'];
```
