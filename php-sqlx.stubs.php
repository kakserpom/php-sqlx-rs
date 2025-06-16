<?php

// Stubs for php-sqlx

namespace Sqlx {
    /**
     * A database driver using SQLx with query helpers and AST cache.
     *
     * This class supports prepared queries, persistent connections, and augmented SQL.
     */
    class Driver {
        const OPT_URL = null;

        const OPT_AST_CACHE_SHARD_COUNT = null;

        const OPT_AST_CACHE_SHARD_SIZE = null;

        const OPT_PERSISTENT_NAME = null;

        const OPT_ASSOC_ARRAYS = null;

        public $assoc_arrays;

        /**
         * Constructs a new SQLx driver instance.
         *
         * # Arguments
         * - `options`: Connection URL as string or associative array with options:
         *   - `url`: (string) database connection string (required)
         *   - `ast_cache_shard_count`: (int) number of AST cache shards (default: 8)
         *   - `ast_cache_shard_size`: (int) size per shard (default: 256)
         *   - `persistent_name`: (string) name of persistent connection
         *   - `assoc_arrays`: (bool) return associative arrays instead of objects
         */
        public function __construct(mixed $options) {}

        /**
         * Returns whether results are returned as associative arrays.
         *
         * If true, result rows are returned as PHP associative arrays (key-value pairs).
         * If false, result rows are returned as PHP `stdClass` objects.
         */
        public function assocArrays(): bool {}

        /**
         * Executes a SQL query and returns a single result.
         *
         * # Arguments
         * - `query`: SQL query string
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * Single row as array or object depending on config
         *
         * # Errors
         * Returns an error if:
         * - SQL query is invalid or execution fails;
         * - a parameter cannot be bound or has incorrect type;
         * - the row contains unsupported database types;
         * - conversion to PHP object fails.
         */
        public function queryRow(string $query, ?array $parameters): mixed {}

        /**
         * Executes a SQL query and returns a single column value from the first row.
         *
         * # Arguments
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional array of indexed/named parameters to bind.
         * - `column`: Optional column name or index to extract from the result row. Defaults to the first column.
         *
         * # Returns
         * The value from the specified column of the first row.
         *
         * # Errors
         * Returns an error if:
         * - the query is invalid or fails to execute;
         * - the specified column does not exist;
         * - the value cannot be converted to a PHP-compatible type.
         */
        public function queryRowColumn(string $query, ?array $parameters, ?mixed $column): mixed {}

        public function queryRowColumnAssoc(string $query, ?array $parameters, ?mixed $column): mixed {}

        /**
         *
             * Executes a SQL query and returns a single column value as a PHP object from the first row.
             *
             * Same as `queryRowColumn`, but forces object mode for decoding structured types (e.g., JSON, composite).
             *
             * # Parameters
             * - `query`: SQL query string to execute.
             * - `parameters`: Optional array of indexed or named parameters to bind.
             * - `column`: Optional column name or zero-based index to extract. Defaults to the first column.
             *
             * # Returns
             * The value from the specified column of the first row, decoded as a PHP object.
             *
             * # Errors
             * Returns an error if:
             * - the query is invalid or fails to execute;
             * - the column does not exist;
             * - the value cannot be converted to a PHP object (e.g., due to encoding or type mismatch).

         */
        public function queryRowColumnObj(string $query, ?array $parameters, ?mixed $column): mixed {}

        /**
         * Executes a SQL query and returns a single column value from the first row, or null if no rows matched.
         *
         * # Arguments
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional array of indexed/named parameters to bind.
         * - `column`: Optional column name or index to extract from the result row.
         *
         * # Returns
         * The value from the specified column of the first row as a PHP value`, or `null` if no row was found.
         *
         * # Errors
         * Returns an error if:
         * - the query is invalid or fails to execute;
         * - the specified column does not exist;
         * - the value cannot be converted to a PHP-compatible type.
         */
        public function queryMaybeRowColumn(string $query, ?array $parameters, ?mixed $column): mixed {}

        /**
         * Executes a SQL query and returns a single column value as a PHP value (array mode), or null if no row matched.
         *
         * Same as `query_maybe_row_column`, but forces associative array mode for complex values.
         *
         * # Arguments
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional array of indexed/named parameters to bind.
         * - `column`: Optional column name or index to extract.
         *
         * # Returns
         * The column value from the first row, or `null` if no row found.
         *
         * # Errors
         * Same as `query_maybe_row_column`.
         */
        public function queryMaybeRowColumnAssoc(string $query, ?array $parameters, ?mixed $column): mixed {}

        /**
         * Executes a SQL query and returns a single column value as a PHP object, or null if no row matched.
         *
         * Same as `query_maybe_row_column`, but forces object mode for complex values.
         *
         * # Arguments
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional array of indexed/named parameters to bind.
         * - `column`: Optional column name or index to extract.
         *
         * # Returns
         * The column value from the first row, or `null` if no row found.
         *
         * # Errors
         * Same as `query_maybe_row_column`.
         */
        public function queryMaybeRowColumnObj(string $query, ?array $parameters, ?mixed $column): mixed {}

        /**
         * Executes a SQL query and returns one row as an associative array.
         *
         * # Arguments
         * - `query`: SQL query string
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Errors
         * Returns an error if:
         * - SQL query is invalid or execution fails;
         * - a parameter cannot be bound or has incorrect type;
         * - the row contains unsupported database types;
         * - conversion to PHP object fails.
         */
        public function queryRowAssoc(string $query, ?array $parameters): mixed {}

        /**
         * Executes a SQL query and returns one row as an object.
         *
         * # Arguments
         * - `query`: SQL query string
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Errors
         * Returns an error if:
         * - SQL query is invalid or execution fails;
         * - a parameter cannot be bound or has incorrect type;
         * - the row contains unsupported database types;
         * - conversion to PHP object fails.
         */
        public function queryRowObj(string $query, ?array $parameters): mixed {}

        /**
         * Executes a SQL query and returns a single result, or `null` if no row matched.
         *
         * # Arguments
         * - `query`: SQL query string
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * Single row as array or object depending on config
         *
         * # Errors
         * Returns an error if the query fails for reasons other than no matching rows.
         * For example, syntax errors, type mismatches, or database connection issues.
         */
        public function queryMaybeRow(string $query, ?array $parameters): mixed {}

        /**
         * Executes a SQL query and returns a single row as a PHP associative array, or `null` if no row matched.
         *
         * # Arguments
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * The result row as an associative array (`array<string, mixed>` in PHP), or `null` if no matching row is found.
         *
         * # Errors
         * Returns an error if:
         * - the query is invalid or fails to execute;
         * - parameters are invalid or cannot be bound;
         * - the row contains unsupported or unconvertible data types.
         */
        public function queryMaybeRowAssoc(string $query, ?array $parameters): mixed {}

        /**
         * Executes a SQL query and returns a single row as a PHP object, or `null` if no row matched.
         *
         * # Arguments
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * The result row as a `stdClass` PHP object, or `null` if no matching row is found.
         *
         * # Errors
         * Returns an error if:
         * - the query is invalid or fails to execute;
         * - parameters are invalid or cannot be bound;
         * - the row contains unsupported or unconvertible data types.
         */
        public function queryMaybeRowObj(string $query, ?array $parameters): mixed {}

        /**
         * Executes a SQL query and returns the specified column values from all result rows.
         *
         * # Arguments
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional array of indexed/named parameters to bind.
         * - `column`: Optional column name or index to extract.
         *
         * # Returns
         * An array of column values, one for each row.
         *
         * # Errors
         * Returns an error if:
         * - the query fails to execute;
         * - the specified column is not found;
         * - a column value cannot be converted to PHP.
         */
        public function queryColumn(string $query, ?array $parameters, ?mixed $column): array {}

        /**
         * Executes a SQL query and returns the specified column values from all rows in associative array mode.
         *
         * # Arguments
         * - `query`: SQL query string.
         * - `parameters`: Optional named parameters.
         * - `column`: Column index or name to extract.
         *
         * # Returns
         * An array of column values (associative arrays for structured data).
         *
         * # Errors
         * Same as `query_column`.
         */
        public function queryColumnAssoc(string $query, ?array $parameters, ?mixed $column): array {}

        /**
         * Executes a SQL query and returns the specified column values from all rows in object mode.
         *
         * # Arguments
         * - `query`: SQL query string.
         * - `parameters`: Optional named parameters.
         * - `column`: Column index or name to extract.
         *
         * # Returns
         * An array of column values (objects for structured data).
         *
         * # Errors
         * Same as `query_column`.
         */
        public function queryColumnObj(string $query, ?array $parameters, ?mixed $column): array {}

        /**
         * Executes a SQL query and returns all results.
         *
         * # Arguments
         * - `query`: SQL query string
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * Array of rows as array or object depending on config
         *
         * # Errors
         * Returns an error if:
         * - SQL query is invalid or fails to execute;
         * - parameter binding fails;
         * - row decoding fails due to an unsupported or mismatched database type;
         * - conversion to PHP Zval fails (e.g., due to memory or encoding issues).
         */
        public function queryAll(string $query, ?array $parameters): array {}

        /**
         * Executes a SQL query and returns all rows as associative arrays.
         *
         * # Arguments
         * - `query`: SQL query string
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Errors
         * Returns an error if:
         * - SQL query is invalid or fails to execute;
         * - parameter binding fails;
         * - row decoding fails due to an unsupported or mismatched database type;
         * - conversion to PHP Zval fails (e.g., due to memory or encoding issues).
         */
        public function queryAllAssoc(string $query, ?array $parameters): array {}

        /**
         * Executes a SQL query and returns all rows as objects.
         *
         * # Arguments
         * - `query`: SQL query string
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Errors
         * Returns an error if:
         * - SQL query is invalid or fails to execute;
         * - parameter binding fails;
         * - row decoding fails due to an unsupported or mismatched database type;
         * - conversion to PHP Zval fails (e.g., due to memory or encoding issues).
         */
        public function queryAllObj(string $query, ?array $parameters): array {}

        /**
         * Creates a prepared query object with the given SQL string.
         *
         * # Arguments
         * - `query`: SQL query string to prepare
         *
         * # Returns
         * Prepared query object
         */
        public function prepare(string $query): \Sqlx\PreparedQuery {}

        /**
         * Executes an INSERT/UPDATE/DELETE query and returns affected row count.
         *
         * # Arguments
         * - `query`: SQL query string
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * Number of affected rows
         *
         * # Errors
         * Returns an error if:
         * - the SQL query is invalid or fails to execute (e.g., due to syntax error, constraint violation, or connection issue);
         * - parameters contain unsupported types or fail to bind correctly;
         * - the runtime fails to execute the query (e.g., task panic or timeout).
         */
        public function execute(string $query, ?array $parameters): int {}

        /**
         * Inserts a row into the given table using a map of fields.
         *
         * # Arguments
         * - `table`: Table name
         * - `row`: Map of column names to values
         *
         * # Returns
         * Number of inserted rows
         *
         * # Errors
         * Returns an error if:
         * - the SQL query is invalid or fails to execute (e.g., due to syntax error, constraint violation, or connection issue);
         * - parameters contain unsupported types or fail to bind correctly;
         * - the runtime fails to execute the query (e.g., task panic or timeout).
         */
        public function insert(string $table, array $row): int {}

        /**
         * Executes a SQL query and returns the rendered query and its parameters.
         *
         * This method does not execute the query but returns the SQL string with placeholders
         * and the bound parameter values for debugging or logging purposes.
         *
         * # Arguments
         * - `query`: SQL query string
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * A list where the first element is the rendered SQL query (string), and the second is an array of bound values
         *
         * # Errors
         * Returns an error if the query can't be parsed, rendered, or if parameters
         * cannot be converted from PHP values.
         */
        public function dry(string $query, ?array $parameters): array {}
    }

    class OrderBy {
        /**
         * ASCending order (A to Z)
         */
        const ASC = null;

        /**
         * DESCending order (Z to A)
         */
        const DESC = null;

        /**
         * Constructs an OrderBy helper with allowed sortable fields.
         *
         * # Arguments
         * - `defined_fields`: Map of allowed sort fields (key = user input, value = SQL expression)
         *
         * # Example
         * ```php
         * $order_by = new Sqlx\OrderBy([
         *     "name",
         *     "age",
         *     "total_posts" => "COUNT(posts.*)"
         * ]);
         * ```
         */
        public function __construct(array $defined_fields) {}

        /**
         * __invoke magic for apply()
         */
        public function __invoke(array $order_by): object {}

        /**
         * Applies ordering rules to a user-defined input.
         *
         * # Arguments
         * - `order_by`: List of fields (as strings or [field, direction] arrays)
         *
         * # Returns
         * A `RenderedOrderBy` object containing validated SQL ORDER BY clauses
         * The returning value is to be used as a placeholder value
         *
         * # Errors
         * This method does not return an error but silently ignores unknown fields.
         * Use validation separately if strict input is required.
         */
        public function apply(array $order_by): object {}
    }

    /**
     * A reusable prepared SQL query with parameter support.
     *
     * Created using `Driver::prepare()`, shares context with original driver.
     */
    class PreparedQuery {
        /**
         * Executes the prepared query with optional parameters.
         *
         * # Arguments
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * Number of affected rows
         *
         * # Errors
         * Returns an error if:
         * - the SQL query is invalid or fails to execute (e.g., due to syntax error, constraint violation, or connection issue);
         * - parameters contain unsupported types or fail to bind correctly;
         * - the runtime fails to execute the query (e.g., task panic or timeout).
         */
        public function execute(?array $parameters): int {}

        /**
         * Executes the prepared query and returns a single result.
         *
         * # Arguments
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * Single row as array or object depending on config
         *
         * # Errors
         * Returns an error if:
         * - SQL query is invalid or execution fails;
         * - a parameter cannot be bound or has incorrect type;
         * - the row contains unsupported database types;
         * - conversion to PHP object fails.
         */
        public function queryRow(?array $parameters): mixed {}

        /**
         * Executes the prepared query and returns one row as an associative array.
         *
         * # Arguments
         * - `parameters`: Optional array of indexed/named parameters to bind.
         */
        public function queryRowAssoc(?array $parameters): mixed {}

        /**
         * Executes the prepared query and returns one row as an object.
         *
         * # Arguments
         * - `parameters`: Optional array of indexed/named parameters to bind.
         */
        public function queryRowObj(?array $parameters): mixed {}

        /**
         * Executes a SQL query and returns a single result, or `null` if no row matched.
         *
         * # Arguments
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * Single row as array or object depending on config
         *
         * # Errors
         * Returns an error if the query fails for reasons other than no matching rows.
         * For example, syntax errors, type mismatches, or database connection issues.
         */
        public function queryMaybeRow(?array $parameters): mixed {}

        /**
         * Executes the SQL query and returns a single row as a PHP associative array, or `null` if no row matched.
         *
         * # Arguments
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * The result row as an associative array (`array<string, mixed>` in PHP), or `null` if no matching row is found.
         *
         * # Errors
         * Returns an error if:
         * - the query is invalid or fails to execute;
         * - parameters are invalid or cannot be bound;
         * - the row contains unsupported or unconvertible data types.
         */
        public function queryMaybeRowAssoc(?array $parameters): mixed {}

        /**
         * Executes a SQL query and returns a single row as a PHP object, or `null` if no row matched.
         *
         * # Arguments
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * The result row as a `stdClass` PHP object, or `null` if no matching row is found.
         *
         * # Errors
         * Returns an error if:
         * - the query is invalid or fails to execute;
         * - parameters are invalid or cannot be bound;
         * - the row contains unsupported or unconvertible data types.
         */
        public function queryMaybeRowObj(?array $parameters): mixed {}

        /**
         * Executes the SQL query and returns the specified column values from all result rows.
         *
         * # Arguments
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional array of indexed/named parameters to bind.
         * - `column`: Optional column name or index to extract.
         *
         * # Returns
         * An array of column values, one for each row.
         *
         * # Errors
         * Returns an error if:
         * - the query fails to execute;
         * - the specified column is not found;
         * - a column value cannot be converted to PHP.
         */
        public function queryColumn(?array $parameters, ?mixed $column): array {}

        /**
         * Executes the SQL query and returns the specified column values from all rows in associative array mode.
         *
         * # Arguments
         * - `query`: SQL query string.
         * - `parameters`: Optional named parameters.
         * - `column`: Column index or name to extract.
         *
         * # Returns
         * An array of column values (associative arrays for structured data).
         *
         * # Errors
         * Same as `query_column`.
         */
        public function queryColumnAssoc(?array $parameters, ?mixed $column): array {}

        /**
         * Executes the SQL query and returns the specified column values from all rows in object mode.
         *
         * # Arguments
         * - `parameters`: Optional named parameters.
         * - `column`: Column index or name to extract.
         *
         * # Returns
         * An array of column values (objects for structured data).
         *
         * # Errors
         * Same as `query_column`.
         */
        public function queryColumnObj(?array $parameters, ?mixed $column): array {}

        /**
         * Executes the prepared query and returns all rows.
         *
         * # Arguments
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * Array of rows as array or object depending on config
         *
         * # Errors
         * Returns an error if:
         * - SQL query is invalid or fails to execute;
         * - parameter binding fails;
         * - row decoding fails due to an unsupported or mismatched database type;
         * - conversion to PHP Zval fails (e.g., due to memory or encoding issues).
         */
        public function queryAll(?array $parameters): array {}

        /**
         * Executes the prepared query and returns all rows as associative arrays.
         *
         * # Arguments
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Errors
         * Returns an error if:
         * - SQL query is invalid or fails to execute;
         * - parameter binding fails;
         * - row decoding fails due to an unsupported or mismatched database type;
         * - conversion to PHP Zval fails (e.g., due to memory or encoding issues).
         */
        public function queryAllAssoc(?array $parameters): array {}

        /**
         * Executes the prepared query and returns all rows as objects.
         *
         * # Arguments
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Errors
         * Returns an error if:
         * - SQL query is invalid or fails to execute;
         * - parameter binding fails;
         * - row decoding fails due to an unsupported or mismatched database type;
         * - conversion to PHP Zval fails (e.g., due to memory or encoding issues).
         */
        public function queryAllObj(?array $parameters): array {}
    }
}
