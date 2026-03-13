<?php

// Stubs for sqlx

namespace Sqlx {
    /**
     * @param mixed $pv
     * @return \Sqlx\JsonWrapper
     */
    function JSON(mixed $pv): \Sqlx\JsonWrapper {}

    /**
     * A PHP-accessible wrapper around a `zend_array` that lazily decodes JSON values.
     *
     * Extends `stdClass` for compatibility with code expecting objects.
     * Implements `ArrayAccess` so that columns can be accessed as array entries.
     * Implements `Iterator` so that columns can be iterated with `foreach`.
     * Implements `JsonSerializable` so that `json_encode()` works correctly.
     */
    class LazyRow extends \stdClass implements \ArrayAccess, \Iterator, \JsonSerializable {
        /**
         * Checks if the current position is valid.
         *
         * @return bool
         */
        public function valid(): bool {}

        /**
         * Checks whether a column exists in the row.
         *
         * @param mixed $offset – The column name as a `Zval` (expected to be a string).
         * @return bool `Ok(true)` if the column exists, `Ok(false)` otherwise, or an error if the offset isn't a string.
         */
        public function offsetExists(mixed $offset): bool {}

        /**
         * Implementation of `JsonSerializable::jsonSerialize()`.
         *
         * Returns the underlying array with all `LazyRowJson` values decoded for `json_encode()`.
         *
         * @return mixed
         */
        public function jsonSerialize(): mixed {}

        /**
         * Magic getter for property access in PHP (`$row->column`).
         *
         * Lazily decodes JSON-wrapped values if needed and replaces the placeholder object
         * with the actual decoded `Zval`.
         *
         * @param string $name – The column name.
         * @return mixed
         * @throws \Exception Returns a `PhpException` if the column is not found or offset is not a string.
         */
        public function __get(string $name): mixed {}

        /**
         * Magic isset for checking if a property exists (`isset($row->column)`).
         *
         * @param string $name – The column name.
         * @return bool `true` if the column exists, `false` otherwise.
         */
        public function __isset(string $name): bool {}

        /**
         * Moves the iterator to the next element.
         *
         * @return void
         */
        public function next(): void {}

        /**
         * Returns the current element value (with lazy JSON decoding).
         *
         * @return mixed
         */
        public function current(): mixed {}

        /**
         * Returns the key of the current element.
         *
         * @return mixed
         */
        public function key(): mixed {}

        /**
         * Rewinds the iterator to the first element.
         *
         * @return void
         */
        public function rewind(): void {}

        /**
         * `ArrayAccess` getter (`$row[$column]`).
         *
         * Performs the same lazy JSON decoding logic as `__get`.
         *
         * @param mixed $offset
         * @return mixed
         */
        public function offsetGet(mixed $offset): mixed {}

        /**
         * `ArrayAccess` setter (`$row[$key] = $value`).
         *
         * Inserts or updates the given key with the provided `Zval`.
         *
         * @param mixed $offset
         * @param mixed $value
         * @return void
         */
        public function offsetSet(mixed $offset, mixed $value): void {}

        /**
         * `ArrayAccess` unsetter (`unset($row[$key])`).
         *
         * Unsetting values is not supported and always returns an error.
         *
         * @param mixed $offset
         * @return void
         */
        public function offsetUnset(mixed $offset): void {}

        public function __construct() {}
    }

    /**
     * A helper PHP class that holds raw JSON bytes for lazy decoding.
     *
     * When accessed, it will be parsed into a PHP value on demand.
     * Implements `JsonSerializable` so that `json_encode()` works correctly.
     */
    class LazyRowJson implements \JsonSerializable {
        /**
         * Implementation of `JsonSerializable::jsonSerialize()`.
         *
         * Decodes and returns the stored JSON data for use with `json_encode()`.
         *
         * @return mixed
         */
        public function jsonSerialize(): mixed {}

        public function __construct() {}
    }

    /**
     * A lazy query result iterator that streams rows on demand.
     *
     * This class implements PHP's `Iterator` interface, allowing it to be used
     * with `foreach` loops. Rows are streamed from the database through a
     * channel as you iterate, providing true lazy loading.
     *
     * # Example
     * ```php
     * $result = $driver->query('SELECT * FROM large_table');
     * foreach ($result as $row) {
     *     // Rows are fetched on demand as you iterate
     *     process($row);
     * }
     * ```
     */
    class MssqlQueryResult implements \Iterator {
        /**
         * Advances the iterator to the next row.
         *
         * @return void
         */
        public function next(): void {}

        /**
         * Consumes all remaining rows and returns them as an array.
         *
         * This will fetch all remaining rows from the stream.
         * Use with caution on large result sets.
         *
         * @return array
         */
        public function toArray(): array {}

        /**
         * Resets the iterator to the beginning.
         *
         * On first call, fetches the first row.
         * Note: The stream cannot be truly rewound - this only works
         * before any iteration has occurred.
         *
         * @return void
         */
        public function rewind(): void {}

        /**
         * Returns the configured buffer size for streaming.
         *
         * @return int
         */
        public function getBatchSize(): int {}

        /**
         * Returns the current index (0-based position).
         *
         * @return int
         */
        public function key(): int {}

        /**
         * Returns the current row.
         *
         * Returns the row at the current iterator position, or null if
         * the position is invalid.
         *
         * @return mixed
         */
        public function current(): mixed {}

        /**
         * Returns the last error that occurred, if any.
         *
         * This is useful for checking if iteration stopped due to an error.
         *
         * @return string|null
         */
        public function getLastError(): ?string {}

        /**
         * Returns the number of rows fetched so far.
         *
         * Note: This returns the count of rows fetched, not the total
         * result set size (which may not be known until iteration completes).
         *
         * @return int
         */
        public function count(): int {}

        /**
         * Returns true if the current position is valid.
         *
         * @return bool
         */
        public function valid(): bool {}

        /**
         * Returns true if the result set has been fully consumed.
         *
         * @return bool
         */
        public function isExhausted(): bool {}

        public function __construct() {}
    }

    /**
     * A lazy query result iterator that streams rows on demand.
     *
     * This class implements PHP's `Iterator` interface, allowing it to be used
     * with `foreach` loops. Rows are streamed from the database through a
     * channel as you iterate, providing true lazy loading.
     *
     * # Example
     * ```php
     * $result = $driver->query('SELECT * FROM large_table');
     * foreach ($result as $row) {
     *     // Rows are fetched on demand as you iterate
     *     process($row);
     * }
     * ```
     */
    class MySqlQueryResult implements \Iterator {
        /**
         * Advances the iterator to the next row.
         *
         * @return void
         */
        public function next(): void {}

        /**
         * Consumes all remaining rows and returns them as an array.
         *
         * This will fetch all remaining rows from the stream.
         * Use with caution on large result sets.
         *
         * @return array
         */
        public function toArray(): array {}

        /**
         * Resets the iterator to the beginning.
         *
         * On first call, fetches the first row.
         * Note: The stream cannot be truly rewound - this only works
         * before any iteration has occurred.
         *
         * @return void
         */
        public function rewind(): void {}

        /**
         * Returns the configured buffer size for streaming.
         *
         * @return int
         */
        public function getBatchSize(): int {}

        /**
         * Returns the current index (0-based position).
         *
         * @return int
         */
        public function key(): int {}

        /**
         * Returns the current row.
         *
         * Returns the row at the current iterator position, or null if
         * the position is invalid.
         *
         * @return mixed
         */
        public function current(): mixed {}

        /**
         * Returns the last error that occurred, if any.
         *
         * This is useful for checking if iteration stopped due to an error.
         *
         * @return string|null
         */
        public function getLastError(): ?string {}

        /**
         * Returns the number of rows fetched so far.
         *
         * Note: This returns the count of rows fetched, not the total
         * result set size (which may not be known until iteration completes).
         *
         * @return int
         */
        public function count(): int {}

        /**
         * Returns true if the current position is valid.
         *
         * @return bool
         */
        public function valid(): bool {}

        /**
         * Returns true if the result set has been fully consumed.
         *
         * @return bool
         */
        public function isExhausted(): bool {}

        public function __construct() {}
    }

    /**
     * A lazy query result iterator that streams rows on demand.
     *
     * This class implements PHP's `Iterator` interface, allowing it to be used
     * with `foreach` loops. Rows are streamed from the database through a
     * channel as you iterate, providing true lazy loading.
     *
     * # Example
     * ```php
     * $result = $driver->query('SELECT * FROM large_table');
     * foreach ($result as $row) {
     *     // Rows are fetched on demand as you iterate
     *     process($row);
     * }
     * ```
     */
    class PgQueryResult implements \Iterator {
        /**
         * Advances the iterator to the next row.
         *
         * @return void
         */
        public function next(): void {}

        /**
         * Consumes all remaining rows and returns them as an array.
         *
         * This will fetch all remaining rows from the stream.
         * Use with caution on large result sets.
         *
         * @return array
         */
        public function toArray(): array {}

        /**
         * Resets the iterator to the beginning.
         *
         * On first call, fetches the first row.
         * Note: The stream cannot be truly rewound - this only works
         * before any iteration has occurred.
         *
         * @return void
         */
        public function rewind(): void {}

        /**
         * Returns the configured buffer size for streaming.
         *
         * @return int
         */
        public function getBatchSize(): int {}

        /**
         * Returns the current index (0-based position).
         *
         * @return int
         */
        public function key(): int {}

        /**
         * Returns the current row.
         *
         * Returns the row at the current iterator position, or null if
         * the position is invalid.
         *
         * @return mixed
         */
        public function current(): mixed {}

        /**
         * Returns the last error that occurred, if any.
         *
         * This is useful for checking if iteration stopped due to an error.
         *
         * @return string|null
         */
        public function getLastError(): ?string {}

        /**
         * Returns the number of rows fetched so far.
         *
         * Note: This returns the count of rows fetched, not the total
         * result set size (which may not be known until iteration completes).
         *
         * @return int
         */
        public function count(): int {}

        /**
         * Returns true if the current position is valid.
         *
         * @return bool
         */
        public function valid(): bool {}

        /**
         * Returns true if the result set has been fully consumed.
         *
         * @return bool
         */
        public function isExhausted(): bool {}

        public function __construct() {}
    }

    /**
     * A prepared SQL query builder.
     *
     * Holds the generated query string, parameters, and placeholder tracking
     * used during safe, composable query construction via AST rendering.
     */
    class MssqlReadQueryBuilder implements Sqlx\ReadQueryBuilderInterface {
        /**
         * @param \Sqlx\MssqlDriver $driver
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public static function factory(\Sqlx\MssqlDriver $driver): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * @param mixed $from
         * @param array|null $parameters
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function from(mixed $from, ?array $parameters = null): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * @param mixed $set
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function set(mixed $set): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `CROSS JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function crossJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `DELETE FROM` clause to the query.
         *
         * @param mixed $from A string table name or a subquery builder.
         * @param array|null $parameters Optional parameters if `from` is a raw string.
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function deleteFrom(mixed $from, ?array $parameters = null): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `FOR SHARE` locking clause to the query.
         *
         * This clause is used in `SELECT` statements to acquire shared locks
         * on the selected rows, allowing concurrent transactions to read but
         * not modify the rows until the current transaction completes.
         *
         * @return \Sqlx\MssqlReadQueryBuilder The query builder with `FOR SHARE` appended.
         */
        public function forShare(): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `FOR UPDATE` locking clause to the query.
         *
         * This clause is used in `SELECT` statements to lock the selected rows
         * for update, preventing other transactions from modifying or acquiring
         * locks on them until the current transaction completes.
         *
         * @return \Sqlx\MssqlReadQueryBuilder The query builder with `FOR UPDATE` appended.
         */
        public function forUpdate(): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `FULL OUTER JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function fullJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `GROUP BY` clause to the query.
         *
         * @param mixed $fields Either a raw string or a `ByClauseRendered` object.
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function groupBy(mixed $fields): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `HAVING` clause to the query.
         *
         * @param mixed $having Either a raw string, a structured array of conditions, or a disjunction (`OrClause`).
         * @param array|null $parameters Optional parameters associated with the `WHERE` condition.
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function having(mixed $having, ?array $parameters = null): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `LEFT JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function leftJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `LIMIT` (and optional `OFFSET`) clause to the query.
         *
         * @param int $limit – Maximum number of rows to return.
         * @param int|null $offset – Optional number of rows to skip before starting to return rows.
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function limit(int $limit, ?int $offset = null): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `NATURAL JOIN` clause to the query.
         *
         * @param string $table
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function naturalJoin(string $table): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `ORDER BY` clause to the query.
         *
         * @param mixed $fields Either a raw string or a `ByClauseRendered` object.
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function orderBy(mixed $fields): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `PAGINATE` clause to the query using a `PaginateClauseRendered` object.
         *
         * This expands into the appropriate SQL `LIMIT` and `OFFSET` syntax during rendering,
         * using the values stored in the given `PaginateClauseRendered` instance.
         *
         * @param mixed $paginate – An instance of `Sqlx\PaginateClauseRendered`, produced by invoking a `PaginateClause`
         * @return \Sqlx\MssqlReadQueryBuilder
         * @throws \Exception Returns an error if the argument is not an instance of `PaginateClauseRendered`.
         */
        public function paginate(mixed $paginate): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `RETURNING` clause to the query.
         *
         * @param mixed $fields A string or array of column names to return.
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function returning(mixed $fields): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `RIGHT JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function rightJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `SELECT` clause to the query.
         *
         * @param mixed $fields Either a raw string or a `SelectClauseRendered` object.
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function select(mixed $fields): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `SET` clause to the query, supporting both keyed and indexed formats.
         *
         * @param mixed $set An associative array mapping fields to values, or a sequential array
         * @param string $context
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function SetAssignments(mixed $set, string $context): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `TRUNCATE TABLE` statement to the query.
         *
         * This command removes all rows from the specified table quickly and efficiently.
         * It is faster than `DELETE FROM` and usually does not fire triggers or return affected row counts.
         *
         * @param string $table – The name of the table to truncate.
         * @return \Sqlx\MssqlReadQueryBuilder
         * @throws \Exception Returns an error if appending the SQL fragment fails (e.g., formatting error).
         */
        public function truncateTable(string $table): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `UNION ALL` clause to the query.
         *
         * @param mixed $query – A raw SQL string or another Builder instance (subquery).
         * @param array|null $parameters – Optional parameters to bind to the unioned subquery.
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function unionAll(mixed $query, ?array $parameters = null): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `UNION` clause to the query.
         *
         * @param mixed $query – A raw SQL string or another Builder instance (subquery).
         * @param array|null $parameters – Optional parameters to bind to the unioned subquery.
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function union(mixed $query, ?array $parameters = null): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `UPDATE` clause to the query.
         *
         * @param mixed $table A raw string representing the table(s).
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function update(mixed $table): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `USING` clause to the query.
         *
         * @param mixed $from Either a string table name or a subquery builder.
         * @param array|null $parameters Optional parameters if `from` is a string.
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function using(mixed $from, ?array $parameters = null): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `VALUES` clause to the query.
         *
         * @param mixed $values Can be:
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function values(mixed $values): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `WHERE` clause to the query.
         *
         * @param mixed $where_
         * @param array|null $parameters Optional parameters associated with the `WHERE` condition.
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function Where(mixed $where_, ?array $parameters = null): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `WITH RECURSIVE` clause to the query.
         *
         * @param string $table_and_fields Table name with field list, e.g. `cte(col1, col2)`.
         * @param mixed $as_
         * @param array|null $parameters Optional parameters for the recursive body.
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function withRecursive(string $table_and_fields, mixed $as_, ?array $parameters = null): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a `WITH` clause to the query.
         *
         * @param string $table Name of the CTE (common table expression).
         * @param mixed $as_
         * @param array|null $parameters Optional parameters for the query body.
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function with(string $table, mixed $as_, ?array $parameters = null): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends a raw SQL fragment to the query without structural validation.
         *
         * This method allows injecting raw SQL clauses directly into the query. It's intended
         * for advanced use cases such as vendor-specific syntax, subqueries, or complex expressions
         * not yet supported by the structured builder methods.
         *
         * @param string $part A raw SQL string to append to the query. It is inserted **verbatim** into the
         * @param array|null $parameters An optional map of named parameters to bind to placeholders within the SQL string.
         * @return \Sqlx\MssqlReadQueryBuilder Returns a mutable reference to the builder for fluent method chaining.
         */
        public function raw(string $part, ?array $parameters = null): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends an `INNER JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function innerJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends an `INNER JOIN` clause to the query.
         * Alias for `inner_join()`.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function join(string $table, string $on, ?array $parameters = null): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends an `INSERT INTO` clause to the query.
         *
         * @param string $table The name of the target table.
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function insertInto(string $table): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends an `OFFSET` clause to the query independently.
         *
         * @param int $offset – Number of rows to skip before returning results.
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function offset(int $offset): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends an `ON CONFLICT` clause to the query.
         *
         * @param mixed $target – A string or array of column names to specify the conflict target.
         * @param mixed $set – Optional `SET` clause. If `null`, generates `DO NOTHING`; otherwise uses the `SET` values.
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function onConflict(mixed $target, mixed $set = null): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends an `ON DUPLICATE KEY UPDATE` clause to the query (`MySQL`).
         *
         * @param mixed $set – An array representing fields and values to update.
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function onDuplicateKeyUpdate(mixed $set): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends an `REPLACE INTO` clause to the query.
         *
         * @param string $table The name of the target table.
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function replaceInto(string $table): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Appends multiple rows to the `VALUES` clause of an `INSERT` statement.
         *
         * Each row must be:
         * - an ordered list of values (indexed array),
         * - or a map of column names to values (associative array) — only for the first row, to infer column order.
         *
         * @param mixed $rows – A sequential array of rows (arrays of values).
         * @return \Sqlx\MssqlReadQueryBuilder
         */
        public function valuesMany(mixed $rows): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Creates a query builder object
         *
         * @return \Sqlx\MssqlReadQueryBuilder Query builder object
         */
        public function builder(): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Escapes `%` and `_` characters in a string for safe use in a LIKE/ILIKE pattern.
         *
         * This helper is designed for safely preparing user input for use with
         * pattern-matching operators like `CONTAINS`, `STARTS_WITH`, or `ENDS_WITH`.
         *
         * ⚠️ **Warning:** This method does **not** quote or escape the full string for raw SQL.
         * It only escapes `%` and `_` characters. You must still pass the result as a bound parameter,
         * not interpolate it directly into the query string.
         *
         * @param mixed $param – The parameter to escape (must be a string).
         * @return string A string with `%` and `_` escaped for LIKE (e.g., `foo\%bar\_baz`)
         * @throws \Exception Returns an error if the input is not a string.
         */
        public function metaQuoteLike(mixed $param): string {}

        /**
         * Executes a query and returns a grouped dictionary (Vec of rows per key).
         *
         * Same as [`queryGroupedDictionary`](crate::Driver::query_grouped_dictionary), but works on a prepared query.
         *
         * The first column is used as the key (must be scalar),
         * and each resulting row is appended to the corresponding key's Vec.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Fails if the query fails, or the first column is not scalar.
         */
        public function queryGroupedDictionary(?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a single result, or `null` if no row matched.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed Single row as array or object depending on config
         */
        public function queryMaybeRow(?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a single row as a PHP object, or `null` if no row matched.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed The result row as a `stdClass` PHP object, or `null` if no matching row is found.
         */
        public function queryMaybeRowObj(?array $parameters = null): mixed {}

        /**
         * Executes the SQL query and returns a single row as a PHP associative array, or `null` if no row matched.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed The result row as an associative array (`array<string, mixed>` in PHP), or `null` if no matching row is found.
         */
        public function queryMaybeRowAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the SQL query and returns the specified column values from all result rows.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param mixed $column Optional column name or index to extract.
         * @return array An array of column values, one for each row.
         */
        public function queryColumn(?array $parameters = null, mixed $column = null): array {}

        /**
         * Executes the SQL query and returns the specified column values from all rows in associative array mode.
         *
         * @param array|null $parameters Optional named parameters.
         * @param mixed $column Column index or name to extract.
         * @return array An array of column values (associative arrays for structured data).
         */
        public function queryColumnAssoc(?array $parameters = null, mixed $column = null): array {}

        /**
         * Executes the SQL query and returns the specified column values from all rows in object mode.
         *
         * @param array|null $parameters Optional named parameters.
         * @param mixed $column Column index or name to extract.
         * @return array An array of column values (objects for structured data).
         */
        public function queryColumnObj(?array $parameters = null, mixed $column = null): array {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an associative array.
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row as an associative array.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an object (`stdClass`).
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row as an object.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row.
         *
         * The result is a `HashMap` where the key is the stringified first column from each row,
         * and the value is the full row, returned as array or object depending on config.
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in associative array mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as associative arrays.
         *
         * @param array|null $parameters
         * @return mixed A dictionary where each key is the first column (as string), and each value is the second column as an associative PHP array.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in object mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as objects.
         *
         * @param array|null $parameters
         * @return mixed A dictionary where each key is the first column (as string), and each value is the second column as a PHP object.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary mapping the first column to the second column.
         *
         * This method expects each result row to contain at least two columns. It converts the first column
         * into a PHP string (used as the key), and the second column into a PHP value (used as the value).
         *
         * @param array|null $parameters
         * @return mixed An associative array (`array<string, mixed>`) where each key is the first column (as string), and the value is the second column.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a PHP string;
         * @throws \Exception - the second column cannot be converted to a PHP value.
         */
        public function queryColumnDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a grouped dictionary where:
         * - the key is the **first column** (must be scalar),
         * - the value is a list of values from the **second column** for each group.
         *
         * This variant uses the driver's default associative array option for JSON values.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a single result.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed Single row as array or object depending on config
         */
        public function queryRow(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns all rows as associative arrays.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array
         */
        public function queryAllAssoc(?array $parameters = null): array {}

        /**
         * Executes the prepared query and returns all rows as objects.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array
         */
        public function queryAllObj(?array $parameters = null): array {}

        /**
         * Executes the prepared query and returns all rows.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array Array of rows as array or object depending on config
         */
        public function queryAll(?array $parameters = null): array {}

        /**
         * Executes the prepared query and returns one row as an associative array.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed
         */
        public function queryRowAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns one row as an object.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed
         */
        public function queryRowObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query with optional parameters.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return int Number of affected rows
         */
        public function execute(?array $parameters = null): int {}

        /**
         * Executes the query and returns a lazy `QueryResult` iterator with rows as associative arrays.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\MssqlQueryResult A `QueryResult` object with rows as associative arrays
         */
        public function queryAssoc(?array $parameters = null, ?int $batch_size = null): \Sqlx\MssqlQueryResult {}

        /**
         * Executes the query and returns a lazy `QueryResult` iterator with rows as objects.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\MssqlQueryResult A `QueryResult` object with rows as PHP objects
         */
        public function queryObj(?array $parameters = null, ?int $batch_size = null): \Sqlx\MssqlQueryResult {}

        /**
         * Executes the query and returns a lazy `QueryResult` iterator.
         *
         * This method returns a `QueryResult` object that implements PHP's `Iterator`
         * interface, streaming rows from the database as you iterate.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\MssqlQueryResult A `QueryResult` object implementing `Iterator`
         */
        public function query(?array $parameters = null, ?int $batch_size = null): \Sqlx\MssqlQueryResult {}

        /**
         * Finalizes the query by appending a semicolon (`;`).
         *
         * This method is optional. Most databases do not require semicolons in prepared queries,
         * but you may use it to explicitly terminate a query string.
         *
         * @param string $_table
         * @return \Sqlx\MssqlReadQueryBuilder The builder instance after appending the semicolon.
         */
        public function end(string $_table): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Merges additional parameters with the builder's accumulated parameters.
         *
         * Parameters passed directly to the method take precedence over builder parameters.
         *
         * @param array|null $parameters
         * @return array|null
         */
        public function mergeParameters(?array $parameters = null): ?array {}

        /**
         * Quotes a single scalar value for safe embedding into SQL.
         *
         * This method renders the given `ParameterValue` into a properly escaped SQL literal,
         * using the driver's configuration (e.g., quoting style, encoding).
         *
         * ⚠️ **Warning:** Prefer using placeholders and parameter binding wherever possible.
         * This method should only be used for debugging or generating static fragments,
         * not for constructing dynamic SQL with user input.
         *
         * @param mixed $param – The parameter to quote (must be a scalar: string, number, or boolean).
         * @return string Quoted SQL string (e.g., `'abc'`, `123`, `TRUE`)
         * @throws \Exception Returns an error if the parameter is not a scalar or if rendering fails.
         */
        public function quote(mixed $param): string {}

        /**
         * Renders the SQL query using named parameters and returns the fully rendered SQL with values injected inline.
         *
         * @return array The rendered SQL query as a string with all parameters interpolated.
         */
        public function dry(): array {}

        /**
         * Returns an array of all currently accumulated parameters.
         *
         * @return array
         */
        public function parameters(): array {}

        /**
         * Returns the fully rendered SQL query with parameters embedded as literals.
         *
         * Used for debugging or fallback rendering when the placeholder limit is exceeded.
         *
         * @return string A string representing the complete SQL statement.
         */
        public function __toString(): string {}

        /**
         * Returns the parameter map currently accumulated in the builder.
         *
         * @return string A cloned map of all parameter names and their corresponding `ParameterValue`.
         */
        public function dryInline(): string {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces PHP objects
         * for the second column if it contains JSON objects.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces associative arrays
         * for the second column if it contains JSON objects.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as PHP objects.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as associative arrays.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedDictionaryAssoc(?array $parameters = null): mixed {}

        public function __construct() {}
    }

    /**
     * A prepared SQL query builder.
     *
     * Holds the generated query string, parameters, and placeholder tracking
     * used during safe, composable query construction via AST rendering.
     */
    class MssqlWriteQueryBuilder implements Sqlx\WriteQueryBuilderInterface {
        /**
         * @param \Sqlx\MssqlDriver $driver
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public static function factory(\Sqlx\MssqlDriver $driver): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * @param mixed $from
         * @param array|null $parameters
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function from(mixed $from, ?array $parameters = null): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * @param mixed $set
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function set(mixed $set): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `CROSS JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function crossJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `DELETE FROM` clause to the query.
         *
         * @param mixed $from A string table name or a subquery builder.
         * @param array|null $parameters Optional parameters if `from` is a raw string.
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function deleteFrom(mixed $from, ?array $parameters = null): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `FOR SHARE` locking clause to the query.
         *
         * This clause is used in `SELECT` statements to acquire shared locks
         * on the selected rows, allowing concurrent transactions to read but
         * not modify the rows until the current transaction completes.
         *
         * @return \Sqlx\MssqlWriteQueryBuilder The query builder with `FOR SHARE` appended.
         */
        public function forShare(): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `FOR UPDATE` locking clause to the query.
         *
         * This clause is used in `SELECT` statements to lock the selected rows
         * for update, preventing other transactions from modifying or acquiring
         * locks on them until the current transaction completes.
         *
         * @return \Sqlx\MssqlWriteQueryBuilder The query builder with `FOR UPDATE` appended.
         */
        public function forUpdate(): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `FULL OUTER JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function fullJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `GROUP BY` clause to the query.
         *
         * @param mixed $fields Either a raw string or a `ByClauseRendered` object.
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function groupBy(mixed $fields): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `HAVING` clause to the query.
         *
         * @param mixed $having Either a raw string, a structured array of conditions, or a disjunction (`OrClause`).
         * @param array|null $parameters Optional parameters associated with the `WHERE` condition.
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function having(mixed $having, ?array $parameters = null): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `LEFT JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function leftJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `LIMIT` (and optional `OFFSET`) clause to the query.
         *
         * @param int $limit – Maximum number of rows to return.
         * @param int|null $offset – Optional number of rows to skip before starting to return rows.
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function limit(int $limit, ?int $offset = null): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `NATURAL JOIN` clause to the query.
         *
         * @param string $table
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function naturalJoin(string $table): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `ORDER BY` clause to the query.
         *
         * @param mixed $fields Either a raw string or a `ByClauseRendered` object.
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function orderBy(mixed $fields): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `PAGINATE` clause to the query using a `PaginateClauseRendered` object.
         *
         * This expands into the appropriate SQL `LIMIT` and `OFFSET` syntax during rendering,
         * using the values stored in the given `PaginateClauseRendered` instance.
         *
         * @param mixed $paginate – An instance of `Sqlx\PaginateClauseRendered`, produced by invoking a `PaginateClause`
         * @return \Sqlx\MssqlWriteQueryBuilder
         * @throws \Exception Returns an error if the argument is not an instance of `PaginateClauseRendered`.
         */
        public function paginate(mixed $paginate): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `RETURNING` clause to the query.
         *
         * @param mixed $fields A string or array of column names to return.
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function returning(mixed $fields): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `RIGHT JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function rightJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `SELECT` clause to the query.
         *
         * @param mixed $fields Either a raw string or a `SelectClauseRendered` object.
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function select(mixed $fields): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `SET` clause to the query, supporting both keyed and indexed formats.
         *
         * @param mixed $set An associative array mapping fields to values, or a sequential array
         * @param string $context
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function SetAssignments(mixed $set, string $context): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `TRUNCATE TABLE` statement to the query.
         *
         * This command removes all rows from the specified table quickly and efficiently.
         * It is faster than `DELETE FROM` and usually does not fire triggers or return affected row counts.
         *
         * @param string $table – The name of the table to truncate.
         * @return \Sqlx\MssqlWriteQueryBuilder
         * @throws \Exception Returns an error if appending the SQL fragment fails (e.g., formatting error).
         */
        public function truncateTable(string $table): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `UNION ALL` clause to the query.
         *
         * @param mixed $query – A raw SQL string or another Builder instance (subquery).
         * @param array|null $parameters – Optional parameters to bind to the unioned subquery.
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function unionAll(mixed $query, ?array $parameters = null): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `UNION` clause to the query.
         *
         * @param mixed $query – A raw SQL string or another Builder instance (subquery).
         * @param array|null $parameters – Optional parameters to bind to the unioned subquery.
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function union(mixed $query, ?array $parameters = null): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `UPDATE` clause to the query.
         *
         * @param mixed $table A raw string representing the table(s).
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function update(mixed $table): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `USING` clause to the query.
         *
         * @param mixed $from Either a string table name or a subquery builder.
         * @param array|null $parameters Optional parameters if `from` is a string.
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function using(mixed $from, ?array $parameters = null): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `VALUES` clause to the query.
         *
         * @param mixed $values Can be:
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function values(mixed $values): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `WHERE` clause to the query.
         *
         * @param mixed $where_
         * @param array|null $parameters Optional parameters associated with the `WHERE` condition.
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function Where(mixed $where_, ?array $parameters = null): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `WITH RECURSIVE` clause to the query.
         *
         * @param string $table_and_fields Table name with field list, e.g. `cte(col1, col2)`.
         * @param mixed $as_
         * @param array|null $parameters Optional parameters for the recursive body.
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function withRecursive(string $table_and_fields, mixed $as_, ?array $parameters = null): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a `WITH` clause to the query.
         *
         * @param string $table Name of the CTE (common table expression).
         * @param mixed $as_
         * @param array|null $parameters Optional parameters for the query body.
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function with(string $table, mixed $as_, ?array $parameters = null): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends a raw SQL fragment to the query without structural validation.
         *
         * This method allows injecting raw SQL clauses directly into the query. It's intended
         * for advanced use cases such as vendor-specific syntax, subqueries, or complex expressions
         * not yet supported by the structured builder methods.
         *
         * @param string $part A raw SQL string to append to the query. It is inserted **verbatim** into the
         * @param array|null $parameters An optional map of named parameters to bind to placeholders within the SQL string.
         * @return \Sqlx\MssqlWriteQueryBuilder Returns a mutable reference to the builder for fluent method chaining.
         */
        public function raw(string $part, ?array $parameters = null): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends an `INNER JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function innerJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends an `INNER JOIN` clause to the query.
         * Alias for `inner_join()`.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function join(string $table, string $on, ?array $parameters = null): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends an `INSERT INTO` clause to the query.
         *
         * @param string $table The name of the target table.
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function insertInto(string $table): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends an `OFFSET` clause to the query independently.
         *
         * @param int $offset – Number of rows to skip before returning results.
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function offset(int $offset): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends an `ON CONFLICT` clause to the query.
         *
         * @param mixed $target – A string or array of column names to specify the conflict target.
         * @param mixed $set – Optional `SET` clause. If `null`, generates `DO NOTHING`; otherwise uses the `SET` values.
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function onConflict(mixed $target, mixed $set = null): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends an `ON DUPLICATE KEY UPDATE` clause to the query (`MySQL`).
         *
         * @param mixed $set – An array representing fields and values to update.
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function onDuplicateKeyUpdate(mixed $set): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends an `REPLACE INTO` clause to the query.
         *
         * @param string $table The name of the target table.
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function replaceInto(string $table): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Appends multiple rows to the `VALUES` clause of an `INSERT` statement.
         *
         * Each row must be:
         * - an ordered list of values (indexed array),
         * - or a map of column names to values (associative array) — only for the first row, to infer column order.
         *
         * @param mixed $rows – A sequential array of rows (arrays of values).
         * @return \Sqlx\MssqlWriteQueryBuilder
         */
        public function valuesMany(mixed $rows): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Creates a query builder object
         *
         * @return \Sqlx\MssqlWriteQueryBuilder Query builder object
         */
        public function builder(): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Escapes `%` and `_` characters in a string for safe use in a LIKE/ILIKE pattern.
         *
         * This helper is designed for safely preparing user input for use with
         * pattern-matching operators like `CONTAINS`, `STARTS_WITH`, or `ENDS_WITH`.
         *
         * ⚠️ **Warning:** This method does **not** quote or escape the full string for raw SQL.
         * It only escapes `%` and `_` characters. You must still pass the result as a bound parameter,
         * not interpolate it directly into the query string.
         *
         * @param mixed $param – The parameter to escape (must be a string).
         * @return string A string with `%` and `_` escaped for LIKE (e.g., `foo\%bar\_baz`)
         * @throws \Exception Returns an error if the input is not a string.
         */
        public function metaQuoteLike(mixed $param): string {}

        /**
         * Executes a query and returns a grouped dictionary (Vec of rows per key).
         *
         * Same as [`queryGroupedDictionary`](crate::Driver::query_grouped_dictionary), but works on a prepared query.
         *
         * The first column is used as the key (must be scalar),
         * and each resulting row is appended to the corresponding key's Vec.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Fails if the query fails, or the first column is not scalar.
         */
        public function queryGroupedDictionary(?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a single result, or `null` if no row matched.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed Single row as array or object depending on config
         */
        public function queryMaybeRow(?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a single row as a PHP object, or `null` if no row matched.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed The result row as a `stdClass` PHP object, or `null` if no matching row is found.
         */
        public function queryMaybeRowObj(?array $parameters = null): mixed {}

        /**
         * Executes the SQL query and returns a single row as a PHP associative array, or `null` if no row matched.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed The result row as an associative array (`array<string, mixed>` in PHP), or `null` if no matching row is found.
         */
        public function queryMaybeRowAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the SQL query and returns the specified column values from all result rows.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param mixed $column Optional column name or index to extract.
         * @return array An array of column values, one for each row.
         */
        public function queryColumn(?array $parameters = null, mixed $column = null): array {}

        /**
         * Executes the SQL query and returns the specified column values from all rows in associative array mode.
         *
         * @param array|null $parameters Optional named parameters.
         * @param mixed $column Column index or name to extract.
         * @return array An array of column values (associative arrays for structured data).
         */
        public function queryColumnAssoc(?array $parameters = null, mixed $column = null): array {}

        /**
         * Executes the SQL query and returns the specified column values from all rows in object mode.
         *
         * @param array|null $parameters Optional named parameters.
         * @param mixed $column Column index or name to extract.
         * @return array An array of column values (objects for structured data).
         */
        public function queryColumnObj(?array $parameters = null, mixed $column = null): array {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an associative array.
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row as an associative array.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an object (`stdClass`).
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row as an object.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row.
         *
         * The result is a `HashMap` where the key is the stringified first column from each row,
         * and the value is the full row, returned as array or object depending on config.
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in associative array mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as associative arrays.
         *
         * @param array|null $parameters
         * @return mixed A dictionary where each key is the first column (as string), and each value is the second column as an associative PHP array.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in object mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as objects.
         *
         * @param array|null $parameters
         * @return mixed A dictionary where each key is the first column (as string), and each value is the second column as a PHP object.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary mapping the first column to the second column.
         *
         * This method expects each result row to contain at least two columns. It converts the first column
         * into a PHP string (used as the key), and the second column into a PHP value (used as the value).
         *
         * @param array|null $parameters
         * @return mixed An associative array (`array<string, mixed>`) where each key is the first column (as string), and the value is the second column.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a PHP string;
         * @throws \Exception - the second column cannot be converted to a PHP value.
         */
        public function queryColumnDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a grouped dictionary where:
         * - the key is the **first column** (must be scalar),
         * - the value is a list of values from the **second column** for each group.
         *
         * This variant uses the driver's default associative array option for JSON values.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a single result.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed Single row as array or object depending on config
         */
        public function queryRow(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns all rows as associative arrays.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array
         */
        public function queryAllAssoc(?array $parameters = null): array {}

        /**
         * Executes the prepared query and returns all rows as objects.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array
         */
        public function queryAllObj(?array $parameters = null): array {}

        /**
         * Executes the prepared query and returns all rows.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array Array of rows as array or object depending on config
         */
        public function queryAll(?array $parameters = null): array {}

        /**
         * Executes the prepared query and returns one row as an associative array.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed
         */
        public function queryRowAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns one row as an object.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed
         */
        public function queryRowObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query with optional parameters.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return int Number of affected rows
         */
        public function execute(?array $parameters = null): int {}

        /**
         * Executes the query and returns a lazy `QueryResult` iterator with rows as associative arrays.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\MssqlQueryResult A `QueryResult` object with rows as associative arrays
         */
        public function queryAssoc(?array $parameters = null, ?int $batch_size = null): \Sqlx\MssqlQueryResult {}

        /**
         * Executes the query and returns a lazy `QueryResult` iterator with rows as objects.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\MssqlQueryResult A `QueryResult` object with rows as PHP objects
         */
        public function queryObj(?array $parameters = null, ?int $batch_size = null): \Sqlx\MssqlQueryResult {}

        /**
         * Executes the query and returns a lazy `QueryResult` iterator.
         *
         * This method returns a `QueryResult` object that implements PHP's `Iterator`
         * interface, streaming rows from the database as you iterate.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\MssqlQueryResult A `QueryResult` object implementing `Iterator`
         */
        public function query(?array $parameters = null, ?int $batch_size = null): \Sqlx\MssqlQueryResult {}

        /**
         * Finalizes the query by appending a semicolon (`;`).
         *
         * This method is optional. Most databases do not require semicolons in prepared queries,
         * but you may use it to explicitly terminate a query string.
         *
         * @param string $_table
         * @return \Sqlx\MssqlWriteQueryBuilder The builder instance after appending the semicolon.
         */
        public function end(string $_table): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Merges additional parameters with the builder's accumulated parameters.
         *
         * Parameters passed directly to the method take precedence over builder parameters.
         *
         * @param array|null $parameters
         * @return array|null
         */
        public function mergeParameters(?array $parameters = null): ?array {}

        /**
         * Quotes a single scalar value for safe embedding into SQL.
         *
         * This method renders the given `ParameterValue` into a properly escaped SQL literal,
         * using the driver's configuration (e.g., quoting style, encoding).
         *
         * ⚠️ **Warning:** Prefer using placeholders and parameter binding wherever possible.
         * This method should only be used for debugging or generating static fragments,
         * not for constructing dynamic SQL with user input.
         *
         * @param mixed $param – The parameter to quote (must be a scalar: string, number, or boolean).
         * @return string Quoted SQL string (e.g., `'abc'`, `123`, `TRUE`)
         * @throws \Exception Returns an error if the parameter is not a scalar or if rendering fails.
         */
        public function quote(mixed $param): string {}

        /**
         * Renders the SQL query using named parameters and returns the fully rendered SQL with values injected inline.
         *
         * @return array The rendered SQL query as a string with all parameters interpolated.
         */
        public function dry(): array {}

        /**
         * Returns an array of all currently accumulated parameters.
         *
         * @return array
         */
        public function parameters(): array {}

        /**
         * Returns the fully rendered SQL query with parameters embedded as literals.
         *
         * Used for debugging or fallback rendering when the placeholder limit is exceeded.
         *
         * @return string A string representing the complete SQL statement.
         */
        public function __toString(): string {}

        /**
         * Returns the parameter map currently accumulated in the builder.
         *
         * @return string A cloned map of all parameter names and their corresponding `ParameterValue`.
         */
        public function dryInline(): string {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces PHP objects
         * for the second column if it contains JSON objects.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces associative arrays
         * for the second column if it contains JSON objects.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as PHP objects.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as associative arrays.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedDictionaryAssoc(?array $parameters = null): mixed {}

        public function __construct() {}
    }

    /**
     * A prepared SQL query builder.
     *
     * Holds the generated query string, parameters, and placeholder tracking
     * used during safe, composable query construction via AST rendering.
     */
    class MySqlReadQueryBuilder implements Sqlx\ReadQueryBuilderInterface {
        /**
         * @param \Sqlx\MySqlDriver $driver
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public static function factory(\Sqlx\MySqlDriver $driver): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * @param mixed $from
         * @param array|null $parameters
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function from(mixed $from, ?array $parameters = null): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * @param mixed $set
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function set(mixed $set): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `CROSS JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function crossJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `DELETE FROM` clause to the query.
         *
         * @param mixed $from A string table name or a subquery builder.
         * @param array|null $parameters Optional parameters if `from` is a raw string.
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function deleteFrom(mixed $from, ?array $parameters = null): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `FOR SHARE` locking clause to the query.
         *
         * This clause is used in `SELECT` statements to acquire shared locks
         * on the selected rows, allowing concurrent transactions to read but
         * not modify the rows until the current transaction completes.
         *
         * @return \Sqlx\MySqlReadQueryBuilder The query builder with `FOR SHARE` appended.
         */
        public function forShare(): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `FOR UPDATE` locking clause to the query.
         *
         * This clause is used in `SELECT` statements to lock the selected rows
         * for update, preventing other transactions from modifying or acquiring
         * locks on them until the current transaction completes.
         *
         * @return \Sqlx\MySqlReadQueryBuilder The query builder with `FOR UPDATE` appended.
         */
        public function forUpdate(): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `FULL OUTER JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function fullJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `GROUP BY` clause to the query.
         *
         * @param mixed $fields Either a raw string or a `ByClauseRendered` object.
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function groupBy(mixed $fields): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `HAVING` clause to the query.
         *
         * @param mixed $having Either a raw string, a structured array of conditions, or a disjunction (`OrClause`).
         * @param array|null $parameters Optional parameters associated with the `WHERE` condition.
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function having(mixed $having, ?array $parameters = null): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `LEFT JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function leftJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `LIMIT` (and optional `OFFSET`) clause to the query.
         *
         * @param int $limit – Maximum number of rows to return.
         * @param int|null $offset – Optional number of rows to skip before starting to return rows.
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function limit(int $limit, ?int $offset = null): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `NATURAL JOIN` clause to the query.
         *
         * @param string $table
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function naturalJoin(string $table): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `ORDER BY` clause to the query.
         *
         * @param mixed $fields Either a raw string or a `ByClauseRendered` object.
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function orderBy(mixed $fields): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `PAGINATE` clause to the query using a `PaginateClauseRendered` object.
         *
         * This expands into the appropriate SQL `LIMIT` and `OFFSET` syntax during rendering,
         * using the values stored in the given `PaginateClauseRendered` instance.
         *
         * @param mixed $paginate – An instance of `Sqlx\PaginateClauseRendered`, produced by invoking a `PaginateClause`
         * @return \Sqlx\MySqlReadQueryBuilder
         * @throws \Exception Returns an error if the argument is not an instance of `PaginateClauseRendered`.
         */
        public function paginate(mixed $paginate): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `RETURNING` clause to the query.
         *
         * @param mixed $fields A string or array of column names to return.
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function returning(mixed $fields): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `RIGHT JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function rightJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `SELECT` clause to the query.
         *
         * @param mixed $fields Either a raw string or a `SelectClauseRendered` object.
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function select(mixed $fields): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `SET` clause to the query, supporting both keyed and indexed formats.
         *
         * @param mixed $set An associative array mapping fields to values, or a sequential array
         * @param string $context
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function SetAssignments(mixed $set, string $context): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `TRUNCATE TABLE` statement to the query.
         *
         * This command removes all rows from the specified table quickly and efficiently.
         * It is faster than `DELETE FROM` and usually does not fire triggers or return affected row counts.
         *
         * @param string $table – The name of the table to truncate.
         * @return \Sqlx\MySqlReadQueryBuilder
         * @throws \Exception Returns an error if appending the SQL fragment fails (e.g., formatting error).
         */
        public function truncateTable(string $table): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `UNION ALL` clause to the query.
         *
         * @param mixed $query – A raw SQL string or another Builder instance (subquery).
         * @param array|null $parameters – Optional parameters to bind to the unioned subquery.
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function unionAll(mixed $query, ?array $parameters = null): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `UNION` clause to the query.
         *
         * @param mixed $query – A raw SQL string or another Builder instance (subquery).
         * @param array|null $parameters – Optional parameters to bind to the unioned subquery.
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function union(mixed $query, ?array $parameters = null): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `UPDATE` clause to the query.
         *
         * @param mixed $table A raw string representing the table(s).
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function update(mixed $table): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `USING` clause to the query.
         *
         * @param mixed $from Either a string table name or a subquery builder.
         * @param array|null $parameters Optional parameters if `from` is a string.
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function using(mixed $from, ?array $parameters = null): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `VALUES` clause to the query.
         *
         * @param mixed $values Can be:
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function values(mixed $values): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `WHERE` clause to the query.
         *
         * @param mixed $where_
         * @param array|null $parameters Optional parameters associated with the `WHERE` condition.
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function Where(mixed $where_, ?array $parameters = null): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `WITH RECURSIVE` clause to the query.
         *
         * @param string $table_and_fields Table name with field list, e.g. `cte(col1, col2)`.
         * @param mixed $as_
         * @param array|null $parameters Optional parameters for the recursive body.
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function withRecursive(string $table_and_fields, mixed $as_, ?array $parameters = null): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a `WITH` clause to the query.
         *
         * @param string $table Name of the CTE (common table expression).
         * @param mixed $as_
         * @param array|null $parameters Optional parameters for the query body.
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function with(string $table, mixed $as_, ?array $parameters = null): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends a raw SQL fragment to the query without structural validation.
         *
         * This method allows injecting raw SQL clauses directly into the query. It's intended
         * for advanced use cases such as vendor-specific syntax, subqueries, or complex expressions
         * not yet supported by the structured builder methods.
         *
         * @param string $part A raw SQL string to append to the query. It is inserted **verbatim** into the
         * @param array|null $parameters An optional map of named parameters to bind to placeholders within the SQL string.
         * @return \Sqlx\MySqlReadQueryBuilder Returns a mutable reference to the builder for fluent method chaining.
         */
        public function raw(string $part, ?array $parameters = null): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends an `INNER JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function innerJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends an `INNER JOIN` clause to the query.
         * Alias for `inner_join()`.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function join(string $table, string $on, ?array $parameters = null): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends an `INSERT INTO` clause to the query.
         *
         * @param string $table The name of the target table.
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function insertInto(string $table): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends an `OFFSET` clause to the query independently.
         *
         * @param int $offset – Number of rows to skip before returning results.
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function offset(int $offset): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends an `ON CONFLICT` clause to the query.
         *
         * @param mixed $target – A string or array of column names to specify the conflict target.
         * @param mixed $set – Optional `SET` clause. If `null`, generates `DO NOTHING`; otherwise uses the `SET` values.
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function onConflict(mixed $target, mixed $set = null): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends an `ON DUPLICATE KEY UPDATE` clause to the query (`MySQL`).
         *
         * @param mixed $set – An array representing fields and values to update.
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function onDuplicateKeyUpdate(mixed $set): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends an `REPLACE INTO` clause to the query.
         *
         * @param string $table The name of the target table.
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function replaceInto(string $table): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Appends multiple rows to the `VALUES` clause of an `INSERT` statement.
         *
         * Each row must be:
         * - an ordered list of values (indexed array),
         * - or a map of column names to values (associative array) — only for the first row, to infer column order.
         *
         * @param mixed $rows – A sequential array of rows (arrays of values).
         * @return \Sqlx\MySqlReadQueryBuilder
         */
        public function valuesMany(mixed $rows): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Creates a query builder object
         *
         * @return \Sqlx\MySqlReadQueryBuilder Query builder object
         */
        public function builder(): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Escapes `%` and `_` characters in a string for safe use in a LIKE/ILIKE pattern.
         *
         * This helper is designed for safely preparing user input for use with
         * pattern-matching operators like `CONTAINS`, `STARTS_WITH`, or `ENDS_WITH`.
         *
         * ⚠️ **Warning:** This method does **not** quote or escape the full string for raw SQL.
         * It only escapes `%` and `_` characters. You must still pass the result as a bound parameter,
         * not interpolate it directly into the query string.
         *
         * @param mixed $param – The parameter to escape (must be a string).
         * @return string A string with `%` and `_` escaped for LIKE (e.g., `foo\%bar\_baz`)
         * @throws \Exception Returns an error if the input is not a string.
         */
        public function metaQuoteLike(mixed $param): string {}

        /**
         * Executes a query and returns a grouped dictionary (Vec of rows per key).
         *
         * Same as [`queryGroupedDictionary`](crate::Driver::query_grouped_dictionary), but works on a prepared query.
         *
         * The first column is used as the key (must be scalar),
         * and each resulting row is appended to the corresponding key's Vec.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Fails if the query fails, or the first column is not scalar.
         */
        public function queryGroupedDictionary(?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a single result, or `null` if no row matched.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed Single row as array or object depending on config
         */
        public function queryMaybeRow(?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a single row as a PHP object, or `null` if no row matched.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed The result row as a `stdClass` PHP object, or `null` if no matching row is found.
         */
        public function queryMaybeRowObj(?array $parameters = null): mixed {}

        /**
         * Executes the SQL query and returns a single row as a PHP associative array, or `null` if no row matched.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed The result row as an associative array (`array<string, mixed>` in PHP), or `null` if no matching row is found.
         */
        public function queryMaybeRowAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the SQL query and returns the specified column values from all result rows.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param mixed $column Optional column name or index to extract.
         * @return array An array of column values, one for each row.
         */
        public function queryColumn(?array $parameters = null, mixed $column = null): array {}

        /**
         * Executes the SQL query and returns the specified column values from all rows in associative array mode.
         *
         * @param array|null $parameters Optional named parameters.
         * @param mixed $column Column index or name to extract.
         * @return array An array of column values (associative arrays for structured data).
         */
        public function queryColumnAssoc(?array $parameters = null, mixed $column = null): array {}

        /**
         * Executes the SQL query and returns the specified column values from all rows in object mode.
         *
         * @param array|null $parameters Optional named parameters.
         * @param mixed $column Column index or name to extract.
         * @return array An array of column values (objects for structured data).
         */
        public function queryColumnObj(?array $parameters = null, mixed $column = null): array {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an associative array.
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row as an associative array.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an object (`stdClass`).
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row as an object.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row.
         *
         * The result is a `HashMap` where the key is the stringified first column from each row,
         * and the value is the full row, returned as array or object depending on config.
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in associative array mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as associative arrays.
         *
         * @param array|null $parameters
         * @return mixed A dictionary where each key is the first column (as string), and each value is the second column as an associative PHP array.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in object mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as objects.
         *
         * @param array|null $parameters
         * @return mixed A dictionary where each key is the first column (as string), and each value is the second column as a PHP object.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary mapping the first column to the second column.
         *
         * This method expects each result row to contain at least two columns. It converts the first column
         * into a PHP string (used as the key), and the second column into a PHP value (used as the value).
         *
         * @param array|null $parameters
         * @return mixed An associative array (`array<string, mixed>`) where each key is the first column (as string), and the value is the second column.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a PHP string;
         * @throws \Exception - the second column cannot be converted to a PHP value.
         */
        public function queryColumnDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a grouped dictionary where:
         * - the key is the **first column** (must be scalar),
         * - the value is a list of values from the **second column** for each group.
         *
         * This variant uses the driver's default associative array option for JSON values.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a single result.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed Single row as array or object depending on config
         */
        public function queryRow(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns all rows as associative arrays.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array
         */
        public function queryAllAssoc(?array $parameters = null): array {}

        /**
         * Executes the prepared query and returns all rows as objects.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array
         */
        public function queryAllObj(?array $parameters = null): array {}

        /**
         * Executes the prepared query and returns all rows.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array Array of rows as array or object depending on config
         */
        public function queryAll(?array $parameters = null): array {}

        /**
         * Executes the prepared query and returns one row as an associative array.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed
         */
        public function queryRowAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns one row as an object.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed
         */
        public function queryRowObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query with optional parameters.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return int Number of affected rows
         */
        public function execute(?array $parameters = null): int {}

        /**
         * Executes the query and returns a lazy `QueryResult` iterator with rows as associative arrays.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\MySqlQueryResult A `QueryResult` object with rows as associative arrays
         */
        public function queryAssoc(?array $parameters = null, ?int $batch_size = null): \Sqlx\MySqlQueryResult {}

        /**
         * Executes the query and returns a lazy `QueryResult` iterator with rows as objects.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\MySqlQueryResult A `QueryResult` object with rows as PHP objects
         */
        public function queryObj(?array $parameters = null, ?int $batch_size = null): \Sqlx\MySqlQueryResult {}

        /**
         * Executes the query and returns a lazy `QueryResult` iterator.
         *
         * This method returns a `QueryResult` object that implements PHP's `Iterator`
         * interface, streaming rows from the database as you iterate.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\MySqlQueryResult A `QueryResult` object implementing `Iterator`
         */
        public function query(?array $parameters = null, ?int $batch_size = null): \Sqlx\MySqlQueryResult {}

        /**
         * Finalizes the query by appending a semicolon (`;`).
         *
         * This method is optional. Most databases do not require semicolons in prepared queries,
         * but you may use it to explicitly terminate a query string.
         *
         * @param string $_table
         * @return \Sqlx\MySqlReadQueryBuilder The builder instance after appending the semicolon.
         */
        public function end(string $_table): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Merges additional parameters with the builder's accumulated parameters.
         *
         * Parameters passed directly to the method take precedence over builder parameters.
         *
         * @param array|null $parameters
         * @return array|null
         */
        public function mergeParameters(?array $parameters = null): ?array {}

        /**
         * Quotes a single scalar value for safe embedding into SQL.
         *
         * This method renders the given `ParameterValue` into a properly escaped SQL literal,
         * using the driver's configuration (e.g., quoting style, encoding).
         *
         * ⚠️ **Warning:** Prefer using placeholders and parameter binding wherever possible.
         * This method should only be used for debugging or generating static fragments,
         * not for constructing dynamic SQL with user input.
         *
         * @param mixed $param – The parameter to quote (must be a scalar: string, number, or boolean).
         * @return string Quoted SQL string (e.g., `'abc'`, `123`, `TRUE`)
         * @throws \Exception Returns an error if the parameter is not a scalar or if rendering fails.
         */
        public function quote(mixed $param): string {}

        /**
         * Renders the SQL query using named parameters and returns the fully rendered SQL with values injected inline.
         *
         * @return array The rendered SQL query as a string with all parameters interpolated.
         */
        public function dry(): array {}

        /**
         * Returns an array of all currently accumulated parameters.
         *
         * @return array
         */
        public function parameters(): array {}

        /**
         * Returns the fully rendered SQL query with parameters embedded as literals.
         *
         * Used for debugging or fallback rendering when the placeholder limit is exceeded.
         *
         * @return string A string representing the complete SQL statement.
         */
        public function __toString(): string {}

        /**
         * Returns the parameter map currently accumulated in the builder.
         *
         * @return string A cloned map of all parameter names and their corresponding `ParameterValue`.
         */
        public function dryInline(): string {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces PHP objects
         * for the second column if it contains JSON objects.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces associative arrays
         * for the second column if it contains JSON objects.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as PHP objects.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as associative arrays.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedDictionaryAssoc(?array $parameters = null): mixed {}

        public function __construct() {}
    }

    /**
     * A prepared SQL query builder.
     *
     * Holds the generated query string, parameters, and placeholder tracking
     * used during safe, composable query construction via AST rendering.
     */
    class MySqlWriteQueryBuilder implements Sqlx\WriteQueryBuilderInterface {
        /**
         * @param \Sqlx\MySqlDriver $driver
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public static function factory(\Sqlx\MySqlDriver $driver): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * @param mixed $from
         * @param array|null $parameters
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function from(mixed $from, ?array $parameters = null): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * @param mixed $set
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function set(mixed $set): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `CROSS JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function crossJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `DELETE FROM` clause to the query.
         *
         * @param mixed $from A string table name or a subquery builder.
         * @param array|null $parameters Optional parameters if `from` is a raw string.
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function deleteFrom(mixed $from, ?array $parameters = null): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `FOR SHARE` locking clause to the query.
         *
         * This clause is used in `SELECT` statements to acquire shared locks
         * on the selected rows, allowing concurrent transactions to read but
         * not modify the rows until the current transaction completes.
         *
         * @return \Sqlx\MySqlWriteQueryBuilder The query builder with `FOR SHARE` appended.
         */
        public function forShare(): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `FOR UPDATE` locking clause to the query.
         *
         * This clause is used in `SELECT` statements to lock the selected rows
         * for update, preventing other transactions from modifying or acquiring
         * locks on them until the current transaction completes.
         *
         * @return \Sqlx\MySqlWriteQueryBuilder The query builder with `FOR UPDATE` appended.
         */
        public function forUpdate(): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `FULL OUTER JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function fullJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `GROUP BY` clause to the query.
         *
         * @param mixed $fields Either a raw string or a `ByClauseRendered` object.
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function groupBy(mixed $fields): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `HAVING` clause to the query.
         *
         * @param mixed $having Either a raw string, a structured array of conditions, or a disjunction (`OrClause`).
         * @param array|null $parameters Optional parameters associated with the `WHERE` condition.
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function having(mixed $having, ?array $parameters = null): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `LEFT JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function leftJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `LIMIT` (and optional `OFFSET`) clause to the query.
         *
         * @param int $limit – Maximum number of rows to return.
         * @param int|null $offset – Optional number of rows to skip before starting to return rows.
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function limit(int $limit, ?int $offset = null): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `NATURAL JOIN` clause to the query.
         *
         * @param string $table
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function naturalJoin(string $table): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `ORDER BY` clause to the query.
         *
         * @param mixed $fields Either a raw string or a `ByClauseRendered` object.
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function orderBy(mixed $fields): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `PAGINATE` clause to the query using a `PaginateClauseRendered` object.
         *
         * This expands into the appropriate SQL `LIMIT` and `OFFSET` syntax during rendering,
         * using the values stored in the given `PaginateClauseRendered` instance.
         *
         * @param mixed $paginate – An instance of `Sqlx\PaginateClauseRendered`, produced by invoking a `PaginateClause`
         * @return \Sqlx\MySqlWriteQueryBuilder
         * @throws \Exception Returns an error if the argument is not an instance of `PaginateClauseRendered`.
         */
        public function paginate(mixed $paginate): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `RETURNING` clause to the query.
         *
         * @param mixed $fields A string or array of column names to return.
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function returning(mixed $fields): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `RIGHT JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function rightJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `SELECT` clause to the query.
         *
         * @param mixed $fields Either a raw string or a `SelectClauseRendered` object.
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function select(mixed $fields): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `SET` clause to the query, supporting both keyed and indexed formats.
         *
         * @param mixed $set An associative array mapping fields to values, or a sequential array
         * @param string $context
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function SetAssignments(mixed $set, string $context): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `TRUNCATE TABLE` statement to the query.
         *
         * This command removes all rows from the specified table quickly and efficiently.
         * It is faster than `DELETE FROM` and usually does not fire triggers or return affected row counts.
         *
         * @param string $table – The name of the table to truncate.
         * @return \Sqlx\MySqlWriteQueryBuilder
         * @throws \Exception Returns an error if appending the SQL fragment fails (e.g., formatting error).
         */
        public function truncateTable(string $table): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `UNION ALL` clause to the query.
         *
         * @param mixed $query – A raw SQL string or another Builder instance (subquery).
         * @param array|null $parameters – Optional parameters to bind to the unioned subquery.
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function unionAll(mixed $query, ?array $parameters = null): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `UNION` clause to the query.
         *
         * @param mixed $query – A raw SQL string or another Builder instance (subquery).
         * @param array|null $parameters – Optional parameters to bind to the unioned subquery.
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function union(mixed $query, ?array $parameters = null): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `UPDATE` clause to the query.
         *
         * @param mixed $table A raw string representing the table(s).
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function update(mixed $table): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `USING` clause to the query.
         *
         * @param mixed $from Either a string table name or a subquery builder.
         * @param array|null $parameters Optional parameters if `from` is a string.
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function using(mixed $from, ?array $parameters = null): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `VALUES` clause to the query.
         *
         * @param mixed $values Can be:
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function values(mixed $values): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `WHERE` clause to the query.
         *
         * @param mixed $where_
         * @param array|null $parameters Optional parameters associated with the `WHERE` condition.
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function Where(mixed $where_, ?array $parameters = null): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `WITH RECURSIVE` clause to the query.
         *
         * @param string $table_and_fields Table name with field list, e.g. `cte(col1, col2)`.
         * @param mixed $as_
         * @param array|null $parameters Optional parameters for the recursive body.
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function withRecursive(string $table_and_fields, mixed $as_, ?array $parameters = null): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a `WITH` clause to the query.
         *
         * @param string $table Name of the CTE (common table expression).
         * @param mixed $as_
         * @param array|null $parameters Optional parameters for the query body.
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function with(string $table, mixed $as_, ?array $parameters = null): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends a raw SQL fragment to the query without structural validation.
         *
         * This method allows injecting raw SQL clauses directly into the query. It's intended
         * for advanced use cases such as vendor-specific syntax, subqueries, or complex expressions
         * not yet supported by the structured builder methods.
         *
         * @param string $part A raw SQL string to append to the query. It is inserted **verbatim** into the
         * @param array|null $parameters An optional map of named parameters to bind to placeholders within the SQL string.
         * @return \Sqlx\MySqlWriteQueryBuilder Returns a mutable reference to the builder for fluent method chaining.
         */
        public function raw(string $part, ?array $parameters = null): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends an `INNER JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function innerJoin(string $table, string $on, ?array $parameters = null): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends an `INNER JOIN` clause to the query.
         * Alias for `inner_join()`.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function join(string $table, string $on, ?array $parameters = null): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends an `INSERT INTO` clause to the query.
         *
         * @param string $table The name of the target table.
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function insertInto(string $table): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends an `OFFSET` clause to the query independently.
         *
         * @param int $offset – Number of rows to skip before returning results.
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function offset(int $offset): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends an `ON CONFLICT` clause to the query.
         *
         * @param mixed $target – A string or array of column names to specify the conflict target.
         * @param mixed $set – Optional `SET` clause. If `null`, generates `DO NOTHING`; otherwise uses the `SET` values.
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function onConflict(mixed $target, mixed $set = null): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends an `ON DUPLICATE KEY UPDATE` clause to the query (`MySQL`).
         *
         * @param mixed $set – An array representing fields and values to update.
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function onDuplicateKeyUpdate(mixed $set): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends an `REPLACE INTO` clause to the query.
         *
         * @param string $table The name of the target table.
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function replaceInto(string $table): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Appends multiple rows to the `VALUES` clause of an `INSERT` statement.
         *
         * Each row must be:
         * - an ordered list of values (indexed array),
         * - or a map of column names to values (associative array) — only for the first row, to infer column order.
         *
         * @param mixed $rows – A sequential array of rows (arrays of values).
         * @return \Sqlx\MySqlWriteQueryBuilder
         */
        public function valuesMany(mixed $rows): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Creates a query builder object
         *
         * @return \Sqlx\MySqlWriteQueryBuilder Query builder object
         */
        public function builder(): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Escapes `%` and `_` characters in a string for safe use in a LIKE/ILIKE pattern.
         *
         * This helper is designed for safely preparing user input for use with
         * pattern-matching operators like `CONTAINS`, `STARTS_WITH`, or `ENDS_WITH`.
         *
         * ⚠️ **Warning:** This method does **not** quote or escape the full string for raw SQL.
         * It only escapes `%` and `_` characters. You must still pass the result as a bound parameter,
         * not interpolate it directly into the query string.
         *
         * @param mixed $param – The parameter to escape (must be a string).
         * @return string A string with `%` and `_` escaped for LIKE (e.g., `foo\%bar\_baz`)
         * @throws \Exception Returns an error if the input is not a string.
         */
        public function metaQuoteLike(mixed $param): string {}

        /**
         * Executes a query and returns a grouped dictionary (Vec of rows per key).
         *
         * Same as [`queryGroupedDictionary`](crate::Driver::query_grouped_dictionary), but works on a prepared query.
         *
         * The first column is used as the key (must be scalar),
         * and each resulting row is appended to the corresponding key's Vec.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Fails if the query fails, or the first column is not scalar.
         */
        public function queryGroupedDictionary(?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a single result, or `null` if no row matched.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed Single row as array or object depending on config
         */
        public function queryMaybeRow(?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a single row as a PHP object, or `null` if no row matched.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed The result row as a `stdClass` PHP object, or `null` if no matching row is found.
         */
        public function queryMaybeRowObj(?array $parameters = null): mixed {}

        /**
         * Executes the SQL query and returns a single row as a PHP associative array, or `null` if no row matched.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed The result row as an associative array (`array<string, mixed>` in PHP), or `null` if no matching row is found.
         */
        public function queryMaybeRowAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the SQL query and returns the specified column values from all result rows.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param mixed $column Optional column name or index to extract.
         * @return array An array of column values, one for each row.
         */
        public function queryColumn(?array $parameters = null, mixed $column = null): array {}

        /**
         * Executes the SQL query and returns the specified column values from all rows in associative array mode.
         *
         * @param array|null $parameters Optional named parameters.
         * @param mixed $column Column index or name to extract.
         * @return array An array of column values (associative arrays for structured data).
         */
        public function queryColumnAssoc(?array $parameters = null, mixed $column = null): array {}

        /**
         * Executes the SQL query and returns the specified column values from all rows in object mode.
         *
         * @param array|null $parameters Optional named parameters.
         * @param mixed $column Column index or name to extract.
         * @return array An array of column values (objects for structured data).
         */
        public function queryColumnObj(?array $parameters = null, mixed $column = null): array {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an associative array.
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row as an associative array.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an object (`stdClass`).
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row as an object.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row.
         *
         * The result is a `HashMap` where the key is the stringified first column from each row,
         * and the value is the full row, returned as array or object depending on config.
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in associative array mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as associative arrays.
         *
         * @param array|null $parameters
         * @return mixed A dictionary where each key is the first column (as string), and each value is the second column as an associative PHP array.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in object mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as objects.
         *
         * @param array|null $parameters
         * @return mixed A dictionary where each key is the first column (as string), and each value is the second column as a PHP object.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary mapping the first column to the second column.
         *
         * This method expects each result row to contain at least two columns. It converts the first column
         * into a PHP string (used as the key), and the second column into a PHP value (used as the value).
         *
         * @param array|null $parameters
         * @return mixed An associative array (`array<string, mixed>`) where each key is the first column (as string), and the value is the second column.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a PHP string;
         * @throws \Exception - the second column cannot be converted to a PHP value.
         */
        public function queryColumnDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a grouped dictionary where:
         * - the key is the **first column** (must be scalar),
         * - the value is a list of values from the **second column** for each group.
         *
         * This variant uses the driver's default associative array option for JSON values.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a single result.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed Single row as array or object depending on config
         */
        public function queryRow(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns all rows as associative arrays.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array
         */
        public function queryAllAssoc(?array $parameters = null): array {}

        /**
         * Executes the prepared query and returns all rows as objects.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array
         */
        public function queryAllObj(?array $parameters = null): array {}

        /**
         * Executes the prepared query and returns all rows.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array Array of rows as array or object depending on config
         */
        public function queryAll(?array $parameters = null): array {}

        /**
         * Executes the prepared query and returns one row as an associative array.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed
         */
        public function queryRowAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns one row as an object.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed
         */
        public function queryRowObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query with optional parameters.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return int Number of affected rows
         */
        public function execute(?array $parameters = null): int {}

        /**
         * Executes the query and returns a lazy `QueryResult` iterator with rows as associative arrays.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\MySqlQueryResult A `QueryResult` object with rows as associative arrays
         */
        public function queryAssoc(?array $parameters = null, ?int $batch_size = null): \Sqlx\MySqlQueryResult {}

        /**
         * Executes the query and returns a lazy `QueryResult` iterator with rows as objects.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\MySqlQueryResult A `QueryResult` object with rows as PHP objects
         */
        public function queryObj(?array $parameters = null, ?int $batch_size = null): \Sqlx\MySqlQueryResult {}

        /**
         * Executes the query and returns a lazy `QueryResult` iterator.
         *
         * This method returns a `QueryResult` object that implements PHP's `Iterator`
         * interface, streaming rows from the database as you iterate.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\MySqlQueryResult A `QueryResult` object implementing `Iterator`
         */
        public function query(?array $parameters = null, ?int $batch_size = null): \Sqlx\MySqlQueryResult {}

        /**
         * Finalizes the query by appending a semicolon (`;`).
         *
         * This method is optional. Most databases do not require semicolons in prepared queries,
         * but you may use it to explicitly terminate a query string.
         *
         * @param string $_table
         * @return \Sqlx\MySqlWriteQueryBuilder The builder instance after appending the semicolon.
         */
        public function end(string $_table): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Merges additional parameters with the builder's accumulated parameters.
         *
         * Parameters passed directly to the method take precedence over builder parameters.
         *
         * @param array|null $parameters
         * @return array|null
         */
        public function mergeParameters(?array $parameters = null): ?array {}

        /**
         * Quotes a single scalar value for safe embedding into SQL.
         *
         * This method renders the given `ParameterValue` into a properly escaped SQL literal,
         * using the driver's configuration (e.g., quoting style, encoding).
         *
         * ⚠️ **Warning:** Prefer using placeholders and parameter binding wherever possible.
         * This method should only be used for debugging or generating static fragments,
         * not for constructing dynamic SQL with user input.
         *
         * @param mixed $param – The parameter to quote (must be a scalar: string, number, or boolean).
         * @return string Quoted SQL string (e.g., `'abc'`, `123`, `TRUE`)
         * @throws \Exception Returns an error if the parameter is not a scalar or if rendering fails.
         */
        public function quote(mixed $param): string {}

        /**
         * Renders the SQL query using named parameters and returns the fully rendered SQL with values injected inline.
         *
         * @return array The rendered SQL query as a string with all parameters interpolated.
         */
        public function dry(): array {}

        /**
         * Returns an array of all currently accumulated parameters.
         *
         * @return array
         */
        public function parameters(): array {}

        /**
         * Returns the fully rendered SQL query with parameters embedded as literals.
         *
         * Used for debugging or fallback rendering when the placeholder limit is exceeded.
         *
         * @return string A string representing the complete SQL statement.
         */
        public function __toString(): string {}

        /**
         * Returns the parameter map currently accumulated in the builder.
         *
         * @return string A cloned map of all parameter names and their corresponding `ParameterValue`.
         */
        public function dryInline(): string {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces PHP objects
         * for the second column if it contains JSON objects.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces associative arrays
         * for the second column if it contains JSON objects.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as PHP objects.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as associative arrays.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedDictionaryAssoc(?array $parameters = null): mixed {}

        public function __construct() {}
    }

    /**
     * A prepared SQL query builder.
     *
     * Holds the generated query string, parameters, and placeholder tracking
     * used during safe, composable query construction via AST rendering.
     */
    class PgReadQueryBuilder implements Sqlx\ReadQueryBuilderInterface {
        /**
         * @param \Sqlx\PgDriver $driver
         * @return \Sqlx\PgReadQueryBuilder
         */
        public static function factory(\Sqlx\PgDriver $driver): \Sqlx\PgReadQueryBuilder {}

        /**
         * @param mixed $from
         * @param array|null $parameters
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function from(mixed $from, ?array $parameters = null): \Sqlx\PgReadQueryBuilder {}

        /**
         * @param mixed $set
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function set(mixed $set): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `CROSS JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function crossJoin(string $table, string $on, ?array $parameters = null): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `DELETE FROM` clause to the query.
         *
         * @param mixed $from A string table name or a subquery builder.
         * @param array|null $parameters Optional parameters if `from` is a raw string.
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function deleteFrom(mixed $from, ?array $parameters = null): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `FOR SHARE` locking clause to the query.
         *
         * This clause is used in `SELECT` statements to acquire shared locks
         * on the selected rows, allowing concurrent transactions to read but
         * not modify the rows until the current transaction completes.
         *
         * @return \Sqlx\PgReadQueryBuilder The query builder with `FOR SHARE` appended.
         */
        public function forShare(): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `FOR UPDATE` locking clause to the query.
         *
         * This clause is used in `SELECT` statements to lock the selected rows
         * for update, preventing other transactions from modifying or acquiring
         * locks on them until the current transaction completes.
         *
         * @return \Sqlx\PgReadQueryBuilder The query builder with `FOR UPDATE` appended.
         */
        public function forUpdate(): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `FULL OUTER JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function fullJoin(string $table, string $on, ?array $parameters = null): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `GROUP BY` clause to the query.
         *
         * @param mixed $fields Either a raw string or a `ByClauseRendered` object.
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function groupBy(mixed $fields): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `HAVING` clause to the query.
         *
         * @param mixed $having Either a raw string, a structured array of conditions, or a disjunction (`OrClause`).
         * @param array|null $parameters Optional parameters associated with the `WHERE` condition.
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function having(mixed $having, ?array $parameters = null): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `LEFT JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function leftJoin(string $table, string $on, ?array $parameters = null): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `LIMIT` (and optional `OFFSET`) clause to the query.
         *
         * @param int $limit – Maximum number of rows to return.
         * @param int|null $offset – Optional number of rows to skip before starting to return rows.
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function limit(int $limit, ?int $offset = null): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `NATURAL JOIN` clause to the query.
         *
         * @param string $table
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function naturalJoin(string $table): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `ORDER BY` clause to the query.
         *
         * @param mixed $fields Either a raw string or a `ByClauseRendered` object.
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function orderBy(mixed $fields): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `PAGINATE` clause to the query using a `PaginateClauseRendered` object.
         *
         * This expands into the appropriate SQL `LIMIT` and `OFFSET` syntax during rendering,
         * using the values stored in the given `PaginateClauseRendered` instance.
         *
         * @param mixed $paginate – An instance of `Sqlx\PaginateClauseRendered`, produced by invoking a `PaginateClause`
         * @return \Sqlx\PgReadQueryBuilder
         * @throws \Exception Returns an error if the argument is not an instance of `PaginateClauseRendered`.
         */
        public function paginate(mixed $paginate): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `RETURNING` clause to the query.
         *
         * @param mixed $fields A string or array of column names to return.
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function returning(mixed $fields): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `RIGHT JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function rightJoin(string $table, string $on, ?array $parameters = null): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `SELECT` clause to the query.
         *
         * @param mixed $fields Either a raw string or a `SelectClauseRendered` object.
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function select(mixed $fields): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `SET` clause to the query, supporting both keyed and indexed formats.
         *
         * @param mixed $set An associative array mapping fields to values, or a sequential array
         * @param string $context
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function SetAssignments(mixed $set, string $context): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `TRUNCATE TABLE` statement to the query.
         *
         * This command removes all rows from the specified table quickly and efficiently.
         * It is faster than `DELETE FROM` and usually does not fire triggers or return affected row counts.
         *
         * @param string $table – The name of the table to truncate.
         * @return \Sqlx\PgReadQueryBuilder
         * @throws \Exception Returns an error if appending the SQL fragment fails (e.g., formatting error).
         */
        public function truncateTable(string $table): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `UNION ALL` clause to the query.
         *
         * @param mixed $query – A raw SQL string or another Builder instance (subquery).
         * @param array|null $parameters – Optional parameters to bind to the unioned subquery.
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function unionAll(mixed $query, ?array $parameters = null): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `UNION` clause to the query.
         *
         * @param mixed $query – A raw SQL string or another Builder instance (subquery).
         * @param array|null $parameters – Optional parameters to bind to the unioned subquery.
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function union(mixed $query, ?array $parameters = null): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `UPDATE` clause to the query.
         *
         * @param mixed $table A raw string representing the table(s).
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function update(mixed $table): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `USING` clause to the query.
         *
         * @param mixed $from Either a string table name or a subquery builder.
         * @param array|null $parameters Optional parameters if `from` is a string.
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function using(mixed $from, ?array $parameters = null): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `VALUES` clause to the query.
         *
         * @param mixed $values Can be:
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function values(mixed $values): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `WHERE` clause to the query.
         *
         * @param mixed $where_
         * @param array|null $parameters Optional parameters associated with the `WHERE` condition.
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function Where(mixed $where_, ?array $parameters = null): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `WITH RECURSIVE` clause to the query.
         *
         * @param string $table_and_fields Table name with field list, e.g. `cte(col1, col2)`.
         * @param mixed $as_
         * @param array|null $parameters Optional parameters for the recursive body.
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function withRecursive(string $table_and_fields, mixed $as_, ?array $parameters = null): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a `WITH` clause to the query.
         *
         * @param string $table Name of the CTE (common table expression).
         * @param mixed $as_
         * @param array|null $parameters Optional parameters for the query body.
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function with(string $table, mixed $as_, ?array $parameters = null): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends a raw SQL fragment to the query without structural validation.
         *
         * This method allows injecting raw SQL clauses directly into the query. It's intended
         * for advanced use cases such as vendor-specific syntax, subqueries, or complex expressions
         * not yet supported by the structured builder methods.
         *
         * @param string $part A raw SQL string to append to the query. It is inserted **verbatim** into the
         * @param array|null $parameters An optional map of named parameters to bind to placeholders within the SQL string.
         * @return \Sqlx\PgReadQueryBuilder Returns a mutable reference to the builder for fluent method chaining.
         */
        public function raw(string $part, ?array $parameters = null): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends an `INNER JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function innerJoin(string $table, string $on, ?array $parameters = null): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends an `INNER JOIN` clause to the query.
         * Alias for `inner_join()`.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function join(string $table, string $on, ?array $parameters = null): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends an `INSERT INTO` clause to the query.
         *
         * @param string $table The name of the target table.
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function insertInto(string $table): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends an `OFFSET` clause to the query independently.
         *
         * @param int $offset – Number of rows to skip before returning results.
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function offset(int $offset): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends an `ON CONFLICT` clause to the query.
         *
         * @param mixed $target – A string or array of column names to specify the conflict target.
         * @param mixed $set – Optional `SET` clause. If `null`, generates `DO NOTHING`; otherwise uses the `SET` values.
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function onConflict(mixed $target, mixed $set = null): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends an `ON DUPLICATE KEY UPDATE` clause to the query (`MySQL`).
         *
         * @param mixed $set – An array representing fields and values to update.
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function onDuplicateKeyUpdate(mixed $set): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends an `REPLACE INTO` clause to the query.
         *
         * @param string $table The name of the target table.
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function replaceInto(string $table): \Sqlx\PgReadQueryBuilder {}

        /**
         * Appends multiple rows to the `VALUES` clause of an `INSERT` statement.
         *
         * Each row must be:
         * - an ordered list of values (indexed array),
         * - or a map of column names to values (associative array) — only for the first row, to infer column order.
         *
         * @param mixed $rows – A sequential array of rows (arrays of values).
         * @return \Sqlx\PgReadQueryBuilder
         */
        public function valuesMany(mixed $rows): \Sqlx\PgReadQueryBuilder {}

        /**
         * Creates a query builder object
         *
         * @return \Sqlx\PgReadQueryBuilder Query builder object
         */
        public function builder(): \Sqlx\PgReadQueryBuilder {}

        /**
         * Escapes `%` and `_` characters in a string for safe use in a LIKE/ILIKE pattern.
         *
         * This helper is designed for safely preparing user input for use with
         * pattern-matching operators like `CONTAINS`, `STARTS_WITH`, or `ENDS_WITH`.
         *
         * ⚠️ **Warning:** This method does **not** quote or escape the full string for raw SQL.
         * It only escapes `%` and `_` characters. You must still pass the result as a bound parameter,
         * not interpolate it directly into the query string.
         *
         * @param mixed $param – The parameter to escape (must be a string).
         * @return string A string with `%` and `_` escaped for LIKE (e.g., `foo\%bar\_baz`)
         * @throws \Exception Returns an error if the input is not a string.
         */
        public function metaQuoteLike(mixed $param): string {}

        /**
         * Executes a query and returns a grouped dictionary (Vec of rows per key).
         *
         * Same as [`queryGroupedDictionary`](crate::Driver::query_grouped_dictionary), but works on a prepared query.
         *
         * The first column is used as the key (must be scalar),
         * and each resulting row is appended to the corresponding key's Vec.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Fails if the query fails, or the first column is not scalar.
         */
        public function queryGroupedDictionary(?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a single result, or `null` if no row matched.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed Single row as array or object depending on config
         */
        public function queryMaybeRow(?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a single row as a PHP object, or `null` if no row matched.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed The result row as a `stdClass` PHP object, or `null` if no matching row is found.
         */
        public function queryMaybeRowObj(?array $parameters = null): mixed {}

        /**
         * Executes the SQL query and returns a single row as a PHP associative array, or `null` if no row matched.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed The result row as an associative array (`array<string, mixed>` in PHP), or `null` if no matching row is found.
         */
        public function queryMaybeRowAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the SQL query and returns the specified column values from all result rows.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param mixed $column Optional column name or index to extract.
         * @return array An array of column values, one for each row.
         */
        public function queryColumn(?array $parameters = null, mixed $column = null): array {}

        /**
         * Executes the SQL query and returns the specified column values from all rows in associative array mode.
         *
         * @param array|null $parameters Optional named parameters.
         * @param mixed $column Column index or name to extract.
         * @return array An array of column values (associative arrays for structured data).
         */
        public function queryColumnAssoc(?array $parameters = null, mixed $column = null): array {}

        /**
         * Executes the SQL query and returns the specified column values from all rows in object mode.
         *
         * @param array|null $parameters Optional named parameters.
         * @param mixed $column Column index or name to extract.
         * @return array An array of column values (objects for structured data).
         */
        public function queryColumnObj(?array $parameters = null, mixed $column = null): array {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an associative array.
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row as an associative array.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an object (`stdClass`).
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row as an object.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row.
         *
         * The result is a `HashMap` where the key is the stringified first column from each row,
         * and the value is the full row, returned as array or object depending on config.
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in associative array mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as associative arrays.
         *
         * @param array|null $parameters
         * @return mixed A dictionary where each key is the first column (as string), and each value is the second column as an associative PHP array.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in object mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as objects.
         *
         * @param array|null $parameters
         * @return mixed A dictionary where each key is the first column (as string), and each value is the second column as a PHP object.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary mapping the first column to the second column.
         *
         * This method expects each result row to contain at least two columns. It converts the first column
         * into a PHP string (used as the key), and the second column into a PHP value (used as the value).
         *
         * @param array|null $parameters
         * @return mixed An associative array (`array<string, mixed>`) where each key is the first column (as string), and the value is the second column.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a PHP string;
         * @throws \Exception - the second column cannot be converted to a PHP value.
         */
        public function queryColumnDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a grouped dictionary where:
         * - the key is the **first column** (must be scalar),
         * - the value is a list of values from the **second column** for each group.
         *
         * This variant uses the driver's default associative array option for JSON values.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a single result.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed Single row as array or object depending on config
         */
        public function queryRow(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns all rows as associative arrays.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array
         */
        public function queryAllAssoc(?array $parameters = null): array {}

        /**
         * Executes the prepared query and returns all rows as objects.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array
         */
        public function queryAllObj(?array $parameters = null): array {}

        /**
         * Executes the prepared query and returns all rows.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array Array of rows as array or object depending on config
         */
        public function queryAll(?array $parameters = null): array {}

        /**
         * Executes the prepared query and returns one row as an associative array.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed
         */
        public function queryRowAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns one row as an object.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed
         */
        public function queryRowObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query with optional parameters.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return int Number of affected rows
         */
        public function execute(?array $parameters = null): int {}

        /**
         * Executes the query and returns a lazy `QueryResult` iterator with rows as associative arrays.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\PgQueryResult A `QueryResult` object with rows as associative arrays
         */
        public function queryAssoc(?array $parameters = null, ?int $batch_size = null): \Sqlx\PgQueryResult {}

        /**
         * Executes the query and returns a lazy `QueryResult` iterator with rows as objects.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\PgQueryResult A `QueryResult` object with rows as PHP objects
         */
        public function queryObj(?array $parameters = null, ?int $batch_size = null): \Sqlx\PgQueryResult {}

        /**
         * Executes the query and returns a lazy `QueryResult` iterator.
         *
         * This method returns a `QueryResult` object that implements PHP's `Iterator`
         * interface, streaming rows from the database as you iterate.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\PgQueryResult A `QueryResult` object implementing `Iterator`
         */
        public function query(?array $parameters = null, ?int $batch_size = null): \Sqlx\PgQueryResult {}

        /**
         * Finalizes the query by appending a semicolon (`;`).
         *
         * This method is optional. Most databases do not require semicolons in prepared queries,
         * but you may use it to explicitly terminate a query string.
         *
         * @param string $_table
         * @return \Sqlx\PgReadQueryBuilder The builder instance after appending the semicolon.
         */
        public function end(string $_table): \Sqlx\PgReadQueryBuilder {}

        /**
         * Merges additional parameters with the builder's accumulated parameters.
         *
         * Parameters passed directly to the method take precedence over builder parameters.
         *
         * @param array|null $parameters
         * @return array|null
         */
        public function mergeParameters(?array $parameters = null): ?array {}

        /**
         * Quotes a single scalar value for safe embedding into SQL.
         *
         * This method renders the given `ParameterValue` into a properly escaped SQL literal,
         * using the driver's configuration (e.g., quoting style, encoding).
         *
         * ⚠️ **Warning:** Prefer using placeholders and parameter binding wherever possible.
         * This method should only be used for debugging or generating static fragments,
         * not for constructing dynamic SQL with user input.
         *
         * @param mixed $param – The parameter to quote (must be a scalar: string, number, or boolean).
         * @return string Quoted SQL string (e.g., `'abc'`, `123`, `TRUE`)
         * @throws \Exception Returns an error if the parameter is not a scalar or if rendering fails.
         */
        public function quote(mixed $param): string {}

        /**
         * Renders the SQL query using named parameters and returns the fully rendered SQL with values injected inline.
         *
         * @return array The rendered SQL query as a string with all parameters interpolated.
         */
        public function dry(): array {}

        /**
         * Returns an array of all currently accumulated parameters.
         *
         * @return array
         */
        public function parameters(): array {}

        /**
         * Returns the fully rendered SQL query with parameters embedded as literals.
         *
         * Used for debugging or fallback rendering when the placeholder limit is exceeded.
         *
         * @return string A string representing the complete SQL statement.
         */
        public function __toString(): string {}

        /**
         * Returns the parameter map currently accumulated in the builder.
         *
         * @return string A cloned map of all parameter names and their corresponding `ParameterValue`.
         */
        public function dryInline(): string {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces PHP objects
         * for the second column if it contains JSON objects.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces associative arrays
         * for the second column if it contains JSON objects.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as PHP objects.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as associative arrays.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedDictionaryAssoc(?array $parameters = null): mixed {}

        public function __construct() {}
    }

    /**
     * A prepared SQL query builder.
     *
     * Holds the generated query string, parameters, and placeholder tracking
     * used during safe, composable query construction via AST rendering.
     */
    class PgWriteQueryBuilder implements Sqlx\WriteQueryBuilderInterface {
        /**
         * @param \Sqlx\PgDriver $driver
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public static function factory(\Sqlx\PgDriver $driver): \Sqlx\PgWriteQueryBuilder {}

        /**
         * @param mixed $from
         * @param array|null $parameters
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function from(mixed $from, ?array $parameters = null): \Sqlx\PgWriteQueryBuilder {}

        /**
         * @param mixed $set
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function set(mixed $set): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `CROSS JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function crossJoin(string $table, string $on, ?array $parameters = null): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `DELETE FROM` clause to the query.
         *
         * @param mixed $from A string table name or a subquery builder.
         * @param array|null $parameters Optional parameters if `from` is a raw string.
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function deleteFrom(mixed $from, ?array $parameters = null): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `FOR SHARE` locking clause to the query.
         *
         * This clause is used in `SELECT` statements to acquire shared locks
         * on the selected rows, allowing concurrent transactions to read but
         * not modify the rows until the current transaction completes.
         *
         * @return \Sqlx\PgWriteQueryBuilder The query builder with `FOR SHARE` appended.
         */
        public function forShare(): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `FOR UPDATE` locking clause to the query.
         *
         * This clause is used in `SELECT` statements to lock the selected rows
         * for update, preventing other transactions from modifying or acquiring
         * locks on them until the current transaction completes.
         *
         * @return \Sqlx\PgWriteQueryBuilder The query builder with `FOR UPDATE` appended.
         */
        public function forUpdate(): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `FULL OUTER JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function fullJoin(string $table, string $on, ?array $parameters = null): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `GROUP BY` clause to the query.
         *
         * @param mixed $fields Either a raw string or a `ByClauseRendered` object.
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function groupBy(mixed $fields): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `HAVING` clause to the query.
         *
         * @param mixed $having Either a raw string, a structured array of conditions, or a disjunction (`OrClause`).
         * @param array|null $parameters Optional parameters associated with the `WHERE` condition.
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function having(mixed $having, ?array $parameters = null): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `LEFT JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function leftJoin(string $table, string $on, ?array $parameters = null): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `LIMIT` (and optional `OFFSET`) clause to the query.
         *
         * @param int $limit – Maximum number of rows to return.
         * @param int|null $offset – Optional number of rows to skip before starting to return rows.
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function limit(int $limit, ?int $offset = null): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `NATURAL JOIN` clause to the query.
         *
         * @param string $table
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function naturalJoin(string $table): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `ORDER BY` clause to the query.
         *
         * @param mixed $fields Either a raw string or a `ByClauseRendered` object.
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function orderBy(mixed $fields): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `PAGINATE` clause to the query using a `PaginateClauseRendered` object.
         *
         * This expands into the appropriate SQL `LIMIT` and `OFFSET` syntax during rendering,
         * using the values stored in the given `PaginateClauseRendered` instance.
         *
         * @param mixed $paginate – An instance of `Sqlx\PaginateClauseRendered`, produced by invoking a `PaginateClause`
         * @return \Sqlx\PgWriteQueryBuilder
         * @throws \Exception Returns an error if the argument is not an instance of `PaginateClauseRendered`.
         */
        public function paginate(mixed $paginate): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `RETURNING` clause to the query.
         *
         * @param mixed $fields A string or array of column names to return.
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function returning(mixed $fields): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `RIGHT JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function rightJoin(string $table, string $on, ?array $parameters = null): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `SELECT` clause to the query.
         *
         * @param mixed $fields Either a raw string or a `SelectClauseRendered` object.
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function select(mixed $fields): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `SET` clause to the query, supporting both keyed and indexed formats.
         *
         * @param mixed $set An associative array mapping fields to values, or a sequential array
         * @param string $context
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function SetAssignments(mixed $set, string $context): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `TRUNCATE TABLE` statement to the query.
         *
         * This command removes all rows from the specified table quickly and efficiently.
         * It is faster than `DELETE FROM` and usually does not fire triggers or return affected row counts.
         *
         * @param string $table – The name of the table to truncate.
         * @return \Sqlx\PgWriteQueryBuilder
         * @throws \Exception Returns an error if appending the SQL fragment fails (e.g., formatting error).
         */
        public function truncateTable(string $table): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `UNION ALL` clause to the query.
         *
         * @param mixed $query – A raw SQL string or another Builder instance (subquery).
         * @param array|null $parameters – Optional parameters to bind to the unioned subquery.
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function unionAll(mixed $query, ?array $parameters = null): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `UNION` clause to the query.
         *
         * @param mixed $query – A raw SQL string or another Builder instance (subquery).
         * @param array|null $parameters – Optional parameters to bind to the unioned subquery.
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function union(mixed $query, ?array $parameters = null): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `UPDATE` clause to the query.
         *
         * @param mixed $table A raw string representing the table(s).
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function update(mixed $table): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `USING` clause to the query.
         *
         * @param mixed $from Either a string table name or a subquery builder.
         * @param array|null $parameters Optional parameters if `from` is a string.
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function using(mixed $from, ?array $parameters = null): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `VALUES` clause to the query.
         *
         * @param mixed $values Can be:
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function values(mixed $values): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `WHERE` clause to the query.
         *
         * @param mixed $where_
         * @param array|null $parameters Optional parameters associated with the `WHERE` condition.
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function Where(mixed $where_, ?array $parameters = null): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `WITH RECURSIVE` clause to the query.
         *
         * @param string $table_and_fields Table name with field list, e.g. `cte(col1, col2)`.
         * @param mixed $as_
         * @param array|null $parameters Optional parameters for the recursive body.
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function withRecursive(string $table_and_fields, mixed $as_, ?array $parameters = null): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a `WITH` clause to the query.
         *
         * @param string $table Name of the CTE (common table expression).
         * @param mixed $as_
         * @param array|null $parameters Optional parameters for the query body.
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function with(string $table, mixed $as_, ?array $parameters = null): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends a raw SQL fragment to the query without structural validation.
         *
         * This method allows injecting raw SQL clauses directly into the query. It's intended
         * for advanced use cases such as vendor-specific syntax, subqueries, or complex expressions
         * not yet supported by the structured builder methods.
         *
         * @param string $part A raw SQL string to append to the query. It is inserted **verbatim** into the
         * @param array|null $parameters An optional map of named parameters to bind to placeholders within the SQL string.
         * @return \Sqlx\PgWriteQueryBuilder Returns a mutable reference to the builder for fluent method chaining.
         */
        public function raw(string $part, ?array $parameters = null): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends an `INNER JOIN` clause to the query.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function innerJoin(string $table, string $on, ?array $parameters = null): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends an `INNER JOIN` clause to the query.
         * Alias for `inner_join()`.
         *
         * @param string $table
         * @param string $on
         * @param array|null $parameters
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function join(string $table, string $on, ?array $parameters = null): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends an `INSERT INTO` clause to the query.
         *
         * @param string $table The name of the target table.
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function insertInto(string $table): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends an `OFFSET` clause to the query independently.
         *
         * @param int $offset – Number of rows to skip before returning results.
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function offset(int $offset): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends an `ON CONFLICT` clause to the query.
         *
         * @param mixed $target – A string or array of column names to specify the conflict target.
         * @param mixed $set – Optional `SET` clause. If `null`, generates `DO NOTHING`; otherwise uses the `SET` values.
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function onConflict(mixed $target, mixed $set = null): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends an `ON DUPLICATE KEY UPDATE` clause to the query (`MySQL`).
         *
         * @param mixed $set – An array representing fields and values to update.
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function onDuplicateKeyUpdate(mixed $set): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends an `REPLACE INTO` clause to the query.
         *
         * @param string $table The name of the target table.
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function replaceInto(string $table): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Appends multiple rows to the `VALUES` clause of an `INSERT` statement.
         *
         * Each row must be:
         * - an ordered list of values (indexed array),
         * - or a map of column names to values (associative array) — only for the first row, to infer column order.
         *
         * @param mixed $rows – A sequential array of rows (arrays of values).
         * @return \Sqlx\PgWriteQueryBuilder
         */
        public function valuesMany(mixed $rows): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Creates a query builder object
         *
         * @return \Sqlx\PgWriteQueryBuilder Query builder object
         */
        public function builder(): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Escapes `%` and `_` characters in a string for safe use in a LIKE/ILIKE pattern.
         *
         * This helper is designed for safely preparing user input for use with
         * pattern-matching operators like `CONTAINS`, `STARTS_WITH`, or `ENDS_WITH`.
         *
         * ⚠️ **Warning:** This method does **not** quote or escape the full string for raw SQL.
         * It only escapes `%` and `_` characters. You must still pass the result as a bound parameter,
         * not interpolate it directly into the query string.
         *
         * @param mixed $param – The parameter to escape (must be a string).
         * @return string A string with `%` and `_` escaped for LIKE (e.g., `foo\%bar\_baz`)
         * @throws \Exception Returns an error if the input is not a string.
         */
        public function metaQuoteLike(mixed $param): string {}

        /**
         * Executes a query and returns a grouped dictionary (Vec of rows per key).
         *
         * Same as [`queryGroupedDictionary`](crate::Driver::query_grouped_dictionary), but works on a prepared query.
         *
         * The first column is used as the key (must be scalar),
         * and each resulting row is appended to the corresponding key's Vec.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Fails if the query fails, or the first column is not scalar.
         */
        public function queryGroupedDictionary(?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a single result, or `null` if no row matched.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed Single row as array or object depending on config
         */
        public function queryMaybeRow(?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a single row as a PHP object, or `null` if no row matched.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed The result row as a `stdClass` PHP object, or `null` if no matching row is found.
         */
        public function queryMaybeRowObj(?array $parameters = null): mixed {}

        /**
         * Executes the SQL query and returns a single row as a PHP associative array, or `null` if no row matched.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed The result row as an associative array (`array<string, mixed>` in PHP), or `null` if no matching row is found.
         */
        public function queryMaybeRowAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the SQL query and returns the specified column values from all result rows.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param mixed $column Optional column name or index to extract.
         * @return array An array of column values, one for each row.
         */
        public function queryColumn(?array $parameters = null, mixed $column = null): array {}

        /**
         * Executes the SQL query and returns the specified column values from all rows in associative array mode.
         *
         * @param array|null $parameters Optional named parameters.
         * @param mixed $column Column index or name to extract.
         * @return array An array of column values (associative arrays for structured data).
         */
        public function queryColumnAssoc(?array $parameters = null, mixed $column = null): array {}

        /**
         * Executes the SQL query and returns the specified column values from all rows in object mode.
         *
         * @param array|null $parameters Optional named parameters.
         * @param mixed $column Column index or name to extract.
         * @return array An array of column values (objects for structured data).
         */
        public function queryColumnObj(?array $parameters = null, mixed $column = null): array {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an associative array.
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row as an associative array.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an object (`stdClass`).
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row as an object.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row.
         *
         * The result is a `HashMap` where the key is the stringified first column from each row,
         * and the value is the full row, returned as array or object depending on config.
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in associative array mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as associative arrays.
         *
         * @param array|null $parameters
         * @return mixed A dictionary where each key is the first column (as string), and each value is the second column as an associative PHP array.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in object mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as objects.
         *
         * @param array|null $parameters
         * @return mixed A dictionary where each key is the first column (as string), and each value is the second column as a PHP object.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary mapping the first column to the second column.
         *
         * This method expects each result row to contain at least two columns. It converts the first column
         * into a PHP string (used as the key), and the second column into a PHP value (used as the value).
         *
         * @param array|null $parameters
         * @return mixed An associative array (`array<string, mixed>`) where each key is the first column (as string), and the value is the second column.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a PHP string;
         * @throws \Exception - the second column cannot be converted to a PHP value.
         */
        public function queryColumnDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a grouped dictionary where:
         * - the key is the **first column** (must be scalar),
         * - the value is a list of values from the **second column** for each group.
         *
         * This variant uses the driver's default associative array option for JSON values.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a single result.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed Single row as array or object depending on config
         */
        public function queryRow(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns all rows as associative arrays.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array
         */
        public function queryAllAssoc(?array $parameters = null): array {}

        /**
         * Executes the prepared query and returns all rows as objects.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array
         */
        public function queryAllObj(?array $parameters = null): array {}

        /**
         * Executes the prepared query and returns all rows.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array Array of rows as array or object depending on config
         */
        public function queryAll(?array $parameters = null): array {}

        /**
         * Executes the prepared query and returns one row as an associative array.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed
         */
        public function queryRowAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns one row as an object.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return mixed
         */
        public function queryRowObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query with optional parameters.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return int Number of affected rows
         */
        public function execute(?array $parameters = null): int {}

        /**
         * Executes the query and returns a lazy `QueryResult` iterator with rows as associative arrays.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\PgQueryResult A `QueryResult` object with rows as associative arrays
         */
        public function queryAssoc(?array $parameters = null, ?int $batch_size = null): \Sqlx\PgQueryResult {}

        /**
         * Executes the query and returns a lazy `QueryResult` iterator with rows as objects.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\PgQueryResult A `QueryResult` object with rows as PHP objects
         */
        public function queryObj(?array $parameters = null, ?int $batch_size = null): \Sqlx\PgQueryResult {}

        /**
         * Executes the query and returns a lazy `QueryResult` iterator.
         *
         * This method returns a `QueryResult` object that implements PHP's `Iterator`
         * interface, streaming rows from the database as you iterate.
         *
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\PgQueryResult A `QueryResult` object implementing `Iterator`
         */
        public function query(?array $parameters = null, ?int $batch_size = null): \Sqlx\PgQueryResult {}

        /**
         * Finalizes the query by appending a semicolon (`;`).
         *
         * This method is optional. Most databases do not require semicolons in prepared queries,
         * but you may use it to explicitly terminate a query string.
         *
         * @param string $_table
         * @return \Sqlx\PgWriteQueryBuilder The builder instance after appending the semicolon.
         */
        public function end(string $_table): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Merges additional parameters with the builder's accumulated parameters.
         *
         * Parameters passed directly to the method take precedence over builder parameters.
         *
         * @param array|null $parameters
         * @return array|null
         */
        public function mergeParameters(?array $parameters = null): ?array {}

        /**
         * Quotes a single scalar value for safe embedding into SQL.
         *
         * This method renders the given `ParameterValue` into a properly escaped SQL literal,
         * using the driver's configuration (e.g., quoting style, encoding).
         *
         * ⚠️ **Warning:** Prefer using placeholders and parameter binding wherever possible.
         * This method should only be used for debugging or generating static fragments,
         * not for constructing dynamic SQL with user input.
         *
         * @param mixed $param – The parameter to quote (must be a scalar: string, number, or boolean).
         * @return string Quoted SQL string (e.g., `'abc'`, `123`, `TRUE`)
         * @throws \Exception Returns an error if the parameter is not a scalar or if rendering fails.
         */
        public function quote(mixed $param): string {}

        /**
         * Renders the SQL query using named parameters and returns the fully rendered SQL with values injected inline.
         *
         * @return array The rendered SQL query as a string with all parameters interpolated.
         */
        public function dry(): array {}

        /**
         * Returns an array of all currently accumulated parameters.
         *
         * @return array
         */
        public function parameters(): array {}

        /**
         * Returns the fully rendered SQL query with parameters embedded as literals.
         *
         * Used for debugging or fallback rendering when the placeholder limit is exceeded.
         *
         * @return string A string representing the complete SQL statement.
         */
        public function __toString(): string {}

        /**
         * Returns the parameter map currently accumulated in the builder.
         *
         * @return string A cloned map of all parameter names and their corresponding `ParameterValue`.
         */
        public function dryInline(): string {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces PHP objects
         * for the second column if it contains JSON objects.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces associative arrays
         * for the second column if it contains JSON objects.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as PHP objects.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as associative arrays.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedDictionaryAssoc(?array $parameters = null): mixed {}

        public function __construct() {}
    }

    /**
     * A rendered ORDER BY / GROUP BY clause result for use in query generation.
     */
    class ByClauseRendered {
        public function __construct() {}
    }

    /**
     * A reusable prepared SQL query with parameter support. Created using `PgDriver::prepare()`, shares context with original driver.
     */
    class MySqlPreparedQuery implements Sqlx\PreparedQueryInterface {
        /**
         * @param array|null $parameters
         * @param mixed $column
         * @return array
         */
        public function queryColumn(?array $parameters = null, mixed $column = null): array {}

        /**
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryValue(?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param array|null $parameters
         * @return array
         */
        public function queryAll(?array $parameters = null): array {}

        /**
         * @param array|null $parameters
         * @return array
         */
        public function queryAllAssoc(?array $parameters = null): array {}

        /**
         * @param array|null $parameters
         * @return array
         */
        public function queryAllObj(?array $parameters = null): array {}

        /**
         * @param array|null $parameters
         * @return int
         */
        public function execute(?array $parameters = null): int {}

        /**
         * @param array|null $parameters
         * @return mixed
         */
        public function queryMaybeRow(?array $parameters = null): mixed {}

        /**
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRow(?array $parameters = null): mixed {}

        /**
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRowAssoc(?array $parameters = null): mixed {}

        /**
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRowObj(?array $parameters = null): mixed {}

        /**
         * Executes a query and returns a grouped dictionary (Vec of rows per key).
         *
         * Same as [`queryGroupedDictionary`](crate::Driver::query_grouped_dictionary), but works on a prepared query.
         *
         * The first column is used as the key (must be scalar),
         * and each resulting row is appended to the corresponding key's Vec.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Fails if the query fails, or the first column is not scalar.
         */
        public function queryGroupedDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an associative array.
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row as an associative array.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an object (`stdClass`).
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row as an object.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row.
         *
         * The result is a `HashMap` where the key is the stringified first column from each row,
         * and the value is the full row, returned as array or object depending on config.
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in associative array mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as associative arrays.
         *
         * @param array|null $parameters
         * @return mixed A dictionary where each key is the first column (as string), and each value is the second column as an associative PHP array.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in object mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as objects.
         *
         * @param array|null $parameters
         * @return mixed A dictionary where each key is the first column (as string), and each value is the second column as a PHP object.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary mapping the first column to the second column.
         *
         * This method expects each result row to contain at least two columns. It converts the first column
         * into a PHP string (used as the key), and the second column into a PHP value (used as the value).
         *
         * @param array|null $parameters
         * @return mixed An associative array (`array<string, mixed>`) where each key is the first column (as string), and the value is the second column.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a PHP string;
         * @throws \Exception - the second column cannot be converted to a PHP value.
         */
        public function queryColumnDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a grouped dictionary where:
         * - the key is the **first column** (must be scalar),
         * - the value is a list of values from the **second column** for each group.
         *
         * This variant uses the driver's default associative array option for JSON values.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionary(?array $parameters = null): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces PHP objects
         * for the second column if it contains JSON objects.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces associative arrays
         * for the second column if it contains JSON objects.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as PHP objects.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as associative arrays.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedDictionaryAssoc(?array $parameters = null): mixed {}

        public function __construct() {}
    }

    /**
     * A reusable prepared SQL query with parameter support. Created using `PgDriver::prepare()`, shares context with original driver.
     */
    class PgPreparedQuery implements Sqlx\PreparedQueryInterface {
        /**
         * @param array|null $parameters
         * @param mixed $column
         * @return array
         */
        public function queryColumn(?array $parameters = null, mixed $column = null): array {}

        /**
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryValue(?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param array|null $parameters
         * @return array
         */
        public function queryAll(?array $parameters = null): array {}

        /**
         * @param array|null $parameters
         * @return array
         */
        public function queryAllAssoc(?array $parameters = null): array {}

        /**
         * @param array|null $parameters
         * @return array
         */
        public function queryAllObj(?array $parameters = null): array {}

        /**
         * @param array|null $parameters
         * @return int
         */
        public function execute(?array $parameters = null): int {}

        /**
         * @param array|null $parameters
         * @return mixed
         */
        public function queryMaybeRow(?array $parameters = null): mixed {}

        /**
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRow(?array $parameters = null): mixed {}

        /**
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRowAssoc(?array $parameters = null): mixed {}

        /**
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRowObj(?array $parameters = null): mixed {}

        /**
         * Executes a query and returns a grouped dictionary (Vec of rows per key).
         *
         * Same as [`queryGroupedDictionary`](crate::Driver::query_grouped_dictionary), but works on a prepared query.
         *
         * The first column is used as the key (must be scalar),
         * and each resulting row is appended to the corresponding key's Vec.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Fails if the query fails, or the first column is not scalar.
         */
        public function queryGroupedDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an associative array.
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row as an associative array.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an object (`stdClass`).
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row as an object.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row.
         *
         * The result is a `HashMap` where the key is the stringified first column from each row,
         * and the value is the full row, returned as array or object depending on config.
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in associative array mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as associative arrays.
         *
         * @param array|null $parameters
         * @return mixed A dictionary where each key is the first column (as string), and each value is the second column as an associative PHP array.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in object mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as objects.
         *
         * @param array|null $parameters
         * @return mixed A dictionary where each key is the first column (as string), and each value is the second column as a PHP object.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary mapping the first column to the second column.
         *
         * This method expects each result row to contain at least two columns. It converts the first column
         * into a PHP string (used as the key), and the second column into a PHP value (used as the value).
         *
         * @param array|null $parameters
         * @return mixed An associative array (`array<string, mixed>`) where each key is the first column (as string), and the value is the second column.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a PHP string;
         * @throws \Exception - the second column cannot be converted to a PHP value.
         */
        public function queryColumnDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a grouped dictionary where:
         * - the key is the **first column** (must be scalar),
         * - the value is a list of values from the **second column** for each group.
         *
         * This variant uses the driver's default associative array option for JSON values.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionary(?array $parameters = null): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces PHP objects
         * for the second column if it contains JSON objects.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces associative arrays
         * for the second column if it contains JSON objects.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as PHP objects.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as associative arrays.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedDictionaryAssoc(?array $parameters = null): mixed {}

        public function __construct() {}
    }

    /**
     * Creates an OR clause from an array of conditions.
     *
     * This function is exposed to PHP as `Sqlx\OR_()` and allows building
     * complex boolean expressions with OR logic.
     *
     * @param array $or An array of conditions, where each condition can be:
     * @return \Sqlx\OrClause An `OrClause` instance that can be used in WHERE clauses.
     */
    function OR_(array $or): \Sqlx\OrClause {}

    /**
     * Database driver for executing SQL queries with advanced features.
     *
     * This class supports:
     * - **Prepared queries**: Cached AST parsing for repeated queries
     * - **Persistent connections**: Reuse connections across PHP requests
     * - **Augmented SQL**: Conditional blocks, IN clause optimization, pagination
     * - **Transactions**: Both callback-based and imperative styles
     * - **Query builders**: Fluent API for constructing queries
     */
    class MssqlDriver implements Sqlx\DriverInterface {
        /**
         * @param mixed $param
         * @return string
         */
        public function quote(mixed $param): string {}

        /**
         * @param mixed $param
         * @return string
         */
        public function quoteLike(mixed $param): string {}

        /**
         * @param string $name
         * @return string
         */
        public function quoteIdentifier(string $name): string {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return array
         */
        public function queryColumn(string $query, ?array $parameters = null, mixed $column = null): array {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return array
         */
        public function queryColumnAssoc(string $query, ?array $parameters = null, mixed $column = null): array {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return array
         */
        public function queryColumnObj(string $query, ?array $parameters = null, mixed $column = null): array {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryMaybeValue(string $query, ?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryMaybeValueAssoc(string $query, ?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryMaybeValueObj(string $query, ?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryValue(string $query, ?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryValueAssoc(string $query, ?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryValueObj(string $query, ?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return array
         */
        public function queryAll(string $query, ?array $parameters = null): array {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return array
         */
        public function queryAllAssoc(string $query, ?array $parameters = null): array {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return array
         */
        public function queryAllObj(string $query, ?array $parameters = null): array {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return int
         */
        public function execute(string $query, ?array $parameters = null): int {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryDictionary(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryDictionaryAssoc(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryDictionaryObj(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryMaybeRow(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryMaybeRowAssoc(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryMaybeRowObj(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRow(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRowAssoc(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRowObj(string $query, ?array $parameters = null): mixed {}

        /**
         * @return bool
         */
        public function assocArrays(): bool {}

        /**
         * @return bool
         */
        public function isClosed(): bool {}

        /**
         * @return void
         */
        public function close(): void {}

        /**
         * Begins a SQL transaction, optionally executing a callable within it.
         *
         * This method supports two modes of operation:
         *
         * **Mode 1: Callback-based (automatic commit/rollback)**
         * ```php
         * $driver->begin(function($driver) {
         *     $driver->execute('INSERT INTO users (name) VALUES (?)', ['John']);
         *     return true; // true = commit, false = rollback
         * });
         * ```
         *
         * **Mode 2: Imperative (manual commit/rollback)**
         * ```php
         * $driver->begin();
         * try {
         *     $driver->execute('INSERT INTO users (name) VALUES (?)', ['John']);
         *     $driver->commit();
         * } catch (\Exception $e) {
         *     $driver->rollback();
         *     throw $e;
         * }
         * ```
         *
         * @param callable|null $callable
         * @return bool
         */
        public function begin(?callable $callable = null): bool {}

        /**
         * Commits the current ongoing transaction.
         *
         * This method should be called after `begin()` was called without a callable.
         * It commits all changes made during the transaction and removes the transaction
         * from the stack.
         *
         * @return void
         */
        public function commit(): void {}

        /**
         * Constructs a new `SQLx` driver instance.
         *
         * @param mixed $url_or_options
         */
        public function __construct(mixed $url_or_options) {}

        /**
         * Creates a prepared query object with the given SQL string.
         *
         * @param string $query SQL query string to prepare
         * @return \Sqlx\Driver\MssqlPreparedQuery Prepared query object
         */
        public function prepare(string $query): \Sqlx\Driver\MssqlPreparedQuery {}

        /**
         * Creates a query builder object
         *
         * @return \Sqlx\MssqlReadQueryBuilder Query builder object
         */
        public function readBuilder(): \Sqlx\MssqlReadQueryBuilder {}

        /**
         * Creates a query builder object
         *
         * @return \Sqlx\MssqlWriteQueryBuilder Query builder object
         */
        public function builder(): \Sqlx\MssqlWriteQueryBuilder {}

        /**
         * Creates a transaction savepoint with the given name.
         *
         * @param string $savepoint
         * @return void
         */
        public function savepoint(string $savepoint): void {}

        /**
         * Describes table columns with their types and metadata.
         *
         * Returns information about each column in the specified table, including
         * name, type, nullability, default value, and ordinal position.
         *
         * @param string $table
         * @param string|null $schema
         * @return array An array of associative arrays, each containing: - `name`: Column name (string) - `type`: Database-specific column type (string, e.g., "varchar(255)", "int") - `nullable`: Whether the column allows NULL values (bool) - `default`: Default value for the column (string|null) - `ordinal`: Column position, 1-based (int)
         */
        public function describeTable(string $table, ?string $schema = null): array {}

        /**
         * Executes a callback with a pinned connection from the pool.
         *
         * All queries executed within the callback will use the same database connection,
         * which is required for session-scoped operations like:
         * - `LAST_INSERT_ID()` in `MySQL`
         * - Temporary tables
         * - Session variables
         * - Advisory locks
         *
         * Unlike `begin()`, this does NOT start a database transaction - each query is
         * auto-committed. Use `begin()` if you need transactional semantics.
         *
         * @param callable $callable
         * @return mixed The value returned by the callback.
         */
        public function withConnection(callable $callable): mixed {}

        /**
         * Executes an SQL query and returns a dictionary grouping rows by the first column.
         *
         * Each row in the result must contain at least one column. The **first column** is used as the **key**, and the
         * **entire row** is converted to a PHP value and added to the list associated with that key.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed A `HashMap<String, Vec<Zval>>` mapping each unique value of the first column to a `Vec` of corresponding rows.
         * @throws \Exception Returns an error if:
         * @throws \Exception - The query fails to render or execute.
         * @throws \Exception - The first column in any row is `NULL` or cannot be converted to a PHP string.
         * @throws \Exception - Any row cannot be fully converted to a PHP value.
         */
        public function queryGroupedDictionary(string $query, ?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a dictionary mapping the first column to the second column.
         *
         * This method expects each result row to contain at least two columns. It converts the first column
         * into a PHP string (used as the key), and the second column into a PHP value (used as the value).
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed An associative array (`array<string, mixed>`) where each key is the first column (as string), and the value is the second column.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to render or execute;
         * @throws \Exception - the first column cannot be converted to a PHP string;
         * @throws \Exception - the second column cannot be converted to a PHP value.
         */
        public function queryColumnDictionary(string $query, ?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a dictionary using associative array mode for values.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as associative arrays.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed Dictionary where each key is the first column, and the value is the second column converted into an associative PHP array.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryAssoc(string $query, ?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a dictionary using object mode for values.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as PHP objects.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed Dictionary where each key is the first column, and the value is the second column converted into a PHP object.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryObj(string $query, ?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a grouped dictionary where:
         * - the key is the **first column** (must be scalar),
         * - the value is a list of values from the **second column** for each group.
         *
         * Useful for queries that logically group rows, such as:
         * ```sql
         * SELECT category, product_name FROM products
         * ```
         * Result:
         * ```php
         * [
         *   "Books" => ["Rust in Action", "The Pragmatic Programmer"],
         *   "Gadgets" => ["Raspberry Pi"]
         * ]
         * ```
         *
         * Throws an error if the first column is not a string.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedColumnDictionary(string $query, ?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a lazy `QueryResult` iterator with rows as associative arrays.
         *
         * Same as `query()`, but forces rows to be returned as associative arrays
         * regardless of the driver's default configuration.
         *
         * @param string $query SQL query string
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\MssqlQueryResult A `QueryResult` object with rows as associative arrays
         */
        public function queryAssoc(string $query, ?array $parameters = null, ?int $batch_size = null): \Sqlx\MssqlQueryResult {}

        /**
         * Executes an SQL query and returns a lazy `QueryResult` iterator with rows as objects.
         *
         * Same as `query()`, but forces rows to be returned as PHP objects
         * regardless of the driver's default configuration.
         *
         * @param string $query SQL query string
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\MssqlQueryResult A `QueryResult` object with rows as PHP objects
         */
        public function queryObj(string $query, ?array $parameters = null, ?int $batch_size = null): \Sqlx\MssqlQueryResult {}

        /**
         * Executes an SQL query and returns a lazy `QueryResult` iterator.
         *
         * This method returns a `QueryResult` object that implements PHP's `Iterator`
         * interface, streaming rows from the database as you iterate.
         * This is memory-efficient for large result sets.
         *
         * @param string $query SQL query string
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\MssqlQueryResult A `QueryResult` object implementing `Iterator`
         */
        public function query(string $query, ?array $parameters = null, ?int $batch_size = null): \Sqlx\MssqlQueryResult {}

        /**
         * Executes an SQL query and returns the rendered query and its parameters.
         *
         * This method does not execute the query but returns the SQL string with placeholders
         * and the bound parameter values for debugging or logging purposes.
         *
         * @param string $query SQL query string
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array A list where the first element is the rendered SQL query (string), and the second is an array of bound values
         */
        public function dry(string $query, ?array $parameters = null): array {}

        /**
         * Inserts a row into the given table using a map of columns.
         *
         * @param string $table Table name
         * @param array $row Map of column names to values
         * @return int Number of inserted rows
         */
        public function insert(string $table, array $row): int {}

        /**
         * Inserts a row or updates it if a conflict occurs on the specified columns.
         *
         * This method generates database-specific SQL for upsert operations:
         * - **`PostgreSQL`**: `INSERT ... ON CONFLICT (cols) DO UPDATE SET ...`
         * - **`MySQL`**: `INSERT ... ON DUPLICATE KEY UPDATE ...`
         * - **`MSSQL`**: Not currently supported (returns an error)
         *
         * @param string $table Table name to insert into
         * @param array $row Map of column names to values for the row to insert
         * @param array $conflict_columns Columns that form the unique constraint for conflict detection
         * @param array|null $update_columns Optional list of columns to update on conflict.
         * @return int Number of affected rows (1 for insert, 2 for update on some databases)
         */
        public function upsert(string $table, array $row, array $conflict_columns, ?array $update_columns = null): int {}

        /**
         * Inserts multiple rows into the given table in a single statement.
         *
         * All rows must have the same columns (determined by the first row).
         * Missing columns in subsequent rows will use `NULL`.
         *
         * @param string $table Table name
         * @param array $rows Vector of maps, each representing a row (column name → value)
         * @return int Number of inserted rows
         */
        public function insertMany(string $table, array $rows): int {}

        /**
         * Registers a callback to be invoked after each query execution.
         *
         * The callback receives:
         * - `string $sql` - The rendered SQL query with placeholders
         * - `string $sqlInline` - The SQL query with inlined parameter values (for logging)
         * - `float $durationMs` - Execution time in milliseconds
         *
         * @param mixed $callback
         * @return void
         */
        public function onQuery(mixed $callback): void {}

        /**
         * Releases a previously created savepoint, making it no longer available.
         *
         * @param string $savepoint
         * @return void
         */
        public function releaseSavepoint(string $savepoint): void {}

        /**
         * Returns true if read replicas are configured for this driver.
         *
         * When read replicas are configured, SELECT queries are automatically
         * routed to replicas using round-robin selection, while write operations
         * (INSERT/UPDATE/DELETE) always go to the primary.
         *
         * @return bool
         */
        public function hasReadReplicas(): bool {}

        /**
         * Rolls back the current ongoing transaction.
         *
         * This method should be called after `begin()` was called without a callable.
         * It discards all changes made during the transaction and removes the transaction
         * from the stack.
         *
         * @return void
         */
        public function rollback(): void {}

        /**
         * Rolls back the current transaction to a previously created savepoint.
         *
         * @param string $savepoint
         * @return void
         */
        public function rollbackToSavepoint(string $savepoint): void {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces PHP objects
         * for the second column if it's a JSON object.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedColumnDictionaryObj(string $query, ?array $parameters = null): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces associative arrays
         * for the second column if it's a JSON object.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedColumnDictionaryAssoc(string $query, ?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces each row to be represented as a PHP associative array.
         *
         * This overrides the driver’s default associative/object mode for this call only.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception - If the first column is not convertible to string.
         * @throws \Exception - If any row fails to convert to an associative array.
         */
        public function queryGroupedDictionaryAssoc(string $query, ?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces each row to be represented as a PHP object.
         *
         * This overrides the driver’s default associative/object mode for this call only.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception - If the first column is not convertible to string.
         * @throws \Exception - If any row fails to convert to an object.
         */
        public function queryGroupedDictionaryObj(string $query, ?array $parameters = null): mixed {}

        /**
         * Sets client metadata for this connection.
         *
         * The metadata is formatted and appended to the application name,
         * making it visible in database monitoring tools. This is useful for tracking
         * request IDs, user IDs, or other debugging information.
         *
         * @param string $application_name
         * @param array $info
         * @return void
         */
        public function setClientInfo(string $application_name, array $info): void {}

        /**
         * Sets the application name for this connection.
         *
         * This helps identify the connection in database monitoring tools:
         * - `PostgreSQL`: Visible in `pg_stat_activity.application_name`
         * - `MySQL`: Stored in session variable `@sqlx_application_name`
         * - `MSSQL`: Stored in session context via `sp_set_session_context`
         *
         * @param string $name
         * @return void
         */
        public function setApplicationName(string $name): void {}
    }

    /**
     * Database driver for executing SQL queries with advanced features.
     *
     * This class supports:
     * - **Prepared queries**: Cached AST parsing for repeated queries
     * - **Persistent connections**: Reuse connections across PHP requests
     * - **Augmented SQL**: Conditional blocks, IN clause optimization, pagination
     * - **Transactions**: Both callback-based and imperative styles
     * - **Query builders**: Fluent API for constructing queries
     */
    class MySqlDriver implements Sqlx\DriverInterface {
        /**
         * @param mixed $param
         * @return string
         */
        public function quote(mixed $param): string {}

        /**
         * @param mixed $param
         * @return string
         */
        public function quoteLike(mixed $param): string {}

        /**
         * @param string $name
         * @return string
         */
        public function quoteIdentifier(string $name): string {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return array
         */
        public function queryColumn(string $query, ?array $parameters = null, mixed $column = null): array {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return array
         */
        public function queryColumnAssoc(string $query, ?array $parameters = null, mixed $column = null): array {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return array
         */
        public function queryColumnObj(string $query, ?array $parameters = null, mixed $column = null): array {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryMaybeValue(string $query, ?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryMaybeValueAssoc(string $query, ?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryMaybeValueObj(string $query, ?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryValue(string $query, ?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryValueAssoc(string $query, ?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryValueObj(string $query, ?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return array
         */
        public function queryAll(string $query, ?array $parameters = null): array {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return array
         */
        public function queryAllAssoc(string $query, ?array $parameters = null): array {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return array
         */
        public function queryAllObj(string $query, ?array $parameters = null): array {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return int
         */
        public function execute(string $query, ?array $parameters = null): int {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryDictionary(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryDictionaryAssoc(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryDictionaryObj(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryMaybeRow(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryMaybeRowAssoc(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryMaybeRowObj(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRow(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRowAssoc(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRowObj(string $query, ?array $parameters = null): mixed {}

        /**
         * @return bool
         */
        public function assocArrays(): bool {}

        /**
         * @return bool
         */
        public function isClosed(): bool {}

        /**
         * @return void
         */
        public function close(): void {}

        /**
         * Begins a SQL transaction, optionally executing a callable within it.
         *
         * This method supports two modes of operation:
         *
         * **Mode 1: Callback-based (automatic commit/rollback)**
         * ```php
         * $driver->begin(function($driver) {
         *     $driver->execute('INSERT INTO users (name) VALUES (?)', ['John']);
         *     return true; // true = commit, false = rollback
         * });
         * ```
         *
         * **Mode 2: Imperative (manual commit/rollback)**
         * ```php
         * $driver->begin();
         * try {
         *     $driver->execute('INSERT INTO users (name) VALUES (?)', ['John']);
         *     $driver->commit();
         * } catch (\Exception $e) {
         *     $driver->rollback();
         *     throw $e;
         * }
         * ```
         *
         * @param callable|null $callable
         * @return bool
         */
        public function begin(?callable $callable = null): bool {}

        /**
         * Commits the current ongoing transaction.
         *
         * This method should be called after `begin()` was called without a callable.
         * It commits all changes made during the transaction and removes the transaction
         * from the stack.
         *
         * @return void
         */
        public function commit(): void {}

        /**
         * Constructs a new `SQLx` driver instance.
         *
         * @param mixed $url_or_options
         */
        public function __construct(mixed $url_or_options) {}

        /**
         * Creates a prepared query object with the given SQL string.
         *
         * @param string $query SQL query string to prepare
         * @return \Sqlx\MySqlPreparedQuery Prepared query object
         */
        public function prepare(string $query): \Sqlx\MySqlPreparedQuery {}

        /**
         * Creates a query builder object
         *
         * @return \Sqlx\MySqlReadQueryBuilder Query builder object
         */
        public function readBuilder(): \Sqlx\MySqlReadQueryBuilder {}

        /**
         * Creates a query builder object
         *
         * @return \Sqlx\MySqlWriteQueryBuilder Query builder object
         */
        public function builder(): \Sqlx\MySqlWriteQueryBuilder {}

        /**
         * Creates a transaction savepoint with the given name.
         *
         * @param string $savepoint
         * @return void
         */
        public function savepoint(string $savepoint): void {}

        /**
         * Describes table columns with their types and metadata.
         *
         * Returns information about each column in the specified table, including
         * name, type, nullability, default value, and ordinal position.
         *
         * @param string $table
         * @param string|null $schema
         * @return array An array of associative arrays, each containing: - `name`: Column name (string) - `type`: Database-specific column type (string, e.g., "varchar(255)", "int") - `nullable`: Whether the column allows NULL values (bool) - `default`: Default value for the column (string|null) - `ordinal`: Column position, 1-based (int)
         */
        public function describeTable(string $table, ?string $schema = null): array {}

        /**
         * Executes a callback with a pinned connection from the pool.
         *
         * All queries executed within the callback will use the same database connection,
         * which is required for session-scoped operations like:
         * - `LAST_INSERT_ID()` in `MySQL`
         * - Temporary tables
         * - Session variables
         * - Advisory locks
         *
         * Unlike `begin()`, this does NOT start a database transaction - each query is
         * auto-committed. Use `begin()` if you need transactional semantics.
         *
         * @param callable $callable
         * @return mixed The value returned by the callback.
         */
        public function withConnection(callable $callable): mixed {}

        /**
         * Executes an SQL query and returns a dictionary grouping rows by the first column.
         *
         * Each row in the result must contain at least one column. The **first column** is used as the **key**, and the
         * **entire row** is converted to a PHP value and added to the list associated with that key.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed A `HashMap<String, Vec<Zval>>` mapping each unique value of the first column to a `Vec` of corresponding rows.
         * @throws \Exception Returns an error if:
         * @throws \Exception - The query fails to render or execute.
         * @throws \Exception - The first column in any row is `NULL` or cannot be converted to a PHP string.
         * @throws \Exception - Any row cannot be fully converted to a PHP value.
         */
        public function queryGroupedDictionary(string $query, ?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a dictionary mapping the first column to the second column.
         *
         * This method expects each result row to contain at least two columns. It converts the first column
         * into a PHP string (used as the key), and the second column into a PHP value (used as the value).
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed An associative array (`array<string, mixed>`) where each key is the first column (as string), and the value is the second column.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to render or execute;
         * @throws \Exception - the first column cannot be converted to a PHP string;
         * @throws \Exception - the second column cannot be converted to a PHP value.
         */
        public function queryColumnDictionary(string $query, ?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a dictionary using associative array mode for values.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as associative arrays.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed Dictionary where each key is the first column, and the value is the second column converted into an associative PHP array.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryAssoc(string $query, ?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a dictionary using object mode for values.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as PHP objects.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed Dictionary where each key is the first column, and the value is the second column converted into a PHP object.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryObj(string $query, ?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a grouped dictionary where:
         * - the key is the **first column** (must be scalar),
         * - the value is a list of values from the **second column** for each group.
         *
         * Useful for queries that logically group rows, such as:
         * ```sql
         * SELECT category, product_name FROM products
         * ```
         * Result:
         * ```php
         * [
         *   "Books" => ["Rust in Action", "The Pragmatic Programmer"],
         *   "Gadgets" => ["Raspberry Pi"]
         * ]
         * ```
         *
         * Throws an error if the first column is not a string.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedColumnDictionary(string $query, ?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a lazy `QueryResult` iterator with rows as associative arrays.
         *
         * Same as `query()`, but forces rows to be returned as associative arrays
         * regardless of the driver's default configuration.
         *
         * @param string $query SQL query string
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\MySqlQueryResult A `QueryResult` object with rows as associative arrays
         */
        public function queryAssoc(string $query, ?array $parameters = null, ?int $batch_size = null): \Sqlx\MySqlQueryResult {}

        /**
         * Executes an SQL query and returns a lazy `QueryResult` iterator with rows as objects.
         *
         * Same as `query()`, but forces rows to be returned as PHP objects
         * regardless of the driver's default configuration.
         *
         * @param string $query SQL query string
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\MySqlQueryResult A `QueryResult` object with rows as PHP objects
         */
        public function queryObj(string $query, ?array $parameters = null, ?int $batch_size = null): \Sqlx\MySqlQueryResult {}

        /**
         * Executes an SQL query and returns a lazy `QueryResult` iterator.
         *
         * This method returns a `QueryResult` object that implements PHP's `Iterator`
         * interface, streaming rows from the database as you iterate.
         * This is memory-efficient for large result sets.
         *
         * @param string $query SQL query string
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\MySqlQueryResult A `QueryResult` object implementing `Iterator`
         */
        public function query(string $query, ?array $parameters = null, ?int $batch_size = null): \Sqlx\MySqlQueryResult {}

        /**
         * Executes an SQL query and returns the rendered query and its parameters.
         *
         * This method does not execute the query but returns the SQL string with placeholders
         * and the bound parameter values for debugging or logging purposes.
         *
         * @param string $query SQL query string
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array A list where the first element is the rendered SQL query (string), and the second is an array of bound values
         */
        public function dry(string $query, ?array $parameters = null): array {}

        /**
         * Inserts a row into the given table using a map of columns.
         *
         * @param string $table Table name
         * @param array $row Map of column names to values
         * @return int Number of inserted rows
         */
        public function insert(string $table, array $row): int {}

        /**
         * Inserts a row or updates it if a conflict occurs on the specified columns.
         *
         * This method generates database-specific SQL for upsert operations:
         * - **`PostgreSQL`**: `INSERT ... ON CONFLICT (cols) DO UPDATE SET ...`
         * - **`MySQL`**: `INSERT ... ON DUPLICATE KEY UPDATE ...`
         * - **`MSSQL`**: Not currently supported (returns an error)
         *
         * @param string $table Table name to insert into
         * @param array $row Map of column names to values for the row to insert
         * @param array $conflict_columns Columns that form the unique constraint for conflict detection
         * @param array|null $update_columns Optional list of columns to update on conflict.
         * @return int Number of affected rows (1 for insert, 2 for update on some databases)
         */
        public function upsert(string $table, array $row, array $conflict_columns, ?array $update_columns = null): int {}

        /**
         * Inserts multiple rows into the given table in a single statement.
         *
         * All rows must have the same columns (determined by the first row).
         * Missing columns in subsequent rows will use `NULL`.
         *
         * @param string $table Table name
         * @param array $rows Vector of maps, each representing a row (column name → value)
         * @return int Number of inserted rows
         */
        public function insertMany(string $table, array $rows): int {}

        /**
         * Registers a callback to be invoked after each query execution.
         *
         * The callback receives:
         * - `string $sql` - The rendered SQL query with placeholders
         * - `string $sqlInline` - The SQL query with inlined parameter values (for logging)
         * - `float $durationMs` - Execution time in milliseconds
         *
         * @param mixed $callback
         * @return void
         */
        public function onQuery(mixed $callback): void {}

        /**
         * Releases a previously created savepoint, making it no longer available.
         *
         * @param string $savepoint
         * @return void
         */
        public function releaseSavepoint(string $savepoint): void {}

        /**
         * Returns true if read replicas are configured for this driver.
         *
         * When read replicas are configured, SELECT queries are automatically
         * routed to replicas using round-robin selection, while write operations
         * (INSERT/UPDATE/DELETE) always go to the primary.
         *
         * @return bool
         */
        public function hasReadReplicas(): bool {}

        /**
         * Rolls back the current ongoing transaction.
         *
         * This method should be called after `begin()` was called without a callable.
         * It discards all changes made during the transaction and removes the transaction
         * from the stack.
         *
         * @return void
         */
        public function rollback(): void {}

        /**
         * Rolls back the current transaction to a previously created savepoint.
         *
         * @param string $savepoint
         * @return void
         */
        public function rollbackToSavepoint(string $savepoint): void {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces PHP objects
         * for the second column if it's a JSON object.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedColumnDictionaryObj(string $query, ?array $parameters = null): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces associative arrays
         * for the second column if it's a JSON object.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedColumnDictionaryAssoc(string $query, ?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces each row to be represented as a PHP associative array.
         *
         * This overrides the driver’s default associative/object mode for this call only.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception - If the first column is not convertible to string.
         * @throws \Exception - If any row fails to convert to an associative array.
         */
        public function queryGroupedDictionaryAssoc(string $query, ?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces each row to be represented as a PHP object.
         *
         * This overrides the driver’s default associative/object mode for this call only.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception - If the first column is not convertible to string.
         * @throws \Exception - If any row fails to convert to an object.
         */
        public function queryGroupedDictionaryObj(string $query, ?array $parameters = null): mixed {}

        /**
         * Sets client metadata for this connection.
         *
         * The metadata is formatted and appended to the application name,
         * making it visible in database monitoring tools. This is useful for tracking
         * request IDs, user IDs, or other debugging information.
         *
         * @param string $application_name
         * @param array $info
         * @return void
         */
        public function setClientInfo(string $application_name, array $info): void {}

        /**
         * Sets the application name for this connection.
         *
         * This helps identify the connection in database monitoring tools:
         * - `PostgreSQL`: Visible in `pg_stat_activity.application_name`
         * - `MySQL`: Stored in session variable `@sqlx_application_name`
         * - `MSSQL`: Stored in session context via `sp_set_session_context`
         *
         * @param string $name
         * @return void
         */
        public function setApplicationName(string $name): void {}
    }

    /**
     * Database driver for executing SQL queries with advanced features.
     *
     * This class supports:
     * - **Prepared queries**: Cached AST parsing for repeated queries
     * - **Persistent connections**: Reuse connections across PHP requests
     * - **Augmented SQL**: Conditional blocks, IN clause optimization, pagination
     * - **Transactions**: Both callback-based and imperative styles
     * - **Query builders**: Fluent API for constructing queries
     */
    class PgDriver implements Sqlx\DriverInterface {
        /**
         * @param mixed $param
         * @return string
         */
        public function quote(mixed $param): string {}

        /**
         * @param mixed $param
         * @return string
         */
        public function quoteLike(mixed $param): string {}

        /**
         * @param string $name
         * @return string
         */
        public function quoteIdentifier(string $name): string {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return array
         */
        public function queryColumn(string $query, ?array $parameters = null, mixed $column = null): array {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return array
         */
        public function queryColumnAssoc(string $query, ?array $parameters = null, mixed $column = null): array {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return array
         */
        public function queryColumnObj(string $query, ?array $parameters = null, mixed $column = null): array {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryMaybeValue(string $query, ?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryMaybeValueAssoc(string $query, ?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryMaybeValueObj(string $query, ?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryValue(string $query, ?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryValueAssoc(string $query, ?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryValueObj(string $query, ?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return array
         */
        public function queryAll(string $query, ?array $parameters = null): array {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return array
         */
        public function queryAllAssoc(string $query, ?array $parameters = null): array {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return array
         */
        public function queryAllObj(string $query, ?array $parameters = null): array {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return int
         */
        public function execute(string $query, ?array $parameters = null): int {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryDictionary(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryDictionaryAssoc(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryDictionaryObj(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryMaybeRow(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryMaybeRowAssoc(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryMaybeRowObj(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRow(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRowAssoc(string $query, ?array $parameters = null): mixed {}

        /**
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRowObj(string $query, ?array $parameters = null): mixed {}

        /**
         * @return bool
         */
        public function assocArrays(): bool {}

        /**
         * @return bool
         */
        public function isClosed(): bool {}

        /**
         * @return void
         */
        public function close(): void {}

        /**
         * Begins a SQL transaction, optionally executing a callable within it.
         *
         * This method supports two modes of operation:
         *
         * **Mode 1: Callback-based (automatic commit/rollback)**
         * ```php
         * $driver->begin(function($driver) {
         *     $driver->execute('INSERT INTO users (name) VALUES (?)', ['John']);
         *     return true; // true = commit, false = rollback
         * });
         * ```
         *
         * **Mode 2: Imperative (manual commit/rollback)**
         * ```php
         * $driver->begin();
         * try {
         *     $driver->execute('INSERT INTO users (name) VALUES (?)', ['John']);
         *     $driver->commit();
         * } catch (\Exception $e) {
         *     $driver->rollback();
         *     throw $e;
         * }
         * ```
         *
         * @param callable|null $callable
         * @return bool
         */
        public function begin(?callable $callable = null): bool {}

        /**
         * Commits the current ongoing transaction.
         *
         * This method should be called after `begin()` was called without a callable.
         * It commits all changes made during the transaction and removes the transaction
         * from the stack.
         *
         * @return void
         */
        public function commit(): void {}

        /**
         * Constructs a new `SQLx` driver instance.
         *
         * @param mixed $url_or_options
         */
        public function __construct(mixed $url_or_options) {}

        /**
         * Creates a prepared query object with the given SQL string.
         *
         * @param string $query SQL query string to prepare
         * @return \Sqlx\PgPreparedQuery Prepared query object
         */
        public function prepare(string $query): \Sqlx\PgPreparedQuery {}

        /**
         * Creates a query builder object
         *
         * @return \Sqlx\PgReadQueryBuilder Query builder object
         */
        public function readBuilder(): \Sqlx\PgReadQueryBuilder {}

        /**
         * Creates a query builder object
         *
         * @return \Sqlx\PgWriteQueryBuilder Query builder object
         */
        public function builder(): \Sqlx\PgWriteQueryBuilder {}

        /**
         * Creates a transaction savepoint with the given name.
         *
         * @param string $savepoint
         * @return void
         */
        public function savepoint(string $savepoint): void {}

        /**
         * Describes table columns with their types and metadata.
         *
         * Returns information about each column in the specified table, including
         * name, type, nullability, default value, and ordinal position.
         *
         * @param string $table
         * @param string|null $schema
         * @return array An array of associative arrays, each containing: - `name`: Column name (string) - `type`: Database-specific column type (string, e.g., "varchar(255)", "int") - `nullable`: Whether the column allows NULL values (bool) - `default`: Default value for the column (string|null) - `ordinal`: Column position, 1-based (int)
         */
        public function describeTable(string $table, ?string $schema = null): array {}

        /**
         * Executes a callback with a pinned connection from the pool.
         *
         * All queries executed within the callback will use the same database connection,
         * which is required for session-scoped operations like:
         * - `LAST_INSERT_ID()` in `MySQL`
         * - Temporary tables
         * - Session variables
         * - Advisory locks
         *
         * Unlike `begin()`, this does NOT start a database transaction - each query is
         * auto-committed. Use `begin()` if you need transactional semantics.
         *
         * @param callable $callable
         * @return mixed The value returned by the callback.
         */
        public function withConnection(callable $callable): mixed {}

        /**
         * Executes an SQL query and returns a dictionary grouping rows by the first column.
         *
         * Each row in the result must contain at least one column. The **first column** is used as the **key**, and the
         * **entire row** is converted to a PHP value and added to the list associated with that key.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed A `HashMap<String, Vec<Zval>>` mapping each unique value of the first column to a `Vec` of corresponding rows.
         * @throws \Exception Returns an error if:
         * @throws \Exception - The query fails to render or execute.
         * @throws \Exception - The first column in any row is `NULL` or cannot be converted to a PHP string.
         * @throws \Exception - Any row cannot be fully converted to a PHP value.
         */
        public function queryGroupedDictionary(string $query, ?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a dictionary mapping the first column to the second column.
         *
         * This method expects each result row to contain at least two columns. It converts the first column
         * into a PHP string (used as the key), and the second column into a PHP value (used as the value).
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed An associative array (`array<string, mixed>`) where each key is the first column (as string), and the value is the second column.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to render or execute;
         * @throws \Exception - the first column cannot be converted to a PHP string;
         * @throws \Exception - the second column cannot be converted to a PHP value.
         */
        public function queryColumnDictionary(string $query, ?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a dictionary using associative array mode for values.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as associative arrays.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed Dictionary where each key is the first column, and the value is the second column converted into an associative PHP array.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryAssoc(string $query, ?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a dictionary using object mode for values.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as PHP objects.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed Dictionary where each key is the first column, and the value is the second column converted into a PHP object.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryObj(string $query, ?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a grouped dictionary where:
         * - the key is the **first column** (must be scalar),
         * - the value is a list of values from the **second column** for each group.
         *
         * Useful for queries that logically group rows, such as:
         * ```sql
         * SELECT category, product_name FROM products
         * ```
         * Result:
         * ```php
         * [
         *   "Books" => ["Rust in Action", "The Pragmatic Programmer"],
         *   "Gadgets" => ["Raspberry Pi"]
         * ]
         * ```
         *
         * Throws an error if the first column is not a string.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedColumnDictionary(string $query, ?array $parameters = null): mixed {}

        /**
         * Executes an SQL query and returns a lazy `QueryResult` iterator with rows as associative arrays.
         *
         * Same as `query()`, but forces rows to be returned as associative arrays
         * regardless of the driver's default configuration.
         *
         * @param string $query SQL query string
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\PgQueryResult A `QueryResult` object with rows as associative arrays
         */
        public function queryAssoc(string $query, ?array $parameters = null, ?int $batch_size = null): \Sqlx\PgQueryResult {}

        /**
         * Executes an SQL query and returns a lazy `QueryResult` iterator with rows as objects.
         *
         * Same as `query()`, but forces rows to be returned as PHP objects
         * regardless of the driver's default configuration.
         *
         * @param string $query SQL query string
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\PgQueryResult A `QueryResult` object with rows as PHP objects
         */
        public function queryObj(string $query, ?array $parameters = null, ?int $batch_size = null): \Sqlx\PgQueryResult {}

        /**
         * Executes an SQL query and returns a lazy `QueryResult` iterator.
         *
         * This method returns a `QueryResult` object that implements PHP's `Iterator`
         * interface, streaming rows from the database as you iterate.
         * This is memory-efficient for large result sets.
         *
         * @param string $query SQL query string
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @param int|null $batch_size Optional buffer size for streaming (default: 100)
         * @return \Sqlx\PgQueryResult A `QueryResult` object implementing `Iterator`
         */
        public function query(string $query, ?array $parameters = null, ?int $batch_size = null): \Sqlx\PgQueryResult {}

        /**
         * Executes an SQL query and returns the rendered query and its parameters.
         *
         * This method does not execute the query but returns the SQL string with placeholders
         * and the bound parameter values for debugging or logging purposes.
         *
         * @param string $query SQL query string
         * @param array|null $parameters Optional array of indexed/named parameters to bind.
         * @return array A list where the first element is the rendered SQL query (string), and the second is an array of bound values
         */
        public function dry(string $query, ?array $parameters = null): array {}

        /**
         * Inserts a row into the given table using a map of columns.
         *
         * @param string $table Table name
         * @param array $row Map of column names to values
         * @return int Number of inserted rows
         */
        public function insert(string $table, array $row): int {}

        /**
         * Inserts a row or updates it if a conflict occurs on the specified columns.
         *
         * This method generates database-specific SQL for upsert operations:
         * - **`PostgreSQL`**: `INSERT ... ON CONFLICT (cols) DO UPDATE SET ...`
         * - **`MySQL`**: `INSERT ... ON DUPLICATE KEY UPDATE ...`
         * - **`MSSQL`**: Not currently supported (returns an error)
         *
         * @param string $table Table name to insert into
         * @param array $row Map of column names to values for the row to insert
         * @param array $conflict_columns Columns that form the unique constraint for conflict detection
         * @param array|null $update_columns Optional list of columns to update on conflict.
         * @return int Number of affected rows (1 for insert, 2 for update on some databases)
         */
        public function upsert(string $table, array $row, array $conflict_columns, ?array $update_columns = null): int {}

        /**
         * Inserts multiple rows into the given table in a single statement.
         *
         * All rows must have the same columns (determined by the first row).
         * Missing columns in subsequent rows will use `NULL`.
         *
         * @param string $table Table name
         * @param array $rows Vector of maps, each representing a row (column name → value)
         * @return int Number of inserted rows
         */
        public function insertMany(string $table, array $rows): int {}

        /**
         * Registers a callback to be invoked after each query execution.
         *
         * The callback receives:
         * - `string $sql` - The rendered SQL query with placeholders
         * - `string $sqlInline` - The SQL query with inlined parameter values (for logging)
         * - `float $durationMs` - Execution time in milliseconds
         *
         * @param mixed $callback
         * @return void
         */
        public function onQuery(mixed $callback): void {}

        /**
         * Releases a previously created savepoint, making it no longer available.
         *
         * @param string $savepoint
         * @return void
         */
        public function releaseSavepoint(string $savepoint): void {}

        /**
         * Returns true if read replicas are configured for this driver.
         *
         * When read replicas are configured, SELECT queries are automatically
         * routed to replicas using round-robin selection, while write operations
         * (INSERT/UPDATE/DELETE) always go to the primary.
         *
         * @return bool
         */
        public function hasReadReplicas(): bool {}

        /**
         * Rolls back the current ongoing transaction.
         *
         * This method should be called after `begin()` was called without a callable.
         * It discards all changes made during the transaction and removes the transaction
         * from the stack.
         *
         * @return void
         */
        public function rollback(): void {}

        /**
         * Rolls back the current transaction to a previously created savepoint.
         *
         * @param string $savepoint
         * @return void
         */
        public function rollbackToSavepoint(string $savepoint): void {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces PHP objects
         * for the second column if it's a JSON object.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedColumnDictionaryObj(string $query, ?array $parameters = null): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces associative arrays
         * for the second column if it's a JSON object.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedColumnDictionaryAssoc(string $query, ?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces each row to be represented as a PHP associative array.
         *
         * This overrides the driver’s default associative/object mode for this call only.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception - If the first column is not convertible to string.
         * @throws \Exception - If any row fails to convert to an associative array.
         */
        public function queryGroupedDictionaryAssoc(string $query, ?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces each row to be represented as a PHP object.
         *
         * This overrides the driver’s default associative/object mode for this call only.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception - If the first column is not convertible to string.
         * @throws \Exception - If any row fails to convert to an object.
         */
        public function queryGroupedDictionaryObj(string $query, ?array $parameters = null): mixed {}

        /**
         * Sets client metadata for this connection.
         *
         * The metadata is formatted and appended to the application name,
         * making it visible in database monitoring tools. This is useful for tracking
         * request IDs, user IDs, or other debugging information.
         *
         * @param string $application_name
         * @param array $info
         * @return void
         */
        public function setClientInfo(string $application_name, array $info): void {}

        /**
         * Sets the application name for this connection.
         *
         * This helps identify the connection in database monitoring tools:
         * - `PostgreSQL`: Visible in `pg_stat_activity.application_name`
         * - `MySQL`: Stored in session variable `@sqlx_application_name`
         * - `MSSQL`: Stored in session context via `sp_set_session_context`
         *
         * @param string $name
         * @return void
         */
        public function setApplicationName(string $name): void {}
    }

    /**
     * Interface for database drivers.
     *
     * This interface defines the contract that all database drivers must implement,
     * providing methods for querying, executing statements, and transactions.
     *
     * Implementing classes: `PgDriver`, `MySqlDriver`, `MssqlDriver`
     */
    interface DriverInterface {
        /**
         * Closes the connection pool and releases all database connections.
         *
         * @return void
         */
        public function close(): void;

        /**
         * Executes an INSERT/UPDATE/DELETE statement and returns affected rows.
         *
         * @param string $query
         * @param array|null $parameters
         * @return int
         */
        public function execute(string $query, ?array $parameters = null): int;

        /**
         * Executes an SQL query and returns a column from all rows as associative arrays.
         *
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return array
         */
        public function queryColumnAssoc(string $query, ?array $parameters = null, mixed $column = null): array;

        /**
         * Executes an SQL query and returns a column from all rows as objects.
         *
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return array
         */
        public function queryColumnObj(string $query, ?array $parameters = null, mixed $column = null): array;

        /**
         * Executes an SQL query and returns a column from all rows.
         *
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return array
         */
        public function queryColumn(string $query, ?array $parameters = null, mixed $column = null): array;

        /**
         * Executes an SQL query and returns a dictionary indexed by the first column.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryDictionary(string $query, ?array $parameters = null): mixed;

        /**
         * Executes an SQL query and returns a dictionary with rows as associative arrays.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryDictionaryAssoc(string $query, ?array $parameters = null): mixed;

        /**
         * Executes an SQL query and returns a dictionary with rows as objects.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryDictionaryObj(string $query, ?array $parameters = null): mixed;

        /**
         * Executes an SQL query and returns a single column value as associative array or null.
         *
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryMaybeValueAssoc(string $query, ?array $parameters = null, mixed $column = null): mixed;

        /**
         * Executes an SQL query and returns a single column value as associative array.
         *
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryValueAssoc(string $query, ?array $parameters = null, mixed $column = null): mixed;

        /**
         * Executes an SQL query and returns a single column value as object or null.
         *
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryMaybeValueObj(string $query, ?array $parameters = null, mixed $column = null): mixed;

        /**
         * Executes an SQL query and returns a single column value as object.
         *
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryValueObj(string $query, ?array $parameters = null, mixed $column = null): mixed;

        /**
         * Executes an SQL query and returns a single column value or null.
         *
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryMaybeValue(string $query, ?array $parameters = null, mixed $column = null): mixed;

        /**
         * Executes an SQL query and returns a single column value.
         *
         * @param string $query
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryValue(string $query, ?array $parameters = null, mixed $column = null): mixed;

        /**
         * Executes an SQL query and returns a single row as associative array or null.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryMaybeRowAssoc(string $query, ?array $parameters = null): mixed;

        /**
         * Executes an SQL query and returns a single row as associative array.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRowAssoc(string $query, ?array $parameters = null): mixed;

        /**
         * Executes an SQL query and returns a single row as object or null.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryMaybeRowObj(string $query, ?array $parameters = null): mixed;

        /**
         * Executes an SQL query and returns a single row as object.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRowObj(string $query, ?array $parameters = null): mixed;

        /**
         * Executes an SQL query and returns a single row or null.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryMaybeRow(string $query, ?array $parameters = null): mixed;

        /**
         * Executes an SQL query and returns a single row.
         *
         * @param string $query
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRow(string $query, ?array $parameters = null): mixed;

        /**
         * Executes an SQL query and returns all rows as associative arrays.
         *
         * @param string $query
         * @param array|null $parameters
         * @return array
         */
        public function queryAllAssoc(string $query, ?array $parameters = null): array;

        /**
         * Executes an SQL query and returns all rows as objects.
         *
         * @param string $query
         * @param array|null $parameters
         * @return array
         */
        public function queryAllObj(string $query, ?array $parameters = null): array;

        /**
         * Executes an SQL query and returns all rows.
         *
         * @param string $query
         * @param array|null $parameters
         * @return array
         */
        public function queryAll(string $query, ?array $parameters = null): array;

        /**
         * Quotes a single scalar value for safe embedding into SQL.
         *
         * @param mixed $param
         * @return string
         */
        public function quote(mixed $param): string;

        /**
         * Quotes a string for use in a LIKE/ILIKE pattern.
         *
         * @param mixed $param
         * @return string
         */
        public function quoteLike(mixed $param): string;

        /**
         * Quotes an identifier (table name, column name).
         *
         * @param string $name
         * @return string
         */
        public function quoteIdentifier(string $name): string;

        /**
         * Returns true if the driver has been closed.
         *
         * @return bool
         */
        public function isClosed(): bool;

        /**
         * Returns whether results are returned as associative arrays.
         *
         * @return bool
         */
        public function assocArrays(): bool;
    }

    /**
     * Interface for prepared queries.
     *
     * Implementing classes: `PgPreparedQuery`, `MySqlPreparedQuery`, `MssqlPreparedQuery`
     */
    interface PreparedQueryInterface {
        /**
         * Executes the prepared query and returns a column from all rows.
         *
         * @param array|null $parameters
         * @param mixed $column
         * @return array
         */
        public function queryColumn(?array $parameters = null, mixed $column = null): array;

        /**
         * Executes the prepared query and returns a single column value.
         *
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryValue(?array $parameters = null, mixed $column = null): mixed;

        /**
         * Executes the prepared query and returns a single row as associative array.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRowAssoc(?array $parameters = null): mixed;

        /**
         * Executes the prepared query and returns a single row as object.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRowObj(?array $parameters = null): mixed;

        /**
         * Executes the prepared query and returns a single row or null.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryMaybeRow(?array $parameters = null): mixed;

        /**
         * Executes the prepared query and returns a single row.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRow(?array $parameters = null): mixed;

        /**
         * Executes the prepared query and returns all rows as associative arrays.
         *
         * @param array|null $parameters
         * @return array
         */
        public function queryAllAssoc(?array $parameters = null): array;

        /**
         * Executes the prepared query and returns all rows as objects.
         *
         * @param array|null $parameters
         * @return array
         */
        public function queryAllObj(?array $parameters = null): array;

        /**
         * Executes the prepared query and returns all rows.
         *
         * @param array|null $parameters
         * @return array
         */
        public function queryAll(?array $parameters = null): array;

        /**
         * Executes the prepared statement and returns affected rows.
         *
         * @param array|null $parameters
         * @return int
         */
        public function execute(?array $parameters = null): int;
    }

    /**
     * Interface for read-only query builders.
     *
     * Implementing classes: `PgReadQueryBuilder`, `MySqlReadQueryBuilder`, `MssqlReadQueryBuilder`
     */
    interface ReadQueryBuilderInterface {
    }

    /**
     * Interface for write query builders.
     *
     * Implementing classes: `PgWriteQueryBuilder`, `MySqlWriteQueryBuilder`, `MssqlWriteQueryBuilder`
     */
    interface WriteQueryBuilderInterface {
    }

    /**
     * Represents a dynamic ORDER BY / GROUP BY clause generator.
     *
     * This struct allows validating and mapping user input (e.g. from HTTP parameters)
     * to a known set of allowed sortable fields or SQL expressions.
     *
     * It supports two modes:
     * - `"name"` (auto-mapped to `"name"`)
     * - `"posts" => "COUNT(posts.*)"` (maps user field to custom SQL)
     *
     * Use with `ByClauseRendered` to safely inject into a query as a single placeholder.
     */
    class ByClause {
        /**
         * Ascending order (A to Z)
         */
        const ASC = 'ASC';

        /**
         * Descending order (Z to A)
         */
        const DESC = 'DESC';

        /**
         * Applies ordering rules to a user-defined input.
         *
         * @param array $columns List of columns (as strings or [field, direction] arrays)
         * @return \Sqlx\ByClauseRendered A `ByClauseRendered` object containing validated SQL ORDER BY clauses. The resulting value is to be used as a placeholder in query bindings.
         */
        public function input(array $columns): \Sqlx\ByClauseRendered {}

        /**
         * Constructs a `ByClause` helper with allowed sortable columns.
         *
         * @param array $allowed_columns Map of allowed sort columns (key = user input, value = SQL expression)
         */
        public function __construct(array $allowed_columns) {}

        /**
         * `__invoke` magic for `apply()`.
         *
         * @param array $columns
         * @return \Sqlx\ByClauseRendered
         */
        public function __invoke(array $columns): \Sqlx\ByClauseRendered {}
    }

    /**
     * Represents an OR clause for building complex boolean conditions.
     *
     * Created using the `OR_()` PHP function, this allows nested OR conditions
     * within WHERE clauses.
     *
     * # PHP Example
     *
     * ```php
     * use function Sqlx\OR_;
     *
     * $builder->where([
     *     ['status', '=', 'active'],
     *     OR_([
     *         ['role', '=', 'admin'],
     *         ['role', '=', 'moderator']
     *     ])
     * ]);
     * ```
     */
    class OrClause {
        public function __construct() {}
    }

    /**
     * Represents the available options for `SQLx` drivers (`PgDriver`, `MySqlDriver`, `MssqlDriver`).
     *
     * These constants are used as keys when constructing an options array passed to `DriverFactory::make(...)`.
     */
    class DriverOptions {
        /**
         * Backoff multiplier for exponential backoff (default: 2.0).
         */
        const OPT_RETRY_MULTIPLIER = 'retry_multiplier';

        /**
         * Enable automatic collapsing of `IN ()` clauses to `FALSE`/`TRUE`.
         */
        const OPT_COLLAPSIBLE_IN = 'collapsible_in';

        /**
         * Enable read-only mode (useful for replicas).
         */
        const OPT_READONLY = 'readonly';

        /**
         * Idle timeout for pooled connections. Accepts string or integer (seconds).
         */
        const OPT_IDLE_TIMEOUT = 'idle_timeout';

        /**
         * Initial backoff duration between retries. Accepts string (`"100ms"`, `"1s"`) or integer (seconds).
         */
        const OPT_RETRY_INITIAL_BACKOFF = 'retry_initial_backoff';

        /**
         * Max entries per AST cache shard (advanced).
         */
        const OPT_AST_CACHE_SHARD_SIZE = 'ast_cache_shard_size';

        /**
         * Maximum backoff duration between retries. Accepts string (`"5s"`, `"1 min"`) or integer (seconds).
         */
        const OPT_RETRY_MAX_BACKOFF = 'retry_max_backoff';

        /**
         * Maximum lifetime of a pooled connection. Accepts string (`"30s"`, `"5 min"`) or integer (seconds).
         */
        const OPT_MAX_LIFETIME = 'max_lifetime';

        /**
         * Maximum number of connections in the pool (default: 10).
         */
        const OPT_MAX_CONNECTIONS = 'max_connections';

        /**
         * Maximum retry attempts for transient failures (default: 0 = disabled).
         */
        const OPT_RETRY_MAX_ATTEMPTS = 'retry_max_attempts';

        /**
         * Minimum number of connections in the pool (default: 0).
         */
        const OPT_MIN_CONNECTIONS = 'min_connections';

        /**
         * Number of AST cache shards (advanced).
         */
        const OPT_AST_CACHE_SHARD_COUNT = 'ast_cache_shard_count';

        /**
         * Pool name to enable persistent connection reuse.
         */
        const OPT_PERSISTENT_NAME = 'persistent_name';

        /**
         * Read replica URLs for automatic read/write splitting.
         * Accepts an array of connection URLs: `['postgres://replica1/db', 'postgres://replica2/db']`
         */
        const OPT_READ_REPLICAS = 'read_replicas';

        /**
         * Required database URL, such as `postgres://user:pass@localhost/db`.
         */
        const OPT_URL = 'url';

        /**
         * Return rows as associative arrays instead of objects (default: false).
         */
        const OPT_ASSOC_ARRAYS = 'assoc_arrays';

        /**
         * Timeout when acquiring a connection from the pool. Accepts string or integer (seconds).
         */
        const OPT_ACQUIRE_TIMEOUT = 'acquire_timeout';

        /**
         * Whether to validate connections before acquiring them from the pool.
         */
        const OPT_TEST_BEFORE_ACQUIRE = 'test_before_acquire';

        public function __construct() {}
    }

    /**
     * The `PaginateClauseRendered` struct holds the result of pagination:
     * - `limit`: Number of items to return (`LIMIT`).
     * - `offset`: Number of items to skip (`OFFSET`).
     */
    class PaginateClauseRendered {
        public function __construct() {}
    }

    /**
     * The `SelectClauseRendered` struct holds validated
     * column clauses for SQL SELECT statements.
     */
    class SelectClauseRendered {
        public function __construct() {}
    }

    /**
     * The `Sqlx\PaginateClause` class represents pagination settings
     * and provides methods to compute the appropriate SQL `LIMIT` and `OFFSET`
     * based on a given page number and items-per-page values.
     */
    class PaginateClause {
        /**
         * Applies pagination settings and returns a `PaginateClauseRendered`.
         *
         * @param int|null $page_number
         * @param int|null $per_page
         * @return \Sqlx\PaginateClauseRendered
         */
        public function input(?int $page_number = null, ?int $per_page = null): \Sqlx\PaginateClauseRendered {}

        /**
         * Magic `__invoke` method allowing the object to be used as a callable
         * for applying pagination.
         *
         * @param int|null $page_number
         * @param int|null $per_page
         * @return \Sqlx\PaginateClauseRendered A `PaginateClauseRendered` with calculated `limit` and `offset`.
         */
        public function __invoke(?int $page_number = null, ?int $per_page = null): \Sqlx\PaginateClauseRendered {}

        /**
         * PHP constructor for `Sqlx\PaginateClause`.
         */
        public function __construct() {}

        /**
         * Sets a fixed number of items per page.
         *
         * Updates `min_per_page`, `max_per_page`, and `default_per_page`
         * to the provided value.
         *
         * @param int $per_page
         * @return void
         * @throws \Exception Returns an error if `per_page < 1`.
         */
        public function perPage(int $per_page): void {}

        /**
         * Sets the maximum number of items per page.
         *
         * Ensures `min_per_page` and `default_per_page` do not exceed
         * the new maximum value.
         *
         * @param int $max_per_page
         * @return void
         * @throws \Exception Returns an error if `max_per_page < 1`.
         */
        public function maxPerPage(int $max_per_page): void {}

        /**
         * Sets the minimum number of items per page.
         *
         * Ensures `max_per_page` and `default_per_page` are at least
         * the new minimum value.
         *
         * @param int $min_per_page
         * @return void
         * @throws \Exception Returns an error if `min_per_page < 1`.
         */
        public function minPerPage(int $min_per_page): void {}
    }

    /**
     * The `Sqlx\\SelectClause` class manages a set of allowed
     * columns for SQL SELECT operations and provides methods
     * to render validated column clauses from user input.
     */
    class SelectClause {
        /**
         * Cnstructor for `Sqlx\\SelectClause`.
         *
         * @param array $allowed_columns Associative array of allowed columns:
         * @return \Sqlx\SelectClause
         */
        public static function allowed(array $allowed_columns): \Sqlx\SelectClause {}

        /**
         * Magic `__invoke` method allowing the object to be
         * used as a callable for rendering select clauses.
         *
         * @param array $columns
         * @return \Sqlx\SelectClauseRendered
         */
        public function __invoke(array $columns): \Sqlx\SelectClauseRendered {}

        /**
         * PHP constructor for `Sqlx\\SelectClause`.
         *
         * @param array $allowed_columns Associative array of allowed columns:
         */
        public function __construct(array $allowed_columns) {}

        /**
         * Renders validated SELECT clause columns from user input.
         *
         * @param array $columns List of column identifiers provided by user.
         * @return \Sqlx\SelectClauseRendered A `SelectClauseRendered` containing only allowed columns. Unknown columns are silently ignored.
         */
        public function input(array $columns): \Sqlx\SelectClauseRendered {}
    }

    class DriverFactory {
        /**
         * Creates a driver instance based on the DSN or config array.
         *
         * @param mixed $url_or_options
         * @return mixed Instance of `Sqlx\PgDriver`, `Sqlx\MySqlDriver`, or `Sqlx\MssqlDriver`
         */
        public static function make(mixed $url_or_options): mixed {}

        public function __construct() {}
    }

    class JsonWrapper {
        public function __construct() {}
    }
}

namespace Sqlx\Driver {
    /**
     * A reusable prepared SQL query with parameter support. Created using `PgDriver::prepare()`, shares context with original driver.
     */
    class MssqlPreparedQuery implements Sqlx\PreparedQueryInterface {
        /**
         * @param array|null $parameters
         * @param mixed $column
         * @return array
         */
        public function queryColumn(?array $parameters = null, mixed $column = null): array {}

        /**
         * @param array|null $parameters
         * @param mixed $column
         * @return mixed
         */
        public function queryValue(?array $parameters = null, mixed $column = null): mixed {}

        /**
         * @param array|null $parameters
         * @return array
         */
        public function queryAll(?array $parameters = null): array {}

        /**
         * @param array|null $parameters
         * @return array
         */
        public function queryAllAssoc(?array $parameters = null): array {}

        /**
         * @param array|null $parameters
         * @return array
         */
        public function queryAllObj(?array $parameters = null): array {}

        /**
         * @param array|null $parameters
         * @return int
         */
        public function execute(?array $parameters = null): int {}

        /**
         * @param array|null $parameters
         * @return mixed
         */
        public function queryMaybeRow(?array $parameters = null): mixed {}

        /**
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRow(?array $parameters = null): mixed {}

        /**
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRowAssoc(?array $parameters = null): mixed {}

        /**
         * @param array|null $parameters
         * @return mixed
         */
        public function queryRowObj(?array $parameters = null): mixed {}

        /**
         * Executes a query and returns a grouped dictionary (Vec of rows per key).
         *
         * Same as [`queryGroupedDictionary`](crate::Driver::query_grouped_dictionary), but works on a prepared query.
         *
         * The first column is used as the key (must be scalar),
         * and each resulting row is appended to the corresponding key's Vec.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Fails if the query fails, or the first column is not scalar.
         */
        public function queryGroupedDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an associative array.
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row as an associative array.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an object (`stdClass`).
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row as an object.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row.
         *
         * The result is a `HashMap` where the key is the stringified first column from each row,
         * and the value is the full row, returned as array or object depending on config.
         *
         * @param array|null $parameters
         * @return mixed A map from the first column (as string) to the corresponding row.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a string;
         * @throws \Exception - any row cannot be decoded or converted to a PHP value.
         */
        public function queryDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in associative array mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as associative arrays.
         *
         * @param array|null $parameters
         * @return mixed A dictionary where each key is the first column (as string), and each value is the second column as an associative PHP array.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in object mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as objects.
         *
         * @param array|null $parameters
         * @return mixed A dictionary where each key is the first column (as string), and each value is the second column as a PHP object.
         * @throws \Exception Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a dictionary mapping the first column to the second column.
         *
         * This method expects each result row to contain at least two columns. It converts the first column
         * into a PHP string (used as the key), and the second column into a PHP value (used as the value).
         *
         * @param array|null $parameters
         * @return mixed An associative array (`array<string, mixed>`) where each key is the first column (as string), and the value is the second column.
         * @throws \Exception Returns an error if:
         * @throws \Exception - the query fails to execute;
         * @throws \Exception - the first column cannot be converted to a PHP string;
         * @throws \Exception - the second column cannot be converted to a PHP value.
         */
        public function queryColumnDictionary(?array $parameters = null): mixed {}

        /**
         * Executes the prepared query and returns a grouped dictionary where:
         * - the key is the **first column** (must be scalar),
         * - the value is a list of values from the **second column** for each group.
         *
         * This variant uses the driver's default associative array option for JSON values.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionary(?array $parameters = null): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces PHP objects
         * for the second column if it contains JSON objects.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces associative arrays
         * for the second column if it contains JSON objects.
         *
         * @param array|null $parameters
         * @return mixed
         * @throws \Exception Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryAssoc(?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as PHP objects.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedDictionaryObj(?array $parameters = null): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as associative arrays.
         *
         * @param array|null $parameters
         * @return mixed
         */
        public function queryGroupedDictionaryAssoc(?array $parameters = null): mixed {}

        public function __construct() {}
    }
}

namespace Sqlx\Exceptions {
    /**
     * Base exception class for php-sqlx errors.
     *
     * This exception class extends PHP's base Exception. Specific error types
     * throw concrete subclasses (e.g., `ConnectionException`, `QueryException`).
     */
    class SqlxException extends \Exception {
        /**
         * Configuration/options error
         */
        const CONFIGURATION = 6;

        /**
         * Database connection failed
         */
        const CONNECTION = 1;

        /**
         * General/unknown error
         */
        const GENERAL = 0;

        /**
         * Invalid identifier or input validation error
         */
        const VALIDATION = 7;

        /**
         * Missing or invalid parameter
         */
        const PARAMETER = 5;

        /**
         * Operation not permitted (e.g., write on readonly)
         */
        const NOT_PERMITTED = 8;

        /**
         * Pool exhausted
         */
        const POOL_EXHAUSTED = 10;

        /**
         * Query execution failed
         */
        const QUERY = 2;

        /**
         * SQL parsing/AST error
         */
        const PARSE = 4;

        /**
         * Timeout error
         */
        const TIMEOUT = 9;

        /**
         * Transaction-related error
         */
        const TRANSACTION = 3;

        public function __construct() {}
    }

    /**
     * Thrown when SQL parsing fails.
     */
    class ParseException extends \Sqlx\Exceptions\SqlxException {
        public function __construct() {}
    }

    /**
     * Thrown when configuration is invalid.
     */
    class ConfigurationException extends \Sqlx\Exceptions\SqlxException {
        public function __construct() {}
    }

    /**
     * Thrown when connection pool is exhausted.
     */
    class PoolExhaustedException extends \Sqlx\Exceptions\SqlxException {
        public function __construct() {}
    }

    /**
     * Thrown when database connection fails.
     */
    class ConnectionException extends \Sqlx\Exceptions\SqlxException {
        public function __construct() {}
    }

    /**
     * Thrown when input validation fails.
     */
    class ValidationException extends \Sqlx\Exceptions\SqlxException {
        public function __construct() {}
    }

    /**
     * Thrown when operation is not permitted.
     */
    class NotPermittedException extends \Sqlx\Exceptions\SqlxException {
        public function __construct() {}
    }

    /**
     * Thrown when operation times out.
     */
    class TimeoutException extends \Sqlx\Exceptions\SqlxException {
        public function __construct() {}
    }

    /**
     * Thrown when parameter binding fails.
     */
    class ParameterException extends \Sqlx\Exceptions\SqlxException {
        public function __construct() {}
    }

    /**
     * Thrown when query execution fails.
     */
    class QueryException extends \Sqlx\Exceptions\SqlxException {
        public function __construct() {}
    }

    /**
     * Thrown when transaction operations fail.
     */
    class TransactionException extends \Sqlx\Exceptions\SqlxException {
        public function __construct() {}
    }
}
