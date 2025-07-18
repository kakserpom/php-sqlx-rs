<?php

// Stubs for php-sqlx

namespace Sqlx {
    function OR_(array $or): \Sqlx\OrClause {}

    /**
     * The `Sqlx\\SelectClause` class manages a set of allowed
     * columns for SQL SELECT operations and provides methods
     * to render validated column clauses from user input.
     */
    class SelectClause {
        public static function allowed(array $allowed_columns): \Sqlx\SelectClause {}

        /**
         * Magic `__invoke` method allowing the object to be
         * used as a callable for rendering select clauses.
         */
        public function __invoke(array $columns): \Sqlx\SelectClauseRendered {}

        /**
         * Renders validated SELECT clause columns from user input.
         *
         * # Arguments
         * - `columns`: List of column identifiers provided by user.
         *
         * # Returns
         * A `SelectClauseRendered` containing only allowed columns.
         * Unknown columns are silently ignored.
         */
        public function input(array $columns): \Sqlx\SelectClauseRendered {}

        /**
         * PHP constructor for `Sqlx\\SelectClause`.
         *
         * # Arguments
         * - `allowed_columns`: Associative array of allowed columns:
         *    - Numeric keys map to simple column names
         *    - String keys map to SQL expressions
         *
         * # Example
         * ```php
         * $select = new Sqlx\\SelectClause([
         *     "id",
         *     "name",
         *     "full_name" => "CONCAT(first, ' ', last)"
         * ]);
         * ```
         */
        public function __construct(array $allowed_columns) {}
    }

    /**
     * The `SelectClauseRendered` struct holds validated
     * column clauses for SQL SELECT statements.
     */
    class SelectClauseRendered {
        public function __construct() {}
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
        const ASC = null;

        /**
         * Descending order (Z to A)
         */
        const DESC = null;

        /**
         * `__invoke` magic for apply().
         */
        public function __invoke(array $columns): \Sqlx\ByClauseRendered {}

        /**
         * Applies ordering rules to a user-defined input.
         *
         * # Arguments
         * - `columns`: List of columns (as strings or [field, direction] arrays)
         *
         * # Returns
         * A `ByClauseRendered` object containing validated SQL ORDER BY clauses.
         * The resulting value is to be used as a placeholder in query bindings.
         *
         * # Notes
         * Unknown or disallowed fields are silently ignored.
         */
        public function input(array $columns): \Sqlx\ByClauseRendered {}

        /**
         * Constructs a `ByClause` helper with allowed sortable columns.
         *
         * # Arguments
         * - `allowed_columns`: Map of allowed sort columns (key = user input, value = SQL expression)
         *
         * # Example
         * ```php
         * $order_by = new Sqlx\ByClause([
         *     "name",
         *     "age",
         *     "total_posts" => "COUNT(posts.*)"
         * ]);
         * ```
         */
        public function __construct(array $allowed_columns) {}
    }

    /**
     * A rendered ORDER BY / GROUP BY clause result for use in query generation.
     */
    class ByClauseRendered {
        public function __construct() {}
    }

    /**
     * The `Sqlx\PaginateClause` class represents pagination settings
     * and provides methods to compute the appropriate SQL `LIMIT` and `OFFSET`
     * based on a given page number and items-per-page values.
     */
    class PaginateClause {
        /**
         * Magic `__invoke` method allowing the object to be used as a callable
         * for applying pagination.
         *
         * # Parameters
         * - `page_number`: Optional page index.
         * - `per_page`: Optional items per page.
         *
         * # Returns
         * A `PaginateClauseRendered` with calculated `limit` and `offset`.
         */
        public function __invoke(?int $page_number, ?int $per_page): \Sqlx\PaginateClauseRendered {}

        /**
         * Sets a fixed number of items per page.
         *
         * Updates `min_per_page`, `max_per_page`, and `default_per_page`
         * to the provided value.
         *
         * # Errors
         * Returns an error if `per_page < 1`.
         */
        public function perPage(int $per_page): mixed {}

        /**
         * Sets the minimum number of items per page.
         *
         * Ensures `max_per_page` and `default_per_page` are at least
         * the new minimum value.
         *
         * # Errors
         * Returns an error if `min_per_page < 1`.
         */
        public function minPerPage(int $min_per_page): mixed {}

        /**
         * Sets the maximum number of items per page.
         *
         * Ensures `min_per_page` and `default_per_page` do not exceed
         * the new maximum value.
         *
         * # Errors
         * Returns an error if `max_per_page < 1`.
         */
        public function maxPerPage(int $max_per_page): mixed {}

        /**
         * Applies pagination settings and returns a `PaginateClauseRendered`.
         *
         * # Parameters and behavior are identical to `render`.
         */
        public function input(?int $page_number, ?int $per_page): \Sqlx\PaginateClauseRendered {}

        /**
         * PHP constructor for `Sqlx\PaginateClause`.
         */
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

    class OrClause {
        public function __construct() {}
    }

    /**
     * This class supports prepared queries, persistent connections, and augmented SQL.
     */
    class MySqlDriver {
        /**
         * Creates a prepared query object with the given SQL string.
         *
         * # Arguments
         * - `query`: SQL query string to prepare
         *
         * # Returns
         * Prepared query object
         */
        public function prepare(string $query): \Sqlx\MySqlPreparedQuery {}

        /**
         * Creates a query builder object
         *
         *
         * # Returns
         * Query builder object
         */
        public function builder(): \Sqlx\MySqlQueryBuilder {}

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
         * # Arguments
         * * `param` – The parameter to quote (must be a scalar: string, number, or boolean).
         *
         * # Returns
         * Quoted SQL string (e.g., `'abc'`, `123`, `TRUE`)
         *
         * # Errors
         * Returns an error if the parameter is not a scalar or if rendering fails.
         *
         * # Example
         * ```php
         * $driver->builder()->quote("O'Reilly"); // "'O''Reilly'"
         * ```
         */
        public function quote(mixed $param): string {}

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
         * # Arguments
         * * `param` – The parameter to escape (must be a string).
         *
         * # Returns
         * A string with `%` and `_` escaped for LIKE (e.g., `foo\%bar\_baz`)
         *
         * # Errors
         * Returns an error if the input is not a string.
         *
         * # Example
         * ```php
         * $escaped = $builder->metaQuoteLike("100%_safe");
         * // Use like:
         * $builder->where([["name", "LIKE", "%$escaped%"]]);
         * ```
         */
        public function metaQuoteLike(mixed $param): string {}

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
         */
        public function queryGroupedColumnDictionary(string $query, ?array $parameters): mixed {}

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
         * Inserts a row into the given table using a map of columns.
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

        /**
         * Begins a new transaction, yields control to the provided callable,
         * and commits or rolls back based on the callable's return value or error.
         *
         * # Parameters
         * - `callable`: A PHP callable receiving this Driver instance.
         *
         * # Behavior
         * - Starts a transaction.
         * - Invokes `callable($this)`.
         * - If the callable returns false, rolls back, and commits otherwise.
         * - On exception or callable error, rolls back and rethrows.
         *
         * # Exceptions
         * Throws an exception if transaction commit, rollback,
         * or callable invocation fails.
         *
         */
        public function begin(callable $callable): mixed {}

        /**
         * Creates a transaction savepoint with the given name.
         *
         * # Parameters
         * - `savepoint`: Name of the savepoint to create.
         *
         * # Exceptions
         * Throws an exception if the driver fails to create the savepoint.
         */
        public function savepoint(string $savepoint): mixed {}

        /**
         * Rolls back the current transaction to a previously created savepoint.
         *
         * # Parameters
         * - `savepoint`: Name of the savepoint to rollback to.
         *
         * # Exceptions
         * Throws an exception if rollback to the savepoint fails.
         */
        public function rollbackToSavepoint(string $savepoint): mixed {}

        /**
         * Releases a previously created savepoint, making it no longer available.
         *
         * # Parameters
         * - `savepoint`: Name of the savepoint to release.
         *
         * # Exceptions
         * Throws an exception if releasing the savepoint fails.
         */
        public function releaseSavepoint(string $savepoint): mixed {}

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
    }

    /**
     * A reusable prepared SQL query with parameter support. Created using `PgDriver::prepare()`, shares context with original driver.
     */
    class MySqlPreparedQuery {
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
         * The first column is used as the key (must be scalar),
         * and each resulting row is appended to the corresponding key's Vec.
         *
         * # Errors
         * Fails if the query fails, or the first column is not scalar.
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
         * - the key is the **first column** (must be scalar),
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

        public function __construct() {}
    }

    /**
     * A prepared SQL query builder.
     *
     * Holds the generated query string, parameters, and placeholder tracking
     * used during safe, composable query construction via AST rendering.
     */
    class MySqlQueryBuilder {
        /**
         * Creates a query builder object
         *
         *
         * # Returns
         * Query builder object
         */
        public function builder(): \Sqlx\MySqlQueryBuilder {}

        public static function factory(\Sqlx\MySqlDriver $driver): \Sqlx\MySqlQueryBuilder {}

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
         * # Arguments
         * * `param` – The parameter to quote (must be a scalar: string, number, or boolean).
         *
         * # Returns
         * Quoted SQL string (e.g., `'abc'`, `123`, `TRUE`)
         *
         * # Errors
         * Returns an error if the parameter is not a scalar or if rendering fails.
         *
         * # Example
         * ```php
         * $driver->builder()->quote("O'Reilly"); // "'O''Reilly'"
         * ```
         */
        public function quote(mixed $param): string {}

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
         * # Arguments
         * * `param` – The parameter to escape (must be a string).
         *
         * # Returns
         * A string with `%` and `_` escaped for LIKE (e.g., `foo\%bar\_baz`)
         *
         * # Errors
         * Returns an error if the input is not a string.
         *
         * # Example
         * ```php
         * $escaped = $builder->metaQuoteLike("100%_safe");
         * // Use like:
         * $builder->where([["name", "LIKE", "%$escaped%"]]);
         * ```
         */
        public function metaQuoteLike(mixed $param): string {}

        /**
         * Appends an `ON CONFLICT` clause to the query.
         *
         * # Arguments
         * * `target` – A string or array of column names to specify the conflict target.
         * * `set` – Optional `SET` clause. If `null`, generates `DO NOTHING`; otherwise uses the `SET` values.
         *
         * # Example
         * ```php
         * $builder->onConflict("id", null);
         * $builder->onConflict(["email", "tenant_id"], ["name" => "New"]);
         * ```
         */
        public function onConflict(mixed $target, ?mixed $set): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends an `ON DUPLICATE KEY UPDATE` clause to the query (MySQL).
         *
         * # Arguments
         * * `set` – An array representing fields and values to update.
         *
         * # Example
         * ```php
         * $builder->onDuplicateKeyUpdate(["email" => "new@example.com"]);
         * ```
         */
        public function onDuplicateKeyUpdate(mixed $set): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends an `INNER JOIN` clause to the query.
         */
        public function innerJoin(string $table, string $on, ?array $parameters): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends an `INNER JOIN` clause to the query.
         * Alias for `inner_join()`.
         */
        public function join(string $table, string $on, ?array $parameters): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `LEFT JOIN` clause to the query.
         */
        public function leftJoin(string $table, string $on, ?array $parameters): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `RIGHT JOIN` clause to the query.
         */
        public function rightJoin(string $table, string $on, ?array $parameters): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `FULL OUTER JOIN` clause to the query.
         */
        public function fullJoin(string $table, string $on, ?array $parameters): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `CROSS JOIN` clause to the query.
         */
        public function crossJoin(string $table, string $on, ?array $parameters): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `NATURAL JOIN` clause to the query.
         *
         * # Note
         * `NATURAL JOIN` does not accept `ON` conditions. The `on` argument is ignored.
         */
        public function naturalJoin(string $table): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `WITH` clause to the query.
         *
         * # Arguments
         * * `table` - Name of the CTE (common table expression).
         * * `as` - The query body to be rendered for the CTE.
         * * `parameters` - Optional parameters for the query body.
         *
         * # Example
         * ```php
         * $builder->with("cte_name", "SELECT * FROM users WHERE active = $active", [
         *     "active" => true,
         * ]);
         * ```
         */
        public function with(string $table, mixed $r#as, ?array $parameters): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `WHERE` clause to the query.
         *
         * # Arguments
         * * `where` - Either a raw string, a structured array of conditions, or a disjunction (`OrClause`).
         * * `parameters` - Optional parameters associated with the `WHERE` condition.
         *
         * # Exceptions
         * Throws an exception if the input is not valid.
         */
        public function where(mixed $r#where, ?array $parameters): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `UNION` clause to the query.
         *
         * # Arguments
         * * `query` – A raw SQL string or another Builder instance (subquery).
         * * `parameters` – Optional parameters to bind to the unioned subquery.
         *
         * # Example
         * ```php
         * $builder->union("SELECT id FROM users");
         * $builder->union($other_builder);
         * ```
         */
        public function union(mixed $query, ?array $parameters): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `UNION ALL` clause to the query.
         *
         * # Arguments
         * * `query` – A raw SQL string or another Builder instance (subquery).
         * * `parameters` – Optional parameters to bind to the unioned subquery.
         *
         * # Example
         * ```php
         * $builder->union_all("SELECT id FROM users");
         * $builder->union_all($other_builder);
         * ```
         */
        public function unionAll(mixed $query, ?array $parameters): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `HAVING` clause to the query.
         *
         * # Arguments
         * * `having` - Either a raw string, a structured array of conditions, or a disjunction (`OrClause`).
         * * `parameters` - Optional parameters associated with the `WHERE` condition.
         *
         * # Exceptions
         * Throws an exception if the input is not valid.
         */
        public function having(mixed $having, ?array $parameters): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `LIMIT` (and optional `OFFSET`) clause to the query.
         *
         * # Arguments
         * * `limit` – Maximum number of rows to return.
         * * `offset` – Optional number of rows to skip before starting to return rows.
         *
         * # Example
         * ```php
         * $builder->limit(10);
         * $builder->limit(10, 20); // LIMIT 10 OFFSET 20
         * ```
         */
        public function limit(int $limit, ?int $offset): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends an `OFFSET` clause to the query independently.
         *
         * # Arguments
         * * `offset` – Number of rows to skip before returning results.
         *
         * # Example
         * ```php
         * $builder->offset(30); // OFFSET 30
         * ```
         */
        public function offset(int $offset): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `DELETE FROM` clause to the query.
         *
         * # Arguments
         * * `from` - A string table name or a nested builder object.
         * * `parameters` - Optional parameters if the `from` is a raw string.
         *
         * # Examples
         * ```php
         * $builder->deleteFrom("users");
         * $builder->deleteFrom($builder->select("id")->from("temp_users"));
         * ```
         */
        public function deleteFrom(mixed $from, ?array $parameters): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `USING` clause to the query.
         *
         * # Arguments
         * * `from` - Either a string table name or a subquery builder.
         * * `parameters` - Optional parameters if `from` is a string.
         */
        public function using(mixed $from, ?array $parameters): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `PAGINATE` clause to the query using a `PaginateClauseRendered` object.
         *
         * This expands into the appropriate SQL `LIMIT` and `OFFSET` syntax during rendering,
         * using the values stored in the given `PaginateClauseRendered` instance.
         *
         * # Arguments
         * * `paginate` – An instance of `Sqlx\PaginateClauseRendered`, produced by invoking a `PaginateClause`
         *               (e.g., `$paginate = (new PaginateClause)($page, $perPage)`).
         *
         * # Errors
         * Returns an error if the argument is not an instance of `PaginateClauseRendered`.
         *
         * # Example
         * ```php
         * $paginate = (new \Sqlx\PaginateClause())->__invoke(2, 10); // page 2, 10 per page
         * $builder->select("*")->from("users")->paginate($paginate);
         * // SELECT * FROM users LIMIT 10 OFFSET 20
         * ```
         */
        public function paginate(mixed $paginate): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `WITH RECURSIVE` clause to the query.
         *
         * # Arguments
         * * `table_and_fields` - Table name with field list, e.g. `cte(col1, col2)`.
         * * `as` - The recursive query body.
         * * `parameters` - Optional parameters for the recursive body.
         *
         * # Example
         * ```php
         * $builder->withRecursive("cte(id, parent)", "SELECT ...", [...]);
         * ```
         */
        public function withRecursive(string $table_and_fields, mixed $r#as, ?array $parameters): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `UPDATE` clause to the query.
         *
         * # Arguments
         * * `table` - A raw string representing the table(s).
         *
         * # Exceptions
         * Throws an exception if the argument is not a string.
         */
        public function update(mixed $table): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `SET` clause to the query, supporting both keyed and indexed formats.
         *
         * # Arguments
         * * `set` - An associative array mapping fields to values, or a sequential array
         *   of `[field, value]` pairs or raw fragments.
         *
         * # Supported Formats
         * ```php
         * $builder->set(["name" => "John", "age" => 30]);
         * $builder->set([["name", "John"], ["age", 30]]);
         * $builder->set(["updated_at = NOW()"]);
         * ```
         *
         * # Exceptions
         * - When the input array is malformed or contains invalid value types.
         */
        public function set(mixed $set): \Sqlx\MySqlQueryBuilder {}

        /**
         * Renders the SQL query using named parameters and returns the fully rendered SQL with values injected inline.
         *
         * # Returns
         * The rendered SQL query as a string with all parameters interpolated.
         *
         * # Exceptions
         * - If rendering or encoding of parameters fails.
         */
        public function dry(): array {}

        /**
         * Returns the parameter map currently accumulated in the builder.
         *
         * # Returns
         * A cloned map of all parameter names and their corresponding `ParameterValue`.
         *
         * # Exceptions
         * - If rendering or encoding of parameters fails.
         */
        public function dryInline(): string {}

        /**
         * Returns the fully rendered SQL query with parameters embedded as literals.
         *
         * Used for debugging or fallback rendering when the placeholder limit is exceeded.
         *
         * # Returns
         * A string representing the complete SQL statement.
         *
         * # Exceptions
         * - If rendering or encoding of parameters fails.
         */
        public function __toString(): string {}

        /**
         * Returns an array of all currently accumulated parameters.
         */
        public function parameters(): array {}

        /**
         * Appends a raw SQL fragment to the query without validation.
         *
         * # Arguments
         * * `part` - The SQL string to append.
         * * `parameters` - Optional parameters to associate with the fragment.
         */
        public function raw(string $part, ?array $parameters): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `SELECT` clause to the query.
         *
         * # Arguments
         * * `fields` - Either a raw string or a `SelectClauseRendered` object.
         *
         * # Exceptions
         * Throws an exception if the argument is not a string or a supported object.
         */
        public function select(mixed $fields): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `ORDER BY` clause to the query.
         *
         * # Arguments
         * * `fields` - Either a raw string or a `ByClauseRendered` object.
         *
         * # Exceptions
         * Throws an exception if the argument is not a string or a supported object.
         */
        public function orderBy(mixed $fields): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `GROUP BY` clause to the query.
         *
         * # Arguments
         * * `fields` - Either a raw string or a `ByClauseRendered` object.
         *
         * # Exceptions
         * Throws an exception if the argument is not a string or a supported object.
         */
        public function groupBy(mixed $fields): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `FOR UPDATE` locking clause to the query.
         *
         * This clause is used in `SELECT` statements to lock the selected rows
         * for update, preventing other transactions from modifying or acquiring
         * locks on them until the current transaction completes.
         *
         * # Example
         * ```php
         * $builder->select("*")->from("users")->for_update();
         * // SELECT * FROM users FOR UPDATE
         * ```
         *
         * # Notes
         * - Only valid in transactional contexts (e.g., PostgreSQL, MySQL with InnoDB).
         * - Useful for implementing pessimistic locking in concurrent systems.
         *
         * # Returns
         * The query builder with `FOR UPDATE` appended.
         */
        public function forUpdate(): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `FOR SHARE` locking clause to the query.
         *
         * This clause is used in `SELECT` statements to acquire shared locks
         * on the selected rows, allowing concurrent transactions to read but
         * not modify the rows until the current transaction completes.
         *
         * # Example
         * ```php
         * $builder->select("*")->from("documents")->for_share();
         * // SELECT * FROM documents FOR SHARE
         * ```
         *
         * # Notes
         * - Supported in PostgreSQL and some MySQL configurations.
         * - Ensures rows cannot be updated or deleted by other transactions while locked.
         *
         * # Returns
         * The query builder with `FOR SHARE` appended.
         */
        public function forShare(): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends an `INSERT INTO` clause to the query.
         *
         * # Arguments
         * * `table` - The name of the target table.
         *
         * # Example
         * ```php
         * $builder->insert("users");
         * ```
         */
        public function insert(string $table): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `VALUES` clause to the query.
         *
         * # Arguments
         * * `values` - Can be:
         *     - An associative array: `["name" => "John", "email" => "j@example.com"]`
         *     - A list of `[column, value]` pairs: `[["name", "John"], ["email", "j@example.com"]]`
         *     - A raw SQL string or a subquery builder
         *
         * # Example
         * ```php
         * $builder->insert("users")->values(["name" => "John", "email" => "j@example.com"]);
         * ```
         */
        public function values(mixed $values): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `TRUNCATE TABLE` statement to the query.
         *
         * This command removes all rows from the specified table quickly and efficiently.
         * It is faster than `DELETE FROM` and usually does not fire triggers or return affected row counts.
         *
         * # Arguments
         * * `table` – The name of the table to truncate.
         *
         * # Example
         * ```php
         * $builder->truncate_table("users");
         * // TRUNCATE TABLE users
         * ```
         *
         * # Notes
         * - May require elevated privileges depending on the database.
         * - This method can be chained with other query builder methods.
         *
         * # Errors
         * Returns an error if appending the SQL fragment fails (e.g., formatting error).
         */
        public function truncateTable(string $table): \Sqlx\MySqlQueryBuilder {}

        /**
         * Finalizes the query by appending a semicolon (`;`).
         *
         * This method is optional. Most databases do not require semicolons in prepared queries,
         * but you may use it to explicitly terminate a query string.
         *
         * # Example
         * ```php
         * $builder->select("*")->from("users")->end();
         * // SELECT * FROM users;
         * ```
         *
         * # Returns
         * The builder instance after appending the semicolon.
         *
         * # Notes
         * - Only appends the semicolon character; does not perform any execution.
         * - Useful when exporting a full query string with a terminating symbol (e.g., for SQL scripts).
         */
        public function end(string $_table): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends multiple rows to the `VALUES` clause of an `INSERT` statement.
         *
         * Each row must be:
         * - an ordered list of values (indexed array),
         * - or a map of column names to values (associative array) — only for the first row, to infer column order.
         *
         * # Arguments
         * * `rows` – A sequential array of rows (arrays of values).
         *
         * # Example
         * ```php
         * $builder->insert("users")->values_many([
         *     ["Alice", "alice@example.com"],
         *     ["Bob", "bob@example.com"]
         * ]);
         *
         * $builder->insert("users")->values_many([
         *     ["name" => "Alice", "email" => "alice@example.com"],
         *     ["name" => "Bob",   "email" => "bob@example.com"]
         * ]);
         * ```
         */
        public function valuesMany(mixed $rows): \Sqlx\MySqlQueryBuilder {}

        /**
         * Appends a `RETURNING` clause to the query.
         *
         * # Arguments
         * * `fields` - A string or array of column names to return.
         *
         * # Supported formats
         * ```php
         * $builder->returning("id");
         * $builder->returning(["id", "name"]);
         * ```
         *
         * # Notes
         * - This is mainly supported in PostgreSQL.
         * - Use with `INSERT`, `UPDATE`, or `DELETE`.
         */
        public function returning(mixed $fields): \Sqlx\MySqlQueryBuilder {}

        public function from(mixed $from, ?array $parameters): \Sqlx\MySqlQueryBuilder {}

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
         * The first column is used as the key (must be scalar),
         * and each resulting row is appended to the corresponding key's Vec.
         *
         * # Errors
         * Fails if the query fails, or the first column is not scalar.
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
         * - the key is the **first column** (must be scalar),
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

        public function __construct() {}
    }

    /**
     * This class supports prepared queries, persistent connections, and augmented SQL.
     */
    class PgDriver {
        /**
         * Creates a prepared query object with the given SQL string.
         *
         * # Arguments
         * - `query`: SQL query string to prepare
         *
         * # Returns
         * Prepared query object
         */
        public function prepare(string $query): \Sqlx\PgPreparedQuery {}

        /**
         * Creates a query builder object
         *
         *
         * # Returns
         * Query builder object
         */
        public function builder(): \Sqlx\PgQueryBuilder {}

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
         * # Arguments
         * * `param` – The parameter to quote (must be a scalar: string, number, or boolean).
         *
         * # Returns
         * Quoted SQL string (e.g., `'abc'`, `123`, `TRUE`)
         *
         * # Errors
         * Returns an error if the parameter is not a scalar or if rendering fails.
         *
         * # Example
         * ```php
         * $driver->builder()->quote("O'Reilly"); // "'O''Reilly'"
         * ```
         */
        public function quote(mixed $param): string {}

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
         * # Arguments
         * * `param` – The parameter to escape (must be a string).
         *
         * # Returns
         * A string with `%` and `_` escaped for LIKE (e.g., `foo\%bar\_baz`)
         *
         * # Errors
         * Returns an error if the input is not a string.
         *
         * # Example
         * ```php
         * $escaped = $builder->metaQuoteLike("100%_safe");
         * // Use like:
         * $builder->where([["name", "LIKE", "%$escaped%"]]);
         * ```
         */
        public function metaQuoteLike(mixed $param): string {}

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
         */
        public function queryGroupedColumnDictionary(string $query, ?array $parameters): mixed {}

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
         * Inserts a row into the given table using a map of columns.
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

        /**
         * Begins a new transaction, yields control to the provided callable,
         * and commits or rolls back based on the callable's return value or error.
         *
         * # Parameters
         * - `callable`: A PHP callable receiving this Driver instance.
         *
         * # Behavior
         * - Starts a transaction.
         * - Invokes `callable($this)`.
         * - If the callable returns false, rolls back, and commits otherwise.
         * - On exception or callable error, rolls back and rethrows.
         *
         * # Exceptions
         * Throws an exception if transaction commit, rollback,
         * or callable invocation fails.
         *
         */
        public function begin(callable $callable): mixed {}

        /**
         * Creates a transaction savepoint with the given name.
         *
         * # Parameters
         * - `savepoint`: Name of the savepoint to create.
         *
         * # Exceptions
         * Throws an exception if the driver fails to create the savepoint.
         */
        public function savepoint(string $savepoint): mixed {}

        /**
         * Rolls back the current transaction to a previously created savepoint.
         *
         * # Parameters
         * - `savepoint`: Name of the savepoint to rollback to.
         *
         * # Exceptions
         * Throws an exception if rollback to the savepoint fails.
         */
        public function rollbackToSavepoint(string $savepoint): mixed {}

        /**
         * Releases a previously created savepoint, making it no longer available.
         *
         * # Parameters
         * - `savepoint`: Name of the savepoint to release.
         *
         * # Exceptions
         * Throws an exception if releasing the savepoint fails.
         */
        public function releaseSavepoint(string $savepoint): mixed {}

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
    }

    /**
     * A reusable prepared SQL query with parameter support. Created using `PgDriver::prepare()`, shares context with original driver.
     */
    class PgPreparedQuery {
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
         * The first column is used as the key (must be scalar),
         * and each resulting row is appended to the corresponding key's Vec.
         *
         * # Errors
         * Fails if the query fails, or the first column is not scalar.
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
         * - the key is the **first column** (must be scalar),
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

        public function __construct() {}
    }

    /**
     * A prepared SQL query builder.
     *
     * Holds the generated query string, parameters, and placeholder tracking
     * used during safe, composable query construction via AST rendering.
     */
    class PgQueryBuilder {
        /**
         * Creates a query builder object
         *
         *
         * # Returns
         * Query builder object
         */
        public function builder(): \Sqlx\PgQueryBuilder {}

        public static function factory(\Sqlx\PgDriver $driver): \Sqlx\PgQueryBuilder {}

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
         * # Arguments
         * * `param` – The parameter to quote (must be a scalar: string, number, or boolean).
         *
         * # Returns
         * Quoted SQL string (e.g., `'abc'`, `123`, `TRUE`)
         *
         * # Errors
         * Returns an error if the parameter is not a scalar or if rendering fails.
         *
         * # Example
         * ```php
         * $driver->builder()->quote("O'Reilly"); // "'O''Reilly'"
         * ```
         */
        public function quote(mixed $param): string {}

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
         * # Arguments
         * * `param` – The parameter to escape (must be a string).
         *
         * # Returns
         * A string with `%` and `_` escaped for LIKE (e.g., `foo\%bar\_baz`)
         *
         * # Errors
         * Returns an error if the input is not a string.
         *
         * # Example
         * ```php
         * $escaped = $builder->metaQuoteLike("100%_safe");
         * // Use like:
         * $builder->where([["name", "LIKE", "%$escaped%"]]);
         * ```
         */
        public function metaQuoteLike(mixed $param): string {}

        /**
         * Appends an `ON CONFLICT` clause to the query.
         *
         * # Arguments
         * * `target` – A string or array of column names to specify the conflict target.
         * * `set` – Optional `SET` clause. If `null`, generates `DO NOTHING`; otherwise uses the `SET` values.
         *
         * # Example
         * ```php
         * $builder->onConflict("id", null);
         * $builder->onConflict(["email", "tenant_id"], ["name" => "New"]);
         * ```
         */
        public function onConflict(mixed $target, ?mixed $set): \Sqlx\PgQueryBuilder {}

        /**
         * Appends an `ON DUPLICATE KEY UPDATE` clause to the query (MySQL).
         *
         * # Arguments
         * * `set` – An array representing fields and values to update.
         *
         * # Example
         * ```php
         * $builder->onDuplicateKeyUpdate(["email" => "new@example.com"]);
         * ```
         */
        public function onDuplicateKeyUpdate(mixed $set): \Sqlx\PgQueryBuilder {}

        /**
         * Appends an `INNER JOIN` clause to the query.
         */
        public function innerJoin(string $table, string $on, ?array $parameters): \Sqlx\PgQueryBuilder {}

        /**
         * Appends an `INNER JOIN` clause to the query.
         * Alias for `inner_join()`.
         */
        public function join(string $table, string $on, ?array $parameters): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `LEFT JOIN` clause to the query.
         */
        public function leftJoin(string $table, string $on, ?array $parameters): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `RIGHT JOIN` clause to the query.
         */
        public function rightJoin(string $table, string $on, ?array $parameters): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `FULL OUTER JOIN` clause to the query.
         */
        public function fullJoin(string $table, string $on, ?array $parameters): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `CROSS JOIN` clause to the query.
         */
        public function crossJoin(string $table, string $on, ?array $parameters): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `NATURAL JOIN` clause to the query.
         *
         * # Note
         * `NATURAL JOIN` does not accept `ON` conditions. The `on` argument is ignored.
         */
        public function naturalJoin(string $table): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `WITH` clause to the query.
         *
         * # Arguments
         * * `table` - Name of the CTE (common table expression).
         * * `as` - The query body to be rendered for the CTE.
         * * `parameters` - Optional parameters for the query body.
         *
         * # Example
         * ```php
         * $builder->with("cte_name", "SELECT * FROM users WHERE active = $active", [
         *     "active" => true,
         * ]);
         * ```
         */
        public function with(string $table, mixed $r#as, ?array $parameters): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `WHERE` clause to the query.
         *
         * # Arguments
         * * `where` - Either a raw string, a structured array of conditions, or a disjunction (`OrClause`).
         * * `parameters` - Optional parameters associated with the `WHERE` condition.
         *
         * # Exceptions
         * Throws an exception if the input is not valid.
         */
        public function where(mixed $r#where, ?array $parameters): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `UNION` clause to the query.
         *
         * # Arguments
         * * `query` – A raw SQL string or another Builder instance (subquery).
         * * `parameters` – Optional parameters to bind to the unioned subquery.
         *
         * # Example
         * ```php
         * $builder->union("SELECT id FROM users");
         * $builder->union($other_builder);
         * ```
         */
        public function union(mixed $query, ?array $parameters): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `UNION ALL` clause to the query.
         *
         * # Arguments
         * * `query` – A raw SQL string or another Builder instance (subquery).
         * * `parameters` – Optional parameters to bind to the unioned subquery.
         *
         * # Example
         * ```php
         * $builder->union_all("SELECT id FROM users");
         * $builder->union_all($other_builder);
         * ```
         */
        public function unionAll(mixed $query, ?array $parameters): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `HAVING` clause to the query.
         *
         * # Arguments
         * * `having` - Either a raw string, a structured array of conditions, or a disjunction (`OrClause`).
         * * `parameters` - Optional parameters associated with the `WHERE` condition.
         *
         * # Exceptions
         * Throws an exception if the input is not valid.
         */
        public function having(mixed $having, ?array $parameters): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `LIMIT` (and optional `OFFSET`) clause to the query.
         *
         * # Arguments
         * * `limit` – Maximum number of rows to return.
         * * `offset` – Optional number of rows to skip before starting to return rows.
         *
         * # Example
         * ```php
         * $builder->limit(10);
         * $builder->limit(10, 20); // LIMIT 10 OFFSET 20
         * ```
         */
        public function limit(int $limit, ?int $offset): \Sqlx\PgQueryBuilder {}

        /**
         * Appends an `OFFSET` clause to the query independently.
         *
         * # Arguments
         * * `offset` – Number of rows to skip before returning results.
         *
         * # Example
         * ```php
         * $builder->offset(30); // OFFSET 30
         * ```
         */
        public function offset(int $offset): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `DELETE FROM` clause to the query.
         *
         * # Arguments
         * * `from` - A string table name or a nested builder object.
         * * `parameters` - Optional parameters if the `from` is a raw string.
         *
         * # Examples
         * ```php
         * $builder->deleteFrom("users");
         * $builder->deleteFrom($builder->select("id")->from("temp_users"));
         * ```
         */
        public function deleteFrom(mixed $from, ?array $parameters): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `USING` clause to the query.
         *
         * # Arguments
         * * `from` - Either a string table name or a subquery builder.
         * * `parameters` - Optional parameters if `from` is a string.
         */
        public function using(mixed $from, ?array $parameters): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `PAGINATE` clause to the query using a `PaginateClauseRendered` object.
         *
         * This expands into the appropriate SQL `LIMIT` and `OFFSET` syntax during rendering,
         * using the values stored in the given `PaginateClauseRendered` instance.
         *
         * # Arguments
         * * `paginate` – An instance of `Sqlx\PaginateClauseRendered`, produced by invoking a `PaginateClause`
         *               (e.g., `$paginate = (new PaginateClause)($page, $perPage)`).
         *
         * # Errors
         * Returns an error if the argument is not an instance of `PaginateClauseRendered`.
         *
         * # Example
         * ```php
         * $paginate = (new \Sqlx\PaginateClause())->__invoke(2, 10); // page 2, 10 per page
         * $builder->select("*")->from("users")->paginate($paginate);
         * // SELECT * FROM users LIMIT 10 OFFSET 20
         * ```
         */
        public function paginate(mixed $paginate): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `WITH RECURSIVE` clause to the query.
         *
         * # Arguments
         * * `table_and_fields` - Table name with field list, e.g. `cte(col1, col2)`.
         * * `as` - The recursive query body.
         * * `parameters` - Optional parameters for the recursive body.
         *
         * # Example
         * ```php
         * $builder->withRecursive("cte(id, parent)", "SELECT ...", [...]);
         * ```
         */
        public function withRecursive(string $table_and_fields, mixed $r#as, ?array $parameters): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `UPDATE` clause to the query.
         *
         * # Arguments
         * * `table` - A raw string representing the table(s).
         *
         * # Exceptions
         * Throws an exception if the argument is not a string.
         */
        public function update(mixed $table): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `SET` clause to the query, supporting both keyed and indexed formats.
         *
         * # Arguments
         * * `set` - An associative array mapping fields to values, or a sequential array
         *   of `[field, value]` pairs or raw fragments.
         *
         * # Supported Formats
         * ```php
         * $builder->set(["name" => "John", "age" => 30]);
         * $builder->set([["name", "John"], ["age", 30]]);
         * $builder->set(["updated_at = NOW()"]);
         * ```
         *
         * # Exceptions
         * - When the input array is malformed or contains invalid value types.
         */
        public function set(mixed $set): \Sqlx\PgQueryBuilder {}

        /**
         * Renders the SQL query using named parameters and returns the fully rendered SQL with values injected inline.
         *
         * # Returns
         * The rendered SQL query as a string with all parameters interpolated.
         *
         * # Exceptions
         * - If rendering or encoding of parameters fails.
         */
        public function dry(): array {}

        /**
         * Returns the parameter map currently accumulated in the builder.
         *
         * # Returns
         * A cloned map of all parameter names and their corresponding `ParameterValue`.
         *
         * # Exceptions
         * - If rendering or encoding of parameters fails.
         */
        public function dryInline(): string {}

        /**
         * Returns the fully rendered SQL query with parameters embedded as literals.
         *
         * Used for debugging or fallback rendering when the placeholder limit is exceeded.
         *
         * # Returns
         * A string representing the complete SQL statement.
         *
         * # Exceptions
         * - If rendering or encoding of parameters fails.
         */
        public function __toString(): string {}

        /**
         * Returns an array of all currently accumulated parameters.
         */
        public function parameters(): array {}

        /**
         * Appends a raw SQL fragment to the query without validation.
         *
         * # Arguments
         * * `part` - The SQL string to append.
         * * `parameters` - Optional parameters to associate with the fragment.
         */
        public function raw(string $part, ?array $parameters): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `SELECT` clause to the query.
         *
         * # Arguments
         * * `fields` - Either a raw string or a `SelectClauseRendered` object.
         *
         * # Exceptions
         * Throws an exception if the argument is not a string or a supported object.
         */
        public function select(mixed $fields): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `ORDER BY` clause to the query.
         *
         * # Arguments
         * * `fields` - Either a raw string or a `ByClauseRendered` object.
         *
         * # Exceptions
         * Throws an exception if the argument is not a string or a supported object.
         */
        public function orderBy(mixed $fields): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `GROUP BY` clause to the query.
         *
         * # Arguments
         * * `fields` - Either a raw string or a `ByClauseRendered` object.
         *
         * # Exceptions
         * Throws an exception if the argument is not a string or a supported object.
         */
        public function groupBy(mixed $fields): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `FOR UPDATE` locking clause to the query.
         *
         * This clause is used in `SELECT` statements to lock the selected rows
         * for update, preventing other transactions from modifying or acquiring
         * locks on them until the current transaction completes.
         *
         * # Example
         * ```php
         * $builder->select("*")->from("users")->for_update();
         * // SELECT * FROM users FOR UPDATE
         * ```
         *
         * # Notes
         * - Only valid in transactional contexts (e.g., PostgreSQL, MySQL with InnoDB).
         * - Useful for implementing pessimistic locking in concurrent systems.
         *
         * # Returns
         * The query builder with `FOR UPDATE` appended.
         */
        public function forUpdate(): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `FOR SHARE` locking clause to the query.
         *
         * This clause is used in `SELECT` statements to acquire shared locks
         * on the selected rows, allowing concurrent transactions to read but
         * not modify the rows until the current transaction completes.
         *
         * # Example
         * ```php
         * $builder->select("*")->from("documents")->for_share();
         * // SELECT * FROM documents FOR SHARE
         * ```
         *
         * # Notes
         * - Supported in PostgreSQL and some MySQL configurations.
         * - Ensures rows cannot be updated or deleted by other transactions while locked.
         *
         * # Returns
         * The query builder with `FOR SHARE` appended.
         */
        public function forShare(): \Sqlx\PgQueryBuilder {}

        /**
         * Appends an `INSERT INTO` clause to the query.
         *
         * # Arguments
         * * `table` - The name of the target table.
         *
         * # Example
         * ```php
         * $builder->insert("users");
         * ```
         */
        public function insert(string $table): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `VALUES` clause to the query.
         *
         * # Arguments
         * * `values` - Can be:
         *     - An associative array: `["name" => "John", "email" => "j@example.com"]`
         *     - A list of `[column, value]` pairs: `[["name", "John"], ["email", "j@example.com"]]`
         *     - A raw SQL string or a subquery builder
         *
         * # Example
         * ```php
         * $builder->insert("users")->values(["name" => "John", "email" => "j@example.com"]);
         * ```
         */
        public function values(mixed $values): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `TRUNCATE TABLE` statement to the query.
         *
         * This command removes all rows from the specified table quickly and efficiently.
         * It is faster than `DELETE FROM` and usually does not fire triggers or return affected row counts.
         *
         * # Arguments
         * * `table` – The name of the table to truncate.
         *
         * # Example
         * ```php
         * $builder->truncate_table("users");
         * // TRUNCATE TABLE users
         * ```
         *
         * # Notes
         * - May require elevated privileges depending on the database.
         * - This method can be chained with other query builder methods.
         *
         * # Errors
         * Returns an error if appending the SQL fragment fails (e.g., formatting error).
         */
        public function truncateTable(string $table): \Sqlx\PgQueryBuilder {}

        /**
         * Finalizes the query by appending a semicolon (`;`).
         *
         * This method is optional. Most databases do not require semicolons in prepared queries,
         * but you may use it to explicitly terminate a query string.
         *
         * # Example
         * ```php
         * $builder->select("*")->from("users")->end();
         * // SELECT * FROM users;
         * ```
         *
         * # Returns
         * The builder instance after appending the semicolon.
         *
         * # Notes
         * - Only appends the semicolon character; does not perform any execution.
         * - Useful when exporting a full query string with a terminating symbol (e.g., for SQL scripts).
         */
        public function end(string $_table): \Sqlx\PgQueryBuilder {}

        /**
         * Appends multiple rows to the `VALUES` clause of an `INSERT` statement.
         *
         * Each row must be:
         * - an ordered list of values (indexed array),
         * - or a map of column names to values (associative array) — only for the first row, to infer column order.
         *
         * # Arguments
         * * `rows` – A sequential array of rows (arrays of values).
         *
         * # Example
         * ```php
         * $builder->insert("users")->values_many([
         *     ["Alice", "alice@example.com"],
         *     ["Bob", "bob@example.com"]
         * ]);
         *
         * $builder->insert("users")->values_many([
         *     ["name" => "Alice", "email" => "alice@example.com"],
         *     ["name" => "Bob",   "email" => "bob@example.com"]
         * ]);
         * ```
         */
        public function valuesMany(mixed $rows): \Sqlx\PgQueryBuilder {}

        /**
         * Appends a `RETURNING` clause to the query.
         *
         * # Arguments
         * * `fields` - A string or array of column names to return.
         *
         * # Supported formats
         * ```php
         * $builder->returning("id");
         * $builder->returning(["id", "name"]);
         * ```
         *
         * # Notes
         * - This is mainly supported in PostgreSQL.
         * - Use with `INSERT`, `UPDATE`, or `DELETE`.
         */
        public function returning(mixed $fields): \Sqlx\PgQueryBuilder {}

        public function from(mixed $from, ?array $parameters): \Sqlx\PgQueryBuilder {}

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
         * The first column is used as the key (must be scalar),
         * and each resulting row is appended to the corresponding key's Vec.
         *
         * # Errors
         * Fails if the query fails, or the first column is not scalar.
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
         * - the key is the **first column** (must be scalar),
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

        public function __construct() {}
    }

    /**
     * This class supports prepared queries, persistent connections, and augmented SQL.
     */
    class MssqlDriver {
        /**
         * Creates a prepared query object with the given SQL string.
         *
         * # Arguments
         * - `query`: SQL query string to prepare
         *
         * # Returns
         * Prepared query object
         */
        public function prepare(string $query): \Sqlx\Driver\MssqlPreparedQuery {}

        /**
         * Creates a query builder object
         *
         *
         * # Returns
         * Query builder object
         */
        public function builder(): \Sqlx\MssqlQueryBuilder {}

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
         * # Arguments
         * * `param` – The parameter to quote (must be a scalar: string, number, or boolean).
         *
         * # Returns
         * Quoted SQL string (e.g., `'abc'`, `123`, `TRUE`)
         *
         * # Errors
         * Returns an error if the parameter is not a scalar or if rendering fails.
         *
         * # Example
         * ```php
         * $driver->builder()->quote("O'Reilly"); // "'O''Reilly'"
         * ```
         */
        public function quote(mixed $param): string {}

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
         * # Arguments
         * * `param` – The parameter to escape (must be a string).
         *
         * # Returns
         * A string with `%` and `_` escaped for LIKE (e.g., `foo\%bar\_baz`)
         *
         * # Errors
         * Returns an error if the input is not a string.
         *
         * # Example
         * ```php
         * $escaped = $builder->metaQuoteLike("100%_safe");
         * // Use like:
         * $builder->where([["name", "LIKE", "%$escaped%"]]);
         * ```
         */
        public function metaQuoteLike(mixed $param): string {}

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
         */
        public function queryGroupedColumnDictionary(string $query, ?array $parameters): mixed {}

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
         * Inserts a row into the given table using a map of columns.
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

        /**
         * Begins a new transaction, yields control to the provided callable,
         * and commits or rolls back based on the callable's return value or error.
         *
         * # Parameters
         * - `callable`: A PHP callable receiving this Driver instance.
         *
         * # Behavior
         * - Starts a transaction.
         * - Invokes `callable($this)`.
         * - If the callable returns false, rolls back, and commits otherwise.
         * - On exception or callable error, rolls back and rethrows.
         *
         * # Exceptions
         * Throws an exception if transaction commit, rollback,
         * or callable invocation fails.
         *
         */
        public function begin(callable $callable): mixed {}

        /**
         * Creates a transaction savepoint with the given name.
         *
         * # Parameters
         * - `savepoint`: Name of the savepoint to create.
         *
         * # Exceptions
         * Throws an exception if the driver fails to create the savepoint.
         */
        public function savepoint(string $savepoint): mixed {}

        /**
         * Rolls back the current transaction to a previously created savepoint.
         *
         * # Parameters
         * - `savepoint`: Name of the savepoint to rollback to.
         *
         * # Exceptions
         * Throws an exception if rollback to the savepoint fails.
         */
        public function rollbackToSavepoint(string $savepoint): mixed {}

        /**
         * Releases a previously created savepoint, making it no longer available.
         *
         * # Parameters
         * - `savepoint`: Name of the savepoint to release.
         *
         * # Exceptions
         * Throws an exception if releasing the savepoint fails.
         */
        public function releaseSavepoint(string $savepoint): mixed {}

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
    }

    /**
     * A prepared SQL query builder.
     *
     * Holds the generated query string, parameters, and placeholder tracking
     * used during safe, composable query construction via AST rendering.
     */
    class MssqlQueryBuilder {
        /**
         * Creates a query builder object
         *
         *
         * # Returns
         * Query builder object
         */
        public function builder(): \Sqlx\MssqlQueryBuilder {}

        public static function factory(\Sqlx\MssqlDriver $driver): \Sqlx\MssqlQueryBuilder {}

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
         * # Arguments
         * * `param` – The parameter to quote (must be a scalar: string, number, or boolean).
         *
         * # Returns
         * Quoted SQL string (e.g., `'abc'`, `123`, `TRUE`)
         *
         * # Errors
         * Returns an error if the parameter is not a scalar or if rendering fails.
         *
         * # Example
         * ```php
         * $driver->builder()->quote("O'Reilly"); // "'O''Reilly'"
         * ```
         */
        public function quote(mixed $param): string {}

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
         * # Arguments
         * * `param` – The parameter to escape (must be a string).
         *
         * # Returns
         * A string with `%` and `_` escaped for LIKE (e.g., `foo\%bar\_baz`)
         *
         * # Errors
         * Returns an error if the input is not a string.
         *
         * # Example
         * ```php
         * $escaped = $builder->metaQuoteLike("100%_safe");
         * // Use like:
         * $builder->where([["name", "LIKE", "%$escaped%"]]);
         * ```
         */
        public function metaQuoteLike(mixed $param): string {}

        /**
         * Appends an `ON CONFLICT` clause to the query.
         *
         * # Arguments
         * * `target` – A string or array of column names to specify the conflict target.
         * * `set` – Optional `SET` clause. If `null`, generates `DO NOTHING`; otherwise uses the `SET` values.
         *
         * # Example
         * ```php
         * $builder->onConflict("id", null);
         * $builder->onConflict(["email", "tenant_id"], ["name" => "New"]);
         * ```
         */
        public function onConflict(mixed $target, ?mixed $set): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends an `ON DUPLICATE KEY UPDATE` clause to the query (MySQL).
         *
         * # Arguments
         * * `set` – An array representing fields and values to update.
         *
         * # Example
         * ```php
         * $builder->onDuplicateKeyUpdate(["email" => "new@example.com"]);
         * ```
         */
        public function onDuplicateKeyUpdate(mixed $set): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends an `INNER JOIN` clause to the query.
         */
        public function innerJoin(string $table, string $on, ?array $parameters): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends an `INNER JOIN` clause to the query.
         * Alias for `inner_join()`.
         */
        public function join(string $table, string $on, ?array $parameters): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `LEFT JOIN` clause to the query.
         */
        public function leftJoin(string $table, string $on, ?array $parameters): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `RIGHT JOIN` clause to the query.
         */
        public function rightJoin(string $table, string $on, ?array $parameters): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `FULL OUTER JOIN` clause to the query.
         */
        public function fullJoin(string $table, string $on, ?array $parameters): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `CROSS JOIN` clause to the query.
         */
        public function crossJoin(string $table, string $on, ?array $parameters): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `NATURAL JOIN` clause to the query.
         *
         * # Note
         * `NATURAL JOIN` does not accept `ON` conditions. The `on` argument is ignored.
         */
        public function naturalJoin(string $table): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `WITH` clause to the query.
         *
         * # Arguments
         * * `table` - Name of the CTE (common table expression).
         * * `as` - The query body to be rendered for the CTE.
         * * `parameters` - Optional parameters for the query body.
         *
         * # Example
         * ```php
         * $builder->with("cte_name", "SELECT * FROM users WHERE active = $active", [
         *     "active" => true,
         * ]);
         * ```
         */
        public function with(string $table, mixed $r#as, ?array $parameters): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `WHERE` clause to the query.
         *
         * # Arguments
         * * `where` - Either a raw string, a structured array of conditions, or a disjunction (`OrClause`).
         * * `parameters` - Optional parameters associated with the `WHERE` condition.
         *
         * # Exceptions
         * Throws an exception if the input is not valid.
         */
        public function where(mixed $r#where, ?array $parameters): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `UNION` clause to the query.
         *
         * # Arguments
         * * `query` – A raw SQL string or another Builder instance (subquery).
         * * `parameters` – Optional parameters to bind to the unioned subquery.
         *
         * # Example
         * ```php
         * $builder->union("SELECT id FROM users");
         * $builder->union($other_builder);
         * ```
         */
        public function union(mixed $query, ?array $parameters): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `UNION ALL` clause to the query.
         *
         * # Arguments
         * * `query` – A raw SQL string or another Builder instance (subquery).
         * * `parameters` – Optional parameters to bind to the unioned subquery.
         *
         * # Example
         * ```php
         * $builder->union_all("SELECT id FROM users");
         * $builder->union_all($other_builder);
         * ```
         */
        public function unionAll(mixed $query, ?array $parameters): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `HAVING` clause to the query.
         *
         * # Arguments
         * * `having` - Either a raw string, a structured array of conditions, or a disjunction (`OrClause`).
         * * `parameters` - Optional parameters associated with the `WHERE` condition.
         *
         * # Exceptions
         * Throws an exception if the input is not valid.
         */
        public function having(mixed $having, ?array $parameters): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `LIMIT` (and optional `OFFSET`) clause to the query.
         *
         * # Arguments
         * * `limit` – Maximum number of rows to return.
         * * `offset` – Optional number of rows to skip before starting to return rows.
         *
         * # Example
         * ```php
         * $builder->limit(10);
         * $builder->limit(10, 20); // LIMIT 10 OFFSET 20
         * ```
         */
        public function limit(int $limit, ?int $offset): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends an `OFFSET` clause to the query independently.
         *
         * # Arguments
         * * `offset` – Number of rows to skip before returning results.
         *
         * # Example
         * ```php
         * $builder->offset(30); // OFFSET 30
         * ```
         */
        public function offset(int $offset): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `DELETE FROM` clause to the query.
         *
         * # Arguments
         * * `from` - A string table name or a nested builder object.
         * * `parameters` - Optional parameters if the `from` is a raw string.
         *
         * # Examples
         * ```php
         * $builder->deleteFrom("users");
         * $builder->deleteFrom($builder->select("id")->from("temp_users"));
         * ```
         */
        public function deleteFrom(mixed $from, ?array $parameters): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `USING` clause to the query.
         *
         * # Arguments
         * * `from` - Either a string table name or a subquery builder.
         * * `parameters` - Optional parameters if `from` is a string.
         */
        public function using(mixed $from, ?array $parameters): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `PAGINATE` clause to the query using a `PaginateClauseRendered` object.
         *
         * This expands into the appropriate SQL `LIMIT` and `OFFSET` syntax during rendering,
         * using the values stored in the given `PaginateClauseRendered` instance.
         *
         * # Arguments
         * * `paginate` – An instance of `Sqlx\PaginateClauseRendered`, produced by invoking a `PaginateClause`
         *               (e.g., `$paginate = (new PaginateClause)($page, $perPage)`).
         *
         * # Errors
         * Returns an error if the argument is not an instance of `PaginateClauseRendered`.
         *
         * # Example
         * ```php
         * $paginate = (new \Sqlx\PaginateClause())->__invoke(2, 10); // page 2, 10 per page
         * $builder->select("*")->from("users")->paginate($paginate);
         * // SELECT * FROM users LIMIT 10 OFFSET 20
         * ```
         */
        public function paginate(mixed $paginate): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `WITH RECURSIVE` clause to the query.
         *
         * # Arguments
         * * `table_and_fields` - Table name with field list, e.g. `cte(col1, col2)`.
         * * `as` - The recursive query body.
         * * `parameters` - Optional parameters for the recursive body.
         *
         * # Example
         * ```php
         * $builder->withRecursive("cte(id, parent)", "SELECT ...", [...]);
         * ```
         */
        public function withRecursive(string $table_and_fields, mixed $r#as, ?array $parameters): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `UPDATE` clause to the query.
         *
         * # Arguments
         * * `table` - A raw string representing the table(s).
         *
         * # Exceptions
         * Throws an exception if the argument is not a string.
         */
        public function update(mixed $table): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `SET` clause to the query, supporting both keyed and indexed formats.
         *
         * # Arguments
         * * `set` - An associative array mapping fields to values, or a sequential array
         *   of `[field, value]` pairs or raw fragments.
         *
         * # Supported Formats
         * ```php
         * $builder->set(["name" => "John", "age" => 30]);
         * $builder->set([["name", "John"], ["age", 30]]);
         * $builder->set(["updated_at = NOW()"]);
         * ```
         *
         * # Exceptions
         * - When the input array is malformed or contains invalid value types.
         */
        public function set(mixed $set): \Sqlx\MssqlQueryBuilder {}

        /**
         * Renders the SQL query using named parameters and returns the fully rendered SQL with values injected inline.
         *
         * # Returns
         * The rendered SQL query as a string with all parameters interpolated.
         *
         * # Exceptions
         * - If rendering or encoding of parameters fails.
         */
        public function dry(): array {}

        /**
         * Returns the parameter map currently accumulated in the builder.
         *
         * # Returns
         * A cloned map of all parameter names and their corresponding `ParameterValue`.
         *
         * # Exceptions
         * - If rendering or encoding of parameters fails.
         */
        public function dryInline(): string {}

        /**
         * Returns the fully rendered SQL query with parameters embedded as literals.
         *
         * Used for debugging or fallback rendering when the placeholder limit is exceeded.
         *
         * # Returns
         * A string representing the complete SQL statement.
         *
         * # Exceptions
         * - If rendering or encoding of parameters fails.
         */
        public function __toString(): string {}

        /**
         * Returns an array of all currently accumulated parameters.
         */
        public function parameters(): array {}

        /**
         * Appends a raw SQL fragment to the query without validation.
         *
         * # Arguments
         * * `part` - The SQL string to append.
         * * `parameters` - Optional parameters to associate with the fragment.
         */
        public function raw(string $part, ?array $parameters): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `SELECT` clause to the query.
         *
         * # Arguments
         * * `fields` - Either a raw string or a `SelectClauseRendered` object.
         *
         * # Exceptions
         * Throws an exception if the argument is not a string or a supported object.
         */
        public function select(mixed $fields): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `ORDER BY` clause to the query.
         *
         * # Arguments
         * * `fields` - Either a raw string or a `ByClauseRendered` object.
         *
         * # Exceptions
         * Throws an exception if the argument is not a string or a supported object.
         */
        public function orderBy(mixed $fields): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `GROUP BY` clause to the query.
         *
         * # Arguments
         * * `fields` - Either a raw string or a `ByClauseRendered` object.
         *
         * # Exceptions
         * Throws an exception if the argument is not a string or a supported object.
         */
        public function groupBy(mixed $fields): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `FOR UPDATE` locking clause to the query.
         *
         * This clause is used in `SELECT` statements to lock the selected rows
         * for update, preventing other transactions from modifying or acquiring
         * locks on them until the current transaction completes.
         *
         * # Example
         * ```php
         * $builder->select("*")->from("users")->for_update();
         * // SELECT * FROM users FOR UPDATE
         * ```
         *
         * # Notes
         * - Only valid in transactional contexts (e.g., PostgreSQL, MySQL with InnoDB).
         * - Useful for implementing pessimistic locking in concurrent systems.
         *
         * # Returns
         * The query builder with `FOR UPDATE` appended.
         */
        public function forUpdate(): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `FOR SHARE` locking clause to the query.
         *
         * This clause is used in `SELECT` statements to acquire shared locks
         * on the selected rows, allowing concurrent transactions to read but
         * not modify the rows until the current transaction completes.
         *
         * # Example
         * ```php
         * $builder->select("*")->from("documents")->for_share();
         * // SELECT * FROM documents FOR SHARE
         * ```
         *
         * # Notes
         * - Supported in PostgreSQL and some MySQL configurations.
         * - Ensures rows cannot be updated or deleted by other transactions while locked.
         *
         * # Returns
         * The query builder with `FOR SHARE` appended.
         */
        public function forShare(): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends an `INSERT INTO` clause to the query.
         *
         * # Arguments
         * * `table` - The name of the target table.
         *
         * # Example
         * ```php
         * $builder->insert("users");
         * ```
         */
        public function insert(string $table): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `VALUES` clause to the query.
         *
         * # Arguments
         * * `values` - Can be:
         *     - An associative array: `["name" => "John", "email" => "j@example.com"]`
         *     - A list of `[column, value]` pairs: `[["name", "John"], ["email", "j@example.com"]]`
         *     - A raw SQL string or a subquery builder
         *
         * # Example
         * ```php
         * $builder->insert("users")->values(["name" => "John", "email" => "j@example.com"]);
         * ```
         */
        public function values(mixed $values): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `TRUNCATE TABLE` statement to the query.
         *
         * This command removes all rows from the specified table quickly and efficiently.
         * It is faster than `DELETE FROM` and usually does not fire triggers or return affected row counts.
         *
         * # Arguments
         * * `table` – The name of the table to truncate.
         *
         * # Example
         * ```php
         * $builder->truncate_table("users");
         * // TRUNCATE TABLE users
         * ```
         *
         * # Notes
         * - May require elevated privileges depending on the database.
         * - This method can be chained with other query builder methods.
         *
         * # Errors
         * Returns an error if appending the SQL fragment fails (e.g., formatting error).
         */
        public function truncateTable(string $table): \Sqlx\MssqlQueryBuilder {}

        /**
         * Finalizes the query by appending a semicolon (`;`).
         *
         * This method is optional. Most databases do not require semicolons in prepared queries,
         * but you may use it to explicitly terminate a query string.
         *
         * # Example
         * ```php
         * $builder->select("*")->from("users")->end();
         * // SELECT * FROM users;
         * ```
         *
         * # Returns
         * The builder instance after appending the semicolon.
         *
         * # Notes
         * - Only appends the semicolon character; does not perform any execution.
         * - Useful when exporting a full query string with a terminating symbol (e.g., for SQL scripts).
         */
        public function end(string $_table): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends multiple rows to the `VALUES` clause of an `INSERT` statement.
         *
         * Each row must be:
         * - an ordered list of values (indexed array),
         * - or a map of column names to values (associative array) — only for the first row, to infer column order.
         *
         * # Arguments
         * * `rows` – A sequential array of rows (arrays of values).
         *
         * # Example
         * ```php
         * $builder->insert("users")->values_many([
         *     ["Alice", "alice@example.com"],
         *     ["Bob", "bob@example.com"]
         * ]);
         *
         * $builder->insert("users")->values_many([
         *     ["name" => "Alice", "email" => "alice@example.com"],
         *     ["name" => "Bob",   "email" => "bob@example.com"]
         * ]);
         * ```
         */
        public function valuesMany(mixed $rows): \Sqlx\MssqlQueryBuilder {}

        /**
         * Appends a `RETURNING` clause to the query.
         *
         * # Arguments
         * * `fields` - A string or array of column names to return.
         *
         * # Supported formats
         * ```php
         * $builder->returning("id");
         * $builder->returning(["id", "name"]);
         * ```
         *
         * # Notes
         * - This is mainly supported in PostgreSQL.
         * - Use with `INSERT`, `UPDATE`, or `DELETE`.
         */
        public function returning(mixed $fields): \Sqlx\MssqlQueryBuilder {}

        public function from(mixed $from, ?array $parameters): \Sqlx\MssqlQueryBuilder {}

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
         * The first column is used as the key (must be scalar),
         * and each resulting row is appended to the corresponding key's Vec.
         *
         * # Errors
         * Fails if the query fails, or the first column is not scalar.
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
         * - the key is the **first column** (must be scalar),
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

        public function __construct() {}
    }

    /**
     * A PHP-accessible wrapper around a `zend_array` that lazily decodes JSON values.
     *
     * Implements `ArrayAccess` so that columns can be accessed as array entries.
     */
    class LazyRow implements \ArrayAccess {
        /**
         * Checks whether a column exists in the row.
         *
         * # Arguments
         *
         * * `offset` – The column name as a `Zval` (expected to be a string).
         *
         * # Returns
         *
         * `Ok(true)` if the column exists, `Ok(false)` otherwise, or an error if the offset isn't a string.
         */
        public function offsetExists(mixed $offset): bool {}

        /**
         * Magic getter for property access in PHP (`$row->column`).
         *
         * Lazily decodes JSON-wrapped values if needed and replaces the placeholder object
         * with the actual decoded `Zval`.
         *
         * # Arguments
         *
         * * `name` – The column name.
         *
         * # Errors
         *
         * Returns a `PhpException` if the column is not found or offset is not a string.
         */
        public function __get(string $name): mixed {}

        /**
         * ArrayAccess getter (`$row[$column]`).
         *
         * Performs the same lazy JSON decoding logic as `__get`.
         */
        public function offsetGet(mixed $offset): mixed {}

        /**
         * ArrayAccess setter (`$row[$key] = $value`).
         *
         * Inserts or updates the given key with the provided `Zval`.
         *
         * # Exceptions
         *
         * Throws an exception if insertion fails or if the offset isn't a string.
         */
        public function offsetSet(mixed $offset, mixed $value): mixed {}

        /**
         * ArrayAccess unsetter (`unset($row[$key])`).
         *
         * Unsetting values is not supported and always returns an error.
         */
        public function offsetUnset(mixed $_offset): mixed {}

        public function __construct() {}
    }

    /**
     * A helper PHP class that holds raw JSON bytes for lazy decoding.
     *
     * When accessed, it will be parsed into a PHP value on demand.
     */
    class LazyRowJson {
        /**
         * Decode the stored JSON into a PHP `Zval`.
         *
         * Uses either `simd-json` or `serde_json` depending on build features.
         *
         * # Errors
         *
         * Propagates JSON parsing exceptions.
         */
        public function takeZval(): mixed {}

        public function __construct() {}
    }

    class DriverOptions {
        const OPT_URL = null;

        const OPT_AST_CACHE_SHARD_COUNT = null;

        const OPT_AST_CACHE_SHARD_SIZE = null;

        const OPT_PERSISTENT_NAME = null;

        const OPT_ASSOC_ARRAYS = null;

        const OPT_MAX_CONNECTIONS = null;

        const OPT_MIN_CONNECTIONS = null;

        const OPT_COLLAPSIBLE_IN = null;

        const OPT_MAX_LIFETIME = null;

        const OPT_IDLE_TIMEOUT = null;

        const OPT_TEST_BEFORE_ACQUIRE = null;

        public function __construct() {}
    }
}

namespace Sqlx\Driver {
    /**
     * A reusable prepared SQL query with parameter support. Created using `PgDriver::prepare()`, shares context with original driver.
     */
    class MssqlPreparedQuery {
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
         * The first column is used as the key (must be scalar),
         * and each resulting row is appended to the corresponding key's Vec.
         *
         * # Errors
         * Fails if the query fails, or the first column is not scalar.
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
         * - the key is the **first column** (must be scalar),
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

        public function __construct() {}
    }
}
