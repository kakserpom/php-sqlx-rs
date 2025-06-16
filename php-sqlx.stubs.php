<?php

// Stubs for php-sqlx

namespace Sqlx {
    class DriverOptions {
        const OPT_URL = null;

        const OPT_AST_CACHE_SHARD_COUNT = null;

        const OPT_AST_CACHE_SHARD_SIZE = null;

        const OPT_PERSISTENT_NAME = null;

        const OPT_ASSOC_ARRAYS = null;
    }

    class OrderBy {
        /**
         * Ascending order (A to Z)
         */
        const ASC = null;

        /**
         * Descending order (Z to A)
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
         * # Exceptions
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
         * Executes the prepared query and returns a dictionary mapping the first column to the second column.
         *
         * This method expects each result row to contain at least two columns. It converts the first column
         * into a PHP string (used as the key), and the second column into a PHP value (used as the value).
         *
         * # Parameters
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * An associative array (`array<string, mixed>`) where each key is the first column (as string),
         * and the value is the second column.
         *
         * # Errors
         * Returns an error if:
         * - the query fails to execute;
         * - the first column cannot be converted to a PHP string;
         * - the second column cannot be converted to a PHP value.
         *
         * # Notes
         * - The order of dictionary entries is preserved.
         * - The query must return at least two columns per row.
         */
        public function queryColumnDictionary(?array $parameters): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in associative array mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as associative arrays.
         *
         * # Parameters
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * A dictionary where each key is the first column (as string),
         * and each value is the second column as an associative PHP array.
         *
         * # Errors
         * Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryAssoc(?array $parameters): mixed {}

        /**
         * Executes the prepared query and returns a dictionary in object mode.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as objects.
         *
         * # Parameters
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * A dictionary where each key is the first column (as string),
         * and each value is the second column as a PHP object.
         *
         * # Errors
         * Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryObj(?array $parameters): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row.
         *
         * The result is a `HashMap` where the key is the stringified first column from each row,
         * and the value is the full row, returned as array or object depending on config.
         *
         * # Parameters
         * - `parameters`: Optional map of named parameters to bind.
         *
         * # Returns
         * A map from the first column (as string) to the corresponding row.
         *
         * # Errors
         * Returns an error if:
         * - the query fails to execute;
         * - the first column cannot be converted to a string;
         * - any row cannot be decoded or converted to a PHP value.
         *
         * # Notes
         * - The iteration order of the returned map is **not** guaranteed.
         */
        public function queryDictionary(?array $parameters): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an associative array.
         *
         * # Parameters
         * - `parameters`: Optional map of named parameters to bind.
         *
         * # Returns
         * A map from the first column (as string) to the corresponding row as an associative array.
         *
         * # Errors
         * Returns an error if:
         * - the query fails to execute;
         * - the first column cannot be converted to a string;
         * - any row cannot be decoded or converted to a PHP value.
         *
         * # Notes
         * - The iteration order of the returned map is **not** guaranteed.
         */
        public function queryDictionaryAssoc(?array $parameters): mixed {}

        /**
         * Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
         * with each row returned as an object (`stdClass`).
         *
         * # Parameters
         * - `parameters`: Optional map of named parameters to bind.
         *
         * # Returns
         * A map from the first column (as string) to the corresponding row as an object.
         *
         * # Errors
         * Returns an error if:
         * - the query fails to execute;
         * - the first column cannot be converted to a string;
         * - any row cannot be decoded or converted to a PHP value.
         *
         * # Notes
         * - The iteration order of the returned map is **not** guaranteed.
         */
        public function queryDictionaryObj(?array $parameters): mixed {}

        /**
         * Executes a query and returns a grouped dictionary (Vec of rows per key).
         *
         * Same as [`queryGroupedDictionary`](crate::Driver::query_grouped_dictionary), but works on a prepared query.
         *
         * The first column is used as the key (must be convertible to string),
         * and each resulting row is appended to the corresponding key's Vec.
         *
         * # Errors
         * Fails if the query fails, or the first column is not string-convertible.
         */
        public function queryGroupedDictionary(?array $parameters): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as associative arrays.
         */
        public function queryGroupedDictionaryAssoc(?array $parameters): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces rows to be decoded as PHP objects.
         */
        public function queryGroupedDictionaryObj(?array $parameters): mixed {}

        /**
         * Executes the prepared query and returns a grouped dictionary where:
         * - the key is the **first column** (must be convertible to string),
         * - the value is a list of values from the **second column** for each group.
         *
         * This variant uses the driver's default associative array option for JSON values.
         *
         * # Errors
         * Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionary(?array $parameters): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces associative arrays
         * for the second column if it contains JSON objects.
         *
         * # Errors
         * Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryAssoc(?array $parameters): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces PHP objects
         * for the second column if it contains JSON objects.
         *
         * # Errors
         * Returns an error if the first column is not convertible to a string.
         */
        public function queryGroupedColumnDictionaryObj(?array $parameters): mixed {}

        /**
         * Executes the prepared query with optional parameters.
         *
         * # Arguments
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * Number of affected rows
         *
         * # Exceptions
         * Throws an exception if:
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
         * # Exceptions
         * Throws an exception if:
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
         * Executes an SQL query and returns a single result, or `null` if no row matched.
         *
         * # Arguments
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * Single row as array or object depending on config
         *
         * # Exceptions
         * Throws an exception if the query fails for reasons other than no matching rows.
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
         * # Exceptions
         * Throws an exception if:
         * - the query is invalid or fails to execute;
         * - parameters are invalid or cannot be bound;
         * - the row contains unsupported or unconvertible data types.
         */
        public function queryMaybeRowAssoc(?array $parameters): mixed {}

        /**
         * Executes an SQL query and returns a single row as a PHP object, or `null` if no row matched.
         *
         * # Arguments
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * The result row as a `stdClass` PHP object, or `null` if no matching row is found.
         *
         * # Exceptions
         * Throws an exception if:
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
         * # Exceptions
         * Throws an exception if:
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
         * # Exceptions
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
         * # Exceptions
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
         * # Exceptions
         * Throws an exception if:
         * - SQL query is invalid or fails to execute;
         * - parameter binding fails;
         * - row decoding fails due to an unsupported or mismatched database type;
         * - conversion to PHP values fails (e.g., due to memory or encoding issues).
         */
        public function queryAll(?array $parameters): array {}

        /**
         * Executes the prepared query and returns all rows as associative arrays.
         *
         * # Arguments
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Exceptions
         * Throws an exception if:
         * - SQL query is invalid or fails to execute;
         * - parameter binding fails;
         * - row decoding fails due to an unsupported or mismatched database type;
         * - conversion to PHP values fails (e.g., due to memory or encoding issues).
         */
        public function queryAllAssoc(?array $parameters): array {}

        /**
         * Executes the prepared query and returns all rows as objects.
         *
         * # Arguments
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Exceptions
         * Throws an exception if:
         * - SQL query is invalid or fails to execute;
         * - parameter binding fails;
         * - row decoding fails due to an unsupported or mismatched database type;
         * - conversion to PHP values fails (e.g., due to memory or encoding issues).
         */
        public function queryAllObj(?array $parameters): array {}
    }

    /**
     * A database driver using SQLx with query helpers and AST cache.
     *
     * This class supports prepared queries, persistent connections, and augmented SQL.
     */
    class Driver {
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
         * Executes an SQL query and returns a single result.
         *
         * # Arguments
         * - `query`: SQL query string
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * Single row as array or object depending on config
         *
         * # Exceptions
         * Throws an exception if:
         * - SQL query is invalid or execution fails;
         * - a parameter cannot be bound or has incorrect type;
         * - the row contains unsupported database types;
         * - conversion to PHP object fails.
         */
        public function queryRow(string $query, ?array $parameters): mixed {}

        /**
         * Executes an SQL query and returns a single column value from the first row.
         *
         * # Arguments
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional array of indexed/named parameters to bind.
         * - `column`: Optional column name or index to extract from the result row. Defaults to the first column.
         *
         * # Returns
         * The value from the specified column of the first row.
         *
         * # Exceptions
         * Throws an exception if:
         * - the query is invalid or fails to execute;
         * - the specified column does not exist;
         * - the value cannot be converted to a PHP-compatible type.
         */
        public function queryValue(string $query, ?array $parameters, ?mixed $column): mixed {}

        public function queryValueAssoc(string $query, ?array $parameters, ?mixed $column): mixed {}

        /**
         *
             * Executes an SQL query and returns a single column value as a PHP object from the first row.
             *
             * Same as `queryValue`, but forces object mode for decoding structured types (e.g., JSON, composite).
             *
             * # Parameters
             * - `query`: SQL query string to execute.
             * - `parameters`: Optional array of indexed or named parameters to bind.
             * - `column`: Optional column name or zero-based index to extract. Defaults to the first column.
             *
             * # Returns
             * The value from the specified column of the first row, decoded as a PHP object.
             *
             * # Exceptions
             * Throws an exception if:
             * - the query is invalid or fails to execute;
             * - the column does not exist;
             * - the value cannot be converted to a PHP object (e.g., due to encoding or type mismatch).

         */
        public function queryValueObj(string $query, ?array $parameters, ?mixed $column): mixed {}

        /**
         * Executes an SQL query and returns a single column value from the first row, or null if no rows matched.
         *
         * # Arguments
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional array of indexed/named parameters to bind.
         * - `column`: Optional column name or index to extract from the result row.
         *
         * # Returns
         * The value from the specified column of the first row as a PHP value`, or `null` if no row was found.
         *
         * # Exceptions
         * Throws an exception if:
         * - the query is invalid or fails to execute;
         * - the specified column does not exist;
         * - the value cannot be converted to a PHP-compatible type.
         */
        public function queryMaybeValue(string $query, ?array $parameters, ?mixed $column): mixed {}

        /**
         * Executes an SQL query and returns a single column value as a PHP value (array mode), or null if no row matched.
         *
         * Same as `query_maybe_value`, but forces associative array mode for complex values.
         *
         * # Arguments
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional array of indexed/named parameters to bind.
         * - `column`: Optional column name or index to extract.
         *
         * # Returns
         * The column value from the first row, or `null` if no row found.
         *
         * # Exceptions
         * Same as `query_maybe_value`.
         */
        public function queryMaybeValueAssoc(string $query, ?array $parameters, ?mixed $column): mixed {}

        /**
         * Executes an SQL query and returns a single column value as a PHP object, or null if no row matched.
         *
         * Same as `query_maybe_value`, but forces object mode for complex values.
         *
         * # Arguments
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional array of indexed/named parameters to bind.
         * - `column`: Optional column name or index to extract.
         *
         * # Returns
         * The column value from the first row, or `null` if no row found.
         *
         * # Exceptions
         * Same as `query_maybe_value`.
         */
        public function queryMaybeValueObj(string $query, ?array $parameters, ?mixed $column): mixed {}

        /**
         * Executes an SQL query and returns one row as an associative array.
         *
         * # Arguments
         * - `query`: SQL query string
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Exceptions
         * Throws an exception if:
         * - SQL query is invalid or execution fails;
         * - a parameter cannot be bound or has incorrect type;
         * - the row contains unsupported database types;
         * - conversion to PHP object fails.
         */
        public function queryRowAssoc(string $query, ?array $parameters): mixed {}

        /**
         * Executes an SQL query and returns one row as an object.
         *
         * # Arguments
         * - `query`: SQL query string
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Exceptions
         * Throws an exception if:
         * - SQL query is invalid or execution fails;
         * - a parameter cannot be bound or has incorrect type;
         * - the row contains unsupported database types;
         * - conversion to PHP object fails.
         */
        public function queryRowObj(string $query, ?array $parameters): mixed {}

        /**
         * Executes an SQL query and returns a single result, or `null` if no row matched.
         *
         * # Arguments
         * - `query`: SQL query string
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * Single row as array or object depending on config
         *
         * # Exceptions
         * Throws an exception if the query fails for reasons other than no matching rows.
         * For example, syntax errors, type mismatches, or database connection issues.
         */
        public function queryMaybeRow(string $query, ?array $parameters): mixed {}

        /**
         * Executes an SQL query and returns a single row as a PHP associative array, or `null` if no row matched.
         *
         * # Arguments
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * The result row as an associative array (`array<string, mixed>` in PHP), or `null` if no matching row is found.
         *
         * # Exceptions
         * Throws an exception if:
         * - the query is invalid or fails to execute;
         * - parameters are invalid or cannot be bound;
         * - the row contains unsupported or unconvertible data types.
         */
        public function queryMaybeRowAssoc(string $query, ?array $parameters): mixed {}

        /**
         * Executes an SQL query and returns a single row as a PHP object, or `null` if no row matched.
         *
         * # Arguments
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * The result row as a `stdClass` PHP object, or `null` if no matching row is found.
         *
         * # Exceptions
         * Throws an exception if:
         * - the query is invalid or fails to execute;
         * - parameters are invalid or cannot be bound;
         * - the row contains unsupported or unconvertible data types.
         */
        public function queryMaybeRowObj(string $query, ?array $parameters): mixed {}

        /**
         * Executes an SQL query and returns the specified column values from all result rows.
         *
         * # Arguments
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional array of indexed/named parameters to bind.
         * - `column`: Optional column name or index to extract.
         *
         * # Returns
         * An array of column values, one for each row.
         *
         * # Exceptions
         * Throws an exception if:
         * - the query fails to execute;
         * - the specified column is not found;
         * - a column value cannot be converted to PHP.
         */
        public function queryColumn(string $query, ?array $parameters, ?mixed $column): array {}

        /**
         * Executes an SQL query and returns the specified column values from all rows in associative array mode.
         *
         * # Arguments
         * - `query`: SQL query string.
         * - `parameters`: Optional named parameters.
         * - `column`: Column index or name to extract.
         *
         * # Returns
         * An array of column values (associative arrays for structured data).
         *
         * # Exceptions
         * Same as `query_column`.
         */
        public function queryColumnAssoc(string $query, ?array $parameters, ?mixed $column): array {}

        /**
         * Executes an SQL query and returns the specified column values from all rows in object mode.
         *
         * # Arguments
         * - `query`: SQL query string.
         * - `parameters`: Optional named parameters.
         * - `column`: Column index or name to extract.
         *
         * # Returns
         * An array of column values (objects for structured data).
         *
         * # Exceptions
         * Same as `query_column`.
         */
        public function queryColumnObj(string $query, ?array $parameters, ?mixed $column): array {}

        /**
         * Executes an SQL query and returns all results.
         *
         * # Arguments
         * - `query`: SQL query string
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Returns
         * Array of rows as array or object depending on config
         *
         * # Exceptions
         * Throws an exception if:
         * - SQL query is invalid or fails to execute;
         * - parameter binding fails;
         * - row decoding fails due to an unsupported or mismatched database type;
         * - conversion to PHP values fails (e.g., due to memory or encoding issues).
         */
        public function queryAll(string $query, ?array $parameters): array {}

        /**
         * Executes an SQL query and returns all rows as associative arrays.
         *
         * # Arguments
         * - `query`: SQL query string
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Exceptions
         * Throws an exception if:
         * - SQL query is invalid or fails to execute;
         * - parameter binding fails;
         * - row decoding fails due to an unsupported or mismatched database type;
         * - conversion to PHP values fails (e.g., due to memory or encoding issues).
         */
        public function queryAllAssoc(string $query, ?array $parameters): array {}

        /**
         * Executes an SQL query and returns all rows as objects.
         *
         * # Arguments
         * - `query`: SQL query string
         * - `parameters`: Optional array of indexed/named parameters to bind.
         *
         * # Exceptions
         * Throws an exception if:
         * - SQL query is invalid or fails to execute;
         * - parameter binding fails;
         * - row decoding fails due to an unsupported or mismatched database type;
         * - conversion to PHP values fails (e.g., due to memory or encoding issues).
         */
        public function queryAllObj(string $query, ?array $parameters): array {}

        /**
         * Executes an SQL query and returns a dictionary (map) indexed by the first column of each row.
         *
         * # Description
         * The result is a `HashMap` where each key is the string value of the first column in a row,
         * and the corresponding value is the row itself (as an array or object depending on config).
         *
         * This variant respects the global `assoc_arrays` setting to determine the row format.
         *
         * # Parameters
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional map of named parameters to bind.
         *
         * # Returns
         * A map from the first column (as string) to the full row.
         *
         * # Errors
         * - If the query fails to execute.
         * - If the first column cannot be converted to a string.
         * - If row decoding or PHP conversion fails.
         *
         * # Notes
         * - The iteration order of the returned map is **not** guaranteed.
         */
        public function queryDictionary(string $query, ?array $parameters): mixed {}

        /**
         * Executes an SQL query and returns a dictionary (map) indexed by the first column of each row,
         * returning each row as an associative array.
         *
         * # Parameters
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional map of named parameters to bind.
         *
         * # Returns
         * A map from the first column (as string) to the full row (as an associative array).
         *
         * # Errors
         * - If the query fails to execute.
         * - If the first column cannot be converted to a string.
         * - If row decoding or PHP conversion fails.
         *
         * # Notes
         * - The iteration order of the returned map is **not** guaranteed.
         */
        public function queryDictionaryAssoc(string $query, ?array $parameters): mixed {}

        /**
         * Executes an SQL query and returns a dictionary (map) indexed by the first column of each row,
         * returning each row as a PHP object.
         *
         * # Parameters
         * - `query`: SQL query string to execute.
         * - `parameters`: Optional map of named parameters to bind.
         *
         * # Returns
         * A map from the first column (as string) to the full row (as an object).
         *
         * # Errors
         * - If the query fails to execute.
         * - If the first column cannot be converted to a string.
         * - If row decoding or PHP conversion fails.
         *
         * # Notes
         * - The iteration order of the returned map is **not** guaranteed.
         */
        public function queryDictionaryObj(string $query, ?array $parameters): mixed {}

        /**
         * Executes an SQL query and returns a dictionary grouping rows by the first column.
         *
         * Each row in the result must contain at least one column. The **first column** is used as the **key**, and the
         * **entire row** is converted to a PHP value and added to the list associated with that key.
         *
         * # Parameters
         * - `query`: SQL query string, optionally using `$param`, `:param`, or positional `?` placeholders.
         * - `parameters`: Optional key–value map of parameters to bind into the query.
         * - `associative_arrays`: If `true`, rows are rendered as PHP associative arrays. If `false`, rows are rendered as objects.
         *   If `None`, falls back to the value in `OPT_ASSOC_ARRAYS`.
         *
         * # Returns
         * A `HashMap<String, Vec<Zval>>` mapping each unique value of the first column to a `Vec` of corresponding rows.
         *
         * # Example
         * A query like:
         * ```sql
         * SELECT category, name FROM products
         * ```
         * could produce:
         * ```php
         * [
         *   "Books" => [ ["category" => "Books", "name" => "Rust in Action"], ... ],
         *   "Toys"  => [ ["category" => "Toys", "name" => "Lego Set"], ... ],
         * ]
         * ```
         *
         * # Errors
         * Returns an error if:
         * - The query fails to render or execute.
         * - The first column in any row is `NULL` or cannot be converted to a PHP string.
         * - Any row cannot be fully converted to a PHP value.
         *
         * # Notes
         * - Row order within each group is preserved
         * - The outer dictionary order is preserved.
         * - Use this method when your result naturally groups by a field, e.g., for building nested structures or aggregations.
         */
        public function queryGroupedDictionary(string $query, ?array $parameters): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces each row to be represented as a PHP associative array.
         *
         * This overrides the driver’s default associative/object mode for this call only.
         *
         * # Errors
         * - If the first column is not convertible to string.
         * - If any row fails to convert to an associative array.
         */
        public function queryGroupedDictionaryAssoc(string $query, ?array $parameters): mixed {}

        /**
         * Same as `query_grouped_dictionary`, but forces each row to be represented as a PHP object.
         *
         * This overrides the driver’s default associative/object mode for this call only.
         *
         * # Errors
         * - If the first column is not convertible to string.
         * - If any row fails to convert to an object.
         */
        public function queryGroupedDictionaryObj(string $query, ?array $parameters): mixed {}

        /**
         * Executes an SQL query and returns a dictionary mapping the first column to the second column.
         *
         * This method expects each result row to contain at least two columns. It converts the first column
         * into a PHP string (used as the key), and the second column into a PHP value (used as the value).
         *
         * # Parameters
         * - `query`: SQL query string with optional placeholders (e.g., `$param`, `:param`, etc.).
         * - `parameters`: Optional associative array of parameters to bind into the query.
         *
         * # Returns
         * An associative array (`array<string, mixed>`) where each key is the first column (as string),
         * and the value is the second column.
         *
         * # Errors
         * Returns an error if:
         * - the query fails to render or execute;
         * - the first column cannot be converted to a PHP string;
         * - the second column cannot be converted to a PHP value.
         *
         * # Notes
         * - The iteration order of dictionary entries is preserved.
         * - The query must return at least two columns per row.
         */
        public function queryColumnDictionary(string $query, ?array $parameters): mixed {}

        /**
         * Executes an SQL query and returns a dictionary using associative array mode for values.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as associative arrays.
         *
         * # Parameters
         * - `query`: SQL query string.
         * - `parameters`: Optional associative array of bind parameters.
         *
         * # Returns
         * Dictionary where each key is the first column, and the value is the second column
         * converted into an associative PHP array.
         *
         * # Errors
         * Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryAssoc(string $query, ?array $parameters): mixed {}

        /**
         * Executes an SQL query and returns a dictionary using object mode for values.
         *
         * Same as `query_column_dictionary`, but forces JSON objects to be represented as PHP objects.
         *
         * # Parameters
         * - `query`: SQL query string.
         * - `parameters`: Optional associative array of bind parameters.
         *
         * # Returns
         * Dictionary where each key is the first column, and the value is the second column
         * converted into a PHP object.
         *
         * # Errors
         * Same as `query_column_dictionary`.
         */
        public function queryColumnDictionaryObj(string $query, ?array $parameters): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces associative arrays
         * for the second column if it's a JSON object.
         */
        public function queryGroupedColumnDictionaryAssoc(string $query, ?array $parameters): mixed {}

        /**
         * Same as `queryGroupedColumnDictionary()`, but forces PHP objects
         * for the second column if it's a JSON object.
         */
        public function queryGroupedColumnDictionaryObj(string $query, ?array $parameters): mixed {}

        /**
         * Executes an SQL query and returns a grouped dictionary where:
         * - the key is the **first column** (must be convertible to string),
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
         */
        public function queryGroupedColumnDictionary(string $query, ?array $parameters): mixed {}

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
         * # Exceptions
         * Throws an exception if:
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
         * # Exceptions
         * Throws an exception if:
         * - the SQL query is invalid or fails to execute (e.g., due to syntax error, constraint violation, or connection issue);
         * - parameters contain unsupported types or fail to bind correctly;
         * - the runtime fails to execute the query (e.g., task panic or timeout).
         */
        public function insert(string $table, array $row): int {}

        /**
         * Executes an SQL query and returns the rendered query and its parameters.
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
         * # Exceptions
         * Throws an exception if the query can't be parsed, rendered, or if parameters
         * cannot be converted from PHP values.
         */
        public function dry(string $query, ?array $parameters): array {}
    }
}
