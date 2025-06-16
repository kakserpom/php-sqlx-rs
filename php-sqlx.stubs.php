<?php

// Stubs for php-sqlx

namespace Sqlx {
    /**
     * A reusable prepared SQL query with parameter support.
     *
     * Created using `Driver::prepare()`, shares context with original driver.
     */
    class PreparedQuery {
        /**
         * Executes the prepared query with optional parameters.
         *
         * # Parameters
         * - `parameters`: Optional map of named parameters
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
         * # Parameters
         * - `parameters`: Optional map of named parameters
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
         * # Parameters
         * - `parameters`: Optional map of named parameters
         */
        public function queryRowAssoc(?array $parameters): mixed {}

        /**
         * Executes the prepared query and returns one row as an object.
         *
         * # Parameters
         * - `parameters`: Optional map of named parameters
         */
        public function queryRowObj(?array $parameters): mixed {}

        /**
         * Executes the prepared query and returns all rows.
         *
         * # Parameters
         * - `parameters`: Optional map of named parameters
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
         * # Parameters
         * - `parameters`: Optional map of named parameters
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
         * # Parameters
         * - `parameters`: Optional map of named parameters
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
         * # Parameters
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
         * # Parameters
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
         * # Parameters
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
         * # Parameters
         * - `query`: SQL query string
         * - `parameters`: Optional map of named parameters
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
         * Executes a SQL query and returns one row as an associative array.
         *
         * # Parameters
         * - `query`: SQL query string
         * - `parameters`: Optional map of named parameters
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
         * # Parameters
         * - `query`: SQL query string
         * - `parameters`: Optional map of named parameters
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
         * Executes a SQL query and returns a single result.
         *
         * # Parameters
         * - `query`: SQL query string
         * - `parameters`: Optional map of named parameters
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
         * Executes a SQL query and returns one row as an associative array.
         *
         * # Parameters
         * - `query`: SQL query string
         * - `parameters`: Optional map of named parameters
         */
        public function queryRowMaybeAssoc(string $query, ?array $parameters): mixed {}

        /**
         * Executes a SQL query and returns one row as an object.
         *
         * # Parameters
         * - `query`: SQL query string
         * - `parameters`: Optional map of named parameters
         */
        public function queryMaybeRowObj(string $query, ?array $parameters): mixed {}

        /**
         * Executes a SQL query and returns all results.
         *
         * # Parameters
         * - `query`: SQL query string
         * - `parameters`: Optional map of named parameters
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
         * # Parameters
         * - `query`: SQL query string
         * - `parameters`: Optional map of named parameters
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
         * # Parameters
         * - `query`: SQL query string
         * - `parameters`: Optional map of named parameters
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
         * # Parameters
         * - `query`: SQL query string to prepare
         *
         * # Returns
         * Prepared query object
         */
        public function prepare(string $query): \Sqlx\PreparedQuery {}

        /**
         * Executes an INSERT/UPDATE/DELETE query and returns affected row count.
         *
         * # Parameters
         * - `query`: SQL query string
         * - `parameters`: Optional map of named parameters
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
         * # Parameters
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
         * # Parameters
         * - `query`: SQL query string
         * - `parameters`: Optional map of named parameters
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
}
