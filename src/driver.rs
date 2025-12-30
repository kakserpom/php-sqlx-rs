//! Database driver implementation macro for php-sqlx.
//!
//! This module provides the [`php_sqlx_impl_driver!`] macro that generates complete
//! database driver implementations with PHP bindings. Each database type (`PostgreSQL`,
//! `MySQL`, MS SQL) uses this macro to create a driver with consistent API.
//!
//! # Generated API
//!
//! The macro generates:
//! - Driver struct with connection pooling and persistent connection support
//! - Prepared query support with AST caching
//! - Query builders (read and write)
//! - Full set of query methods (`query_row`, `query_all`, `query_column`, etc.)
//! - Transaction support (both callback and imperative styles)
//! - Savepoint management
//!
//! # Usage
//!
//! ```rust,ignore
//! php_sqlx_impl_driver!(
//!     PgDriver,           // Struct name
//!     "Sqlx\\PgDriver",   // PHP class name
//!     PgInnerDriver,      // Inner driver type
//!     PgPreparedQuery,    // Prepared query type
//!     PgReadQueryBuilder, // Read builder type
//!     PgWriteQueryBuilder // Write builder type
//! );
//! ```

/// Generates a complete database driver implementation with PHP bindings.
///
/// This macro creates a driver struct, PHP class bindings, and all query methods
/// for a specific database backend. It reduces boilerplate by providing a consistent
/// API across `PostgreSQL`, `MySQL`, and MS SQL drivers.
///
/// # Arguments
///
/// - `$struct` - The Rust struct name for the driver (e.g., `PgDriver`)
/// - `$class` - The PHP class name as a string literal (e.g., `"Sqlx\\PgDriver"`)
/// - `$inner` - The inner driver type that implements database-specific logic
/// - `$prepared_query` - The prepared query type for this database
/// - `$read_query_builder` - The read query builder type
/// - `$write_query_builder` - The write query builder type
#[macro_export]
macro_rules! php_sqlx_impl_driver {
    ( $struct:ident, $class:literal, $inner:ident, $prepared_query:ident, $read_query_builder:ident, $write_query_builder:ident $(,)? ) => {
        mod conversion;
        pub mod prepared_query;
        pub mod read_query_builder;
        pub mod write_query_builder;
        use $crate::error::Error as SqlxError;
        use dashmap::DashMap;
        use ext_php_rs::builders::ModuleBuilder;
        use ext_php_rs::prelude::*;
        use ext_php_rs::types::Zval;
        use ext_php_rs::{php_class, php_impl};
        use inner::$inner;
        use itertools::Itertools;
        pub use prepared_query::$prepared_query;
        use read_query_builder::$read_query_builder;
        use std::collections::BTreeMap;
        use std::sync::{Arc, LazyLock, Once};
        use write_query_builder::$write_query_builder;
        use $crate::ast::UpsertStyle;
        use $crate::options::DriverInnerOptions;
        use $crate::options::DriverOptionsArg;
        use $crate::param_value::ParameterValue;
        use $crate::utils::types::ColumnArgument;
        pub mod inner;

        /// Global registry for persistent driver connections.
        ///
        /// This allows named connections to be reused across PHP requests when
        /// `OPT_PERSISTENT_NAME` is specified. The registry maps connection names
        /// to shared inner driver instances.
        static PERSISTENT_DRIVER_REGISTRY: LazyLock<DashMap<String, Arc<$inner>>> =
            LazyLock::new(|| DashMap::new());

        /// Database driver for executing SQL queries with advanced features.
        ///
        /// This class supports:
        /// - **Prepared queries**: Cached AST parsing for repeated queries
        /// - **Persistent connections**: Reuse connections across PHP requests
        /// - **Augmented SQL**: Conditional blocks, IN clause optimization, pagination
        /// - **Transactions**: Both callback-based and imperative styles
        /// - **Query builders**: Fluent API for constructing queries
        #[php_class]
        #[php(name = $class)]
        #[derive(Clone)]
        pub struct $struct {
            /// The inner driver instance containing connection pool and configuration.
            pub driver_inner: Arc<$inner>,
        }

        /// Registers driver classes with the PHP module builder.
        pub fn build(module: ModuleBuilder) -> ModuleBuilder {
            module
                .class::<$struct>()
                .class::<$prepared_query>()
                .class::<$read_query_builder>()
                .class::<$write_query_builder>()
        }

        impl $struct {
            /// Creates a new driver instance with the given options.
            ///
            /// If `persistent_name` is set in options, checks the global registry
            /// for an existing connection and reuses it. Otherwise, creates a new
            /// connection pool and optionally registers it for persistence.
            pub fn new(options: DriverInnerOptions) -> $crate::error::Result<Self> {
                static INIT: Once = Once::new();
                INIT.call_once(|| {
                    $crate::utils::adhoc_php_class_implements($class, "Sqlx\\DriverInterface");
                });

                if let Some(name) = options.persistent_name.as_ref() {
                    if let Some(driver_inner) = PERSISTENT_DRIVER_REGISTRY.get(name) {
                        return Ok(Self {
                            driver_inner: driver_inner.clone(),
                        });
                    }
                }
                let persistent_name = options.persistent_name.clone();
                let driver_inner = Arc::new(<$inner>::new(options)?);
                if let Some(name) = persistent_name {
                    PERSISTENT_DRIVER_REGISTRY.insert(name, driver_inner.clone());
                }
                Ok(Self { driver_inner })
            }
        }

        #[php_impl]
        impl $struct {
            /// Constructs a new `SQLx` driver instance.
            ///
            /// # Arguments
            /// - `options`: Connection URL as string or associative array with options:
            ///   - `url`: (string) database connection string (required)
            ///   - `ast_cache_shard_count`: (int) number of AST cache shards (default: 8)
            ///   - `ast_cache_shard_size`: (int) size per shard (default: 256)
            ///   - `persistent_name`: (string) name of persistent connection
            ///   - `assoc_arrays`: (bool) return associative arrays instead of objects
            pub fn __construct(url_or_options: DriverOptionsArg) -> $crate::error::Result<Self> {
                Self::new(url_or_options.parse()?)
            }

            /// Creates a prepared query object with the given SQL string.
            ///
            /// # Arguments
            /// - `query`: SQL query string to prepare
            ///
            /// # Returns
            /// Prepared query object
            #[must_use]
            pub fn prepare(&self, query: &str) -> $prepared_query {
                $prepared_query::new(query, self.driver_inner.clone())
            }

            /// Creates a query builder object
            ///
            ///
            /// # Returns
            /// Query builder object
            #[must_use]
            pub fn builder(&self) -> $write_query_builder {
                $write_query_builder::new(self.driver_inner.clone())
            }

            /// Creates a query builder object
            ///
            ///
            /// # Returns
            /// Query builder object
            #[must_use]
            pub fn read_builder(&self) -> $read_query_builder {
                $read_query_builder::new(self.driver_inner.clone())
            }

            /// Quotes a single scalar value for safe embedding into SQL.
            ///
            /// This method renders the given `ParameterValue` into a properly escaped SQL literal,
            /// using the driver's configuration (e.g., quoting style, encoding).
            ///
            /// ⚠️ **Warning:** Prefer using placeholders and parameter binding wherever possible.
            /// This method should only be used for debugging or generating static fragments,
            /// not for constructing dynamic SQL with user input.
            ///
            /// # Arguments
            /// * `param` – The parameter to quote (must be a scalar: string, number, or boolean).
            ///
            /// # Returns
            /// Quoted SQL string (e.g., `'abc'`, `123`, `TRUE`)
            ///
            /// # Errors
            /// Returns an error if the parameter is not a scalar or if rendering fails.
            ///
            /// # Example
            /// ```php
            /// $driver->builder()->quote("O'Reilly"); // "'O''Reilly'"
            /// ```
            pub fn quote(&self, param: ParameterValue) -> $crate::error::Result<String> {
                param.quote(&self.driver_inner.settings)
            }

            /// Quotes a string for use in a LIKE/ILIKE pattern, escaping both SQL special
            /// characters (like single quotes) and LIKE metacharacters (`%` and `_`).
            ///
            /// This is like `quote()` but additionally escapes `%` and `_` so they match literally.
            ///
            /// # Arguments
            /// * `param` – The parameter to quote (must be a string).
            ///
            /// # Returns
            /// A fully quoted SQL string with LIKE metacharacters escaped.
            ///
            /// # Errors
            /// Returns an error if the input is not a string.
            ///
            /// # Example
            /// ```php
            /// $quoted = $driver->quoteLike("O'Reilly%");
            /// // Returns: 'O''Reilly\%'
            /// ```
            pub fn quote_like(&self, param: ParameterValue) -> $crate::error::Result<String> {
                param.quote_like(&self.driver_inner.settings)
            }

            /// Quotes an identifier (table name, column name) using the appropriate quote style.
            ///
            /// - PostgreSQL: `"identifier"` (double quotes)
            /// - MySQL: `` `identifier` `` (backticks)
            /// - MSSQL: `[identifier]` (brackets)
            ///
            /// Special characters in the identifier are properly escaped.
            ///
            /// # Arguments
            /// * `name` – The identifier to quote.
            ///
            /// # Returns
            /// The quoted identifier string.
            ///
            /// # Example
            /// ```php
            /// // PostgreSQL
            /// $driver->quoteIdentifier("user"); // Returns: "user"
            /// $driver->quoteIdentifier('my"column'); // Returns: "my""column"
            ///
            /// // MySQL
            /// $driver->quoteIdentifier("user"); // Returns: `user`
            /// ```
            pub fn quote_identifier(&self, name: &str) -> String {
                $crate::param_value::quote::quote_identifier(name, &self.driver_inner.settings)
            }

            /// Returns whether results are returned as associative arrays.
            ///
            /// If true, result rows are returned as PHP associative arrays (key-value pairs).
            /// If false, result rows are returned as PHP `stdClass` objects.
            #[must_use]
            pub fn assoc_arrays(&self) -> bool {
                self.driver_inner.options.associative_arrays
            }

            /// Executes an SQL query and returns a single result.
            ///
            /// # Arguments
            /// - `query`: SQL query string
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Returns
            /// Single row as array or object depending on config
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - SQL query is invalid or execution fails;
            /// - a parameter cannot be bound or has incorrect type;
            /// - the row contains unsupported database types;
            /// - conversion to PHP object fails.
            pub fn query_row(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner.query_row(query, parameters, None)
            }

            /// Executes an SQL query and returns a single column value from the first row.
            ///
            /// # Arguments
            /// - `query`: SQL query string to execute.
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            /// - `column`: Optional column name or index to extract from the result row. Defaults to the first column.
            ///
            /// # Returns
            /// The value from the specified column of the first row.
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - the query is invalid or fails to execute;
            /// - the specified column does not exist;
            /// - the value cannot be converted to a PHP-compatible type.
            pub fn query_value(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
                column: Option<ColumnArgument>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_value(query, parameters, column, None)
            }

            pub fn query_value_assoc(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
                column: Option<ColumnArgument>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_value(query, parameters, column, Some(true))
            }

            /**
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
            pub fn query_value_obj(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
                column: Option<ColumnArgument>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_value(query, parameters, column, Some(false))
            }

            /// Executes an SQL query and returns a single column value from the first row, or null if no rows matched.
            ///
            /// # Arguments
            /// - `query`: SQL query string to execute.
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            /// - `column`: Optional column name or index to extract from the result row.
            ///
            /// # Returns
            /// The value from the specified column of the first row as a PHP value, or `null` if no row was found.
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - the query is invalid or fails to execute;
            /// - the specified column does not exist;
            /// - the value cannot be converted to a PHP-compatible type.
            pub fn query_maybe_value(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
                column: Option<ColumnArgument>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_maybe_value(query, parameters, column, None)
            }

            /// Executes an SQL query and returns a single column value as a PHP value (array mode), or null if no row matched.
            ///
            /// Same as `query_maybe_value`, but forces associative array mode for complex values.
            ///
            /// # Arguments
            /// - `query`: SQL query string to execute.
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            /// - `column`: Optional column name or index to extract.
            ///
            /// # Returns
            /// The column value from the first row, or `null` if no row found.
            ///
            /// # Exceptions
            /// Same as `query_maybe_value`.
            pub fn query_maybe_value_assoc(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
                column: Option<ColumnArgument>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_maybe_value(query, parameters, column, Some(true))
            }

            /// Executes an SQL query and returns a single column value as a PHP object, or null if no row matched.
            ///
            /// Same as `query_maybe_value`, but forces object mode for complex values.
            ///
            /// # Arguments
            /// - `query`: SQL query string to execute.
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            /// - `column`: Optional column name or index to extract.
            ///
            /// # Returns
            /// The column value from the first row, or `null` if no row found.
            ///
            /// # Exceptions
            /// Same as `query_maybe_value`.
            pub fn query_maybe_value_obj(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
                column: Option<ColumnArgument>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_maybe_value(query, parameters, column, Some(false))
            }

            /// Executes an SQL query and returns one row as an associative array.
            ///
            /// # Arguments
            /// - `query`: SQL query string
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - SQL query is invalid or execution fails;
            /// - a parameter cannot be bound or has incorrect type;
            /// - the row contains unsupported database types;
            /// - conversion to PHP object fails.
            pub fn query_row_assoc(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner.query_row(query, parameters, Some(true))
            }

            /// Executes an SQL query and returns one row as an object.
            ///
            /// # Arguments
            /// - `query`: SQL query string
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - SQL query is invalid or execution fails;
            /// - a parameter cannot be bound or has incorrect type;
            /// - the row contains unsupported database types;
            /// - conversion to PHP object fails.
            pub fn query_row_obj(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner.query_row(query, parameters, Some(false))
            }

            /// Executes an SQL query and returns a single result, or `null` if no row matched.
            ///
            /// # Arguments
            /// - `query`: SQL query string
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Returns
            /// Single row as array or object depending on config
            ///
            /// # Exceptions
            /// Throws an exception if the query fails for reasons other than no matching rows.
            /// For example, syntax errors, type mismatches, or database connection issues.
            pub fn query_maybe_row(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner.query_maybe_row(query, parameters, None)
            }

            /// Executes an SQL query and returns a single row as a PHP associative array, or `null` if no row matched.
            ///
            /// # Arguments
            /// - `query`: SQL query string to execute.
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Returns
            /// The result row as an associative array (`array<string, mixed>` in PHP), or `null` if no matching row is found.
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - the query is invalid or fails to execute;
            /// - parameters are invalid or cannot be bound;
            /// - the row contains unsupported or unconvertible data types.
            pub fn query_maybe_row_assoc(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_maybe_row(query, parameters, Some(true))
            }

            /// Executes an SQL query and returns a single row as a PHP object, or `null` if no row matched.
            ///
            /// # Arguments
            /// - `query`: SQL query string to execute.
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Returns
            /// The result row as a `stdClass` PHP object, or `null` if no matching row is found.
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - the query is invalid or fails to execute;
            /// - parameters are invalid or cannot be bound;
            /// - the row contains unsupported or unconvertible data types.
            pub fn query_maybe_row_obj(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_maybe_row(query, parameters, Some(false))
            }

            /// Executes an SQL query and returns the specified column values from all result rows.
            ///
            /// # Arguments
            /// - `query`: SQL query string to execute.
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            /// - `column`: Optional column name or index to extract.
            ///
            /// # Returns
            /// An array of column values, one for each row.
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - the query fails to execute;
            /// - the specified column is not found;
            /// - a column value cannot be converted to PHP.
            pub fn query_column(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
                column: Option<ColumnArgument>,
            ) -> $crate::error::Result<Vec<Zval>> {
                self.driver_inner
                    .query_column(query, parameters, column, None)
            }

            /// Executes an SQL query and returns the specified column values from all rows in associative array mode.
            ///
            /// # Arguments
            /// - `query`: SQL query string.
            /// - `parameters`: Optional named parameters.
            /// - `column`: Column index or name to extract.
            ///
            /// # Returns
            /// An array of column values (associative arrays for structured data).
            ///
            /// # Exceptions
            /// Same as `query_column`.
            pub fn query_column_assoc(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
                column: Option<ColumnArgument>,
            ) -> $crate::error::Result<Vec<Zval>> {
                self.driver_inner
                    .query_column(query, parameters, column, Some(true))
            }

            /// Executes an SQL query and returns the specified column values from all rows in object mode.
            ///
            /// # Arguments
            /// - `query`: SQL query string.
            /// - `parameters`: Optional named parameters.
            /// - `column`: Column index or name to extract.
            ///
            /// # Returns
            /// An array of column values (objects for structured data).
            ///
            /// # Exceptions
            /// Same as `query_column`.
            pub fn query_column_obj(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
                column: Option<ColumnArgument>,
            ) -> $crate::error::Result<Vec<Zval>> {
                self.driver_inner
                    .query_column(query, parameters, column, Some(false))
            }

            /// Executes an SQL query and returns all results.
            ///
            /// # Arguments
            /// - `query`: SQL query string
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Returns
            /// Array of rows as array or object depending on config
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - SQL query is invalid or fails to execute;
            /// - parameter binding fails;
            /// - row decoding fails due to an unsupported or mismatched database type;
            /// - conversion to PHP values fails (e.g., due to memory or encoding issues).
            pub fn query_all(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Vec<Zval>> {
                self.driver_inner.query_all(query, parameters, None)
            }

            /// Executes an SQL query and returns all rows as associative arrays.
            ///
            /// # Arguments
            /// - `query`: SQL query string
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - SQL query is invalid or fails to execute;
            /// - parameter binding fails;
            /// - row decoding fails due to an unsupported or mismatched database type;
            /// - conversion to PHP values fails (e.g., due to memory or encoding issues).
            pub fn query_all_assoc(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Vec<Zval>> {
                self.driver_inner.query_all(query, parameters, Some(true))
            }

            /// Executes an SQL query and returns all rows as objects.
            ///
            /// # Arguments
            /// - `query`: SQL query string
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - SQL query is invalid or fails to execute;
            /// - parameter binding fails;
            /// - row decoding fails due to an unsupported or mismatched database type;
            /// - conversion to PHP values fails (e.g., due to memory or encoding issues).
            pub fn query_all_obj(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Vec<Zval>> {
                self.driver_inner.query_all(query, parameters, Some(false))
            }

            /// Executes an SQL query and returns a dictionary (map) indexed by the first column of each row.
            ///
            /// # Description
            /// The result is a `HashMap` where each key is the string value of the first column in a row,
            /// and the corresponding value is the row itself (as an array or object depending on config).
            ///
            /// This variant respects the global `assoc_arrays` setting to determine the row format.
            ///
            /// # Parameters
            /// - `query`: SQL query string to execute.
            /// - `parameters`: Optional map of named parameters to bind.
            ///
            /// # Returns
            /// A map from the first column (as string) to the full row.
            ///
            /// # Errors
            /// - If the query fails to execute.
            /// - If the first column cannot be converted to a string.
            /// - If row decoding or PHP conversion fails.
            ///
            /// # Notes
            /// - The iteration order of the returned map is **not** guaranteed.
            pub fn query_dictionary(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner.query_dictionary(query, parameters, None)
            }

            /// Executes an SQL query and returns a dictionary (map) indexed by the first column of each row,
            /// returning each row as an associative array.
            ///
            /// # Parameters
            /// - `query`: SQL query string to execute.
            /// - `parameters`: Optional map of named parameters to bind.
            ///
            /// # Returns
            /// A map from the first column (as string) to the full row (as an associative array).
            ///
            /// # Errors
            /// - If the query fails to execute.
            /// - If the first column cannot be converted to a string.
            /// - If row decoding or PHP conversion fails.
            ///
            /// # Notes
            /// - The iteration order of the returned map is **not** guaranteed.
            pub fn query_dictionary_assoc(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_dictionary(query, parameters, Some(true))
            }

            /// Executes an SQL query and returns a dictionary (map) indexed by the first column of each row,
            /// returning each row as a PHP object.
            ///
            /// # Parameters
            /// - `query`: SQL query string to execute.
            /// - `parameters`: Optional map of named parameters to bind.
            ///
            /// # Returns
            /// A map from the first column (as string) to the full row (as an object).
            ///
            /// # Errors
            /// - If the query fails to execute.
            /// - If the first column cannot be converted to a string.
            /// - If row decoding or PHP conversion fails.
            ///
            /// # Notes
            /// - The iteration order of the returned map is **not** guaranteed.
            pub fn query_dictionary_obj(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_dictionary(query, parameters, Some(false))
            }

            /// Executes an SQL query and returns a dictionary grouping rows by the first column.
            ///
            /// Each row in the result must contain at least one column. The **first column** is used as the **key**, and the
            /// **entire row** is converted to a PHP value and added to the list associated with that key.
            ///
            /// # Parameters
            /// - `query`: SQL query string, optionally using `$param`, `:param`, or positional `?` placeholders.
            /// - `parameters`: Optional key–value map of parameters to bind into the query.
            /// - `associative_arrays`: If `true`, rows are rendered as PHP associative arrays. If `false`, rows are rendered as objects.
            ///   If `None`, falls back to the value in `OPT_ASSOC_ARRAYS`.
            ///
            /// # Returns
            /// A `HashMap<String, Vec<Zval>>` mapping each unique value of the first column to a `Vec` of corresponding rows.
            ///
            /// # Example
            /// A query like:
            /// ```sql
            /// SELECT category, name FROM products
            /// ```
            /// could produce:
            /// ```php
            /// [
            ///   "Books" => [ ["category" => "Books", "name" => "Rust in Action"], ... ],
            ///   "Toys"  => [ ["category" => "Toys", "name" => "Lego Set"], ... ],
            /// ]
            /// ```
            ///
            /// # Errors
            /// Returns an error if:
            /// - The query fails to render or execute.
            /// - The first column in any row is `NULL` or cannot be converted to a PHP string.
            /// - Any row cannot be fully converted to a PHP value.
            ///
            /// # Notes
            /// - Row order within each group is preserved
            /// - The outer dictionary order is preserved.
            /// - Use this method when your result naturally groups by a field, e.g., for building nested structures or aggregations.
            pub fn query_grouped_dictionary(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_grouped_dictionary(query, parameters, None)
            }

            /// Same as `query_grouped_dictionary`, but forces each row to be represented as a PHP associative array.
            ///
            /// This overrides the driver’s default associative/object mode for this call only.
            ///
            /// # Errors
            /// - If the first column is not convertible to string.
            /// - If any row fails to convert to an associative array.
            pub fn query_grouped_dictionary_assoc(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_grouped_dictionary(query, parameters, Some(true))
            }

            /// Same as `query_grouped_dictionary`, but forces each row to be represented as a PHP object.
            ///
            /// This overrides the driver’s default associative/object mode for this call only.
            ///
            /// # Errors
            /// - If the first column is not convertible to string.
            /// - If any row fails to convert to an object.
            pub fn query_grouped_dictionary_obj(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_grouped_dictionary(query, parameters, Some(false))
            }

            /// Executes an SQL query and returns a dictionary mapping the first column to the second column.
            ///
            /// This method expects each result row to contain at least two columns. It converts the first column
            /// into a PHP string (used as the key), and the second column into a PHP value (used as the value).
            ///
            /// # Parameters
            /// - `query`: SQL query string with optional placeholders (e.g., `$param`, `:param`, etc.).
            /// - `parameters`: Optional associative array of parameters to bind into the query.
            ///
            /// # Returns
            /// An associative array (`array<string, mixed>`) where each key is the first column (as string),
            /// and the value is the second column.
            ///
            /// # Errors
            /// Returns an error if:
            /// - the query fails to render or execute;
            /// - the first column cannot be converted to a PHP string;
            /// - the second column cannot be converted to a PHP value.
            ///
            /// # Notes
            /// - The iteration order of dictionary entries is preserved.
            /// - The query must return at least two columns per row.
            pub fn query_column_dictionary(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_column_dictionary(query, parameters, None)
            }

            /// Executes an SQL query and returns a dictionary using associative array mode for values.
            ///
            /// Same as `query_column_dictionary`, but forces JSON objects to be represented as associative arrays.
            ///
            /// # Parameters
            /// - `query`: SQL query string.
            /// - `parameters`: Optional associative array of bind parameters.
            ///
            /// # Returns
            /// Dictionary where each key is the first column, and the value is the second column
            /// converted into an associative PHP array.
            ///
            /// # Errors
            /// Same as `query_column_dictionary`.
            pub fn query_column_dictionary_assoc(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_column_dictionary(query, parameters, Some(true))
            }

            /// Executes an SQL query and returns a dictionary using object mode for values.
            ///
            /// Same as `query_column_dictionary`, but forces JSON objects to be represented as PHP objects.
            ///
            /// # Parameters
            /// - `query`: SQL query string.
            /// - `parameters`: Optional associative array of bind parameters.
            ///
            /// # Returns
            /// Dictionary where each key is the first column, and the value is the second column
            /// converted into a PHP object.
            ///
            /// # Errors
            /// Same as `query_column_dictionary`.
            pub fn query_column_dictionary_obj(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_column_dictionary(query, parameters, Some(false))
            }

            /// Same as `queryGroupedColumnDictionary()`, but forces associative arrays
            /// for the second column if it's a JSON object.
            pub fn query_grouped_column_dictionary_assoc(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_grouped_column_dictionary(query, parameters, Some(true))
            }

            /// Same as `queryGroupedColumnDictionary()`, but forces PHP objects
            /// for the second column if it's a JSON object.
            pub fn query_grouped_column_dictionary_obj(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_grouped_column_dictionary(query, parameters, Some(false))
            }

            /// Executes an SQL query and returns a grouped dictionary where:
            /// - the key is the **first column** (must be scalar),
            /// - the value is a list of values from the **second column** for each group.
            ///
            /// Useful for queries that logically group rows, such as:
            /// ```sql
            /// SELECT category, product_name FROM products
            /// ```
            /// Result:
            /// ```php
            /// [
            ///   "Books" => ["Rust in Action", "The Pragmatic Programmer"],
            ///   "Gadgets" => ["Raspberry Pi"]
            /// ]
            /// ```
            ///
            /// Throws an error if the first column is not a string.
            pub fn query_grouped_column_dictionary(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_grouped_column_dictionary(query, parameters, None)
            }

            /// Describes table columns with their types and metadata.
            ///
            /// Returns information about each column in the specified table, including
            /// name, type, nullability, default value, and ordinal position.
            ///
            /// # Parameters
            /// - `table`: Name of the table to describe.
            /// - `schema`: Optional schema name. If `None`, uses the database default schema.
            ///
            /// # Returns
            /// An array of associative arrays, each containing:
            /// - `name`: Column name (string)
            /// - `type`: Database-specific column type (string, e.g., "varchar(255)", "int")
            /// - `nullable`: Whether the column allows NULL values (bool)
            /// - `default`: Default value for the column (string|null)
            /// - `ordinal`: Column position, 1-based (int)
            ///
            /// # Example
            /// ```php
            /// $columns = $driver->describeTable('users');
            /// // [
            /// //   ['name' => 'id', 'type' => 'integer', 'nullable' => false, 'default' => null, 'ordinal' => 1],
            /// //   ['name' => 'email', 'type' => 'varchar(255)', 'nullable' => false, 'default' => null, 'ordinal' => 2],
            /// // ]
            ///
            /// // With schema
            /// $columns = $driver->describeTable('users', 'public');
            /// ```
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - the table or schema name is invalid (contains invalid characters);
            /// - the query fails to execute;
            /// - the table does not exist.
            pub fn describe_table(
                &self,
                table: &str,
                schema: Option<&str>,
            ) -> $crate::error::Result<Vec<Zval>> {
                self.driver_inner.describe_table(table, schema)
            }

            /// Executes an INSERT/UPDATE/DELETE query and returns affected row count.
            ///
            /// # Arguments
            /// - `query`: SQL query string
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Returns
            /// Number of affected rows
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - the SQL query is invalid or fails to execute (e.g., due to syntax error, constraint violation, or connection issue);
            /// - parameters contain unsupported types or fail to bind correctly;
            /// - the runtime fails to execute the query (e.g., task panic or timeout).
            pub fn execute(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<u64> {
                self.driver_inner.execute(query, parameters)
            }

            /// Inserts a row into the given table using a map of columns.
            ///
            /// # Arguments
            /// - `table`: Table name
            /// - `row`: Map of column names to values
            ///
            /// # Returns
            /// Number of inserted rows
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - the SQL query is invalid or fails to execute (e.g., due to syntax error, constraint violation, or connection issue);
            /// - parameters contain unsupported types or fail to bind correctly;
            /// - the runtime fails to execute the query (e.g., task panic or timeout).
            pub fn insert(
                &self,
                table: &str,
                row: BTreeMap<String, ParameterValue>,
            ) -> $crate::error::Result<u64> {
                self.execute(
                    &format!(
                        "INSERT INTO {table} ({}) VALUES ({})",
                        row.keys().join(", "),
                        row.keys().map(|k| format!("${k}")).join(", ")
                    ),
                    Some(row),
                )
            }

            /// Inserts multiple rows into the given table in a single statement.
            ///
            /// All rows must have the same columns (determined by the first row).
            /// Missing columns in subsequent rows will use `NULL`.
            ///
            /// # Arguments
            /// - `table`: Table name
            /// - `rows`: Vector of maps, each representing a row (column name → value)
            ///
            /// # Returns
            /// Number of inserted rows
            ///
            /// # Example
            /// ```php
            /// $driver->insertMany('users', [
            ///     ['name' => 'Alice', 'email' => 'alice@example.com'],
            ///     ['name' => 'Bob', 'email' => 'bob@example.com'],
            ///     ['name' => 'Carol', 'email' => 'carol@example.com'],
            /// ]);
            /// ```
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - the rows array is empty;
            /// - the SQL query fails to execute;
            /// - parameters contain unsupported types.
            pub fn insert_many(
                &self,
                table: &str,
                rows: Vec<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<u64> {
                if rows.is_empty() {
                    return Err(SqlxError::Other("insertMany requires at least one row".to_string()));
                }

                // Get columns from first row (clone to avoid borrow issues)
                let columns: Vec<String> = rows[0].keys().cloned().collect();
                let num_rows = rows.len();

                // Build VALUES tuples with unique parameter names
                let values_clauses: Vec<String> = (0..num_rows)
                    .map(|i| {
                        let placeholders: Vec<String> = columns
                            .iter()
                            .map(|col| format!("${col}_{i}"))
                            .collect();
                        format!("({})", placeholders.join(", "))
                    })
                    .collect();

                // Collect all parameters with unique names
                let mut all_params = BTreeMap::new();
                for (i, row) in rows.into_iter().enumerate() {
                    for col in &columns {
                        let param_name = format!("{col}_{i}");
                        let value = row.get(col).cloned().unwrap_or(ParameterValue::Null);
                        all_params.insert(param_name, value);
                    }
                }

                let sql = format!(
                    "INSERT INTO {table} ({}) VALUES {}",
                    columns.iter().join(", "),
                    values_clauses.join(", ")
                );

                self.execute(&sql, Some(all_params))
            }

            /// Inserts a row or updates it if a conflict occurs on the specified columns.
            ///
            /// This method generates database-specific SQL for upsert operations:
            /// - **PostgreSQL**: `INSERT ... ON CONFLICT (cols) DO UPDATE SET ...`
            /// - **MySQL**: `INSERT ... ON DUPLICATE KEY UPDATE ...`
            /// - **MSSQL**: Not currently supported (returns an error)
            ///
            /// # Arguments
            /// - `table`: Table name to insert into
            /// - `row`: Map of column names to values for the row to insert
            /// - `conflict_columns`: Columns that form the unique constraint for conflict detection
            /// - `update_columns`: Optional list of columns to update on conflict.
            ///   If `None` or empty, all non-conflict columns from `row` are updated.
            ///
            /// # Returns
            /// Number of affected rows (1 for insert, 2 for update on some databases)
            ///
            /// # Example
            /// ```php
            /// // Insert or update user by email (unique constraint)
            /// $driver->upsert('users', [
            ///     'email' => 'alice@example.com',
            ///     'name' => 'Alice',
            ///     'login_count' => 1
            /// ], ['email'], ['name', 'login_count']);
            ///
            /// // Update all non-key columns on conflict
            /// $driver->upsert('users', $userData, ['email']);
            /// ```
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - the database type doesn't support upsert (MSSQL);
            /// - the SQL query fails to execute;
            /// - parameters contain unsupported types.
            pub fn upsert(
                &self,
                table: &str,
                row: BTreeMap<String, ParameterValue>,
                conflict_columns: Vec<String>,
                update_columns: Option<Vec<String>>,
            ) -> $crate::error::Result<u64> {
                // Determine which columns to update on conflict
                let conflict_set: std::collections::BTreeSet<_> = conflict_columns.iter().collect();
                let update_cols: Vec<&String> = match &update_columns {
                    Some(cols) if !cols.is_empty() => cols.iter().collect(),
                    _ => row.keys().filter(|k| !conflict_set.contains(k)).collect(),
                };

                // Build SQL based on upsert_style setting
                let query = match self.driver_inner.settings.upsert_style {
                    UpsertStyle::OnConflict => {
                        // PostgreSQL: INSERT ... ON CONFLICT (cols) DO UPDATE SET col = EXCLUDED.col
                        let update_set = update_cols
                            .iter()
                            .map(|c| format!("{c} = EXCLUDED.{c}"))
                            .join(", ");

                        if update_set.is_empty() {
                            // No columns to update - use DO NOTHING
                            format!(
                                "INSERT INTO {table} ({}) VALUES ({}) ON CONFLICT ({}) DO NOTHING",
                                row.keys().join(", "),
                                row.keys().map(|k| format!("${k}")).join(", "),
                                conflict_columns.join(", ")
                            )
                        } else {
                            format!(
                                "INSERT INTO {table} ({}) VALUES ({}) ON CONFLICT ({}) DO UPDATE SET {}",
                                row.keys().join(", "),
                                row.keys().map(|k| format!("${k}")).join(", "),
                                conflict_columns.join(", "),
                                update_set
                            )
                        }
                    }
                    UpsertStyle::OnDuplicateKey => {
                        // MySQL: INSERT ... ON DUPLICATE KEY UPDATE col = VALUES(col)
                        let update_set = update_cols
                            .iter()
                            .map(|c| format!("{c} = VALUES({c})"))
                            .join(", ");

                        if update_set.is_empty() {
                            // No columns to update - use INSERT IGNORE
                            format!(
                                "INSERT IGNORE INTO {table} ({}) VALUES ({})",
                                row.keys().join(", "),
                                row.keys().map(|k| format!("${k}")).join(", ")
                            )
                        } else {
                            format!(
                                "INSERT INTO {table} ({}) VALUES ({}) ON DUPLICATE KEY UPDATE {}",
                                row.keys().join(", "),
                                row.keys().map(|k| format!("${k}")).join(", "),
                                update_set
                            )
                        }
                    }
                    UpsertStyle::Unsupported => {
                        return Err(SqlxError::Other(
                            "upsert() is not supported for this database. Use MERGE statement directly.".to_string()
                        ));
                    }
                };

                self.execute(&query, Some(row))
            }

            /// Executes an SQL query and returns the rendered query and its parameters.
            ///
            /// This method does not execute the query but returns the SQL string with placeholders
            /// and the bound parameter values for debugging or logging purposes.
            ///
            /// # Arguments
            /// - `query`: SQL query string
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Returns
            /// A list where the first element is the rendered SQL query (string), and the second is an array of bound values
            ///
            /// # Exceptions
            /// Throws an exception if the query can't be parsed, rendered, or if parameters
            /// cannot be converted from PHP values.
            pub fn dry(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Vec<Zval>> {
                self.driver_inner.dry(query, parameters)
            }

            /// Registers a callback to be invoked after each query execution.
            ///
            /// The callback receives:
            /// - `string $sql` - The rendered SQL query with placeholders
            /// - `string $sqlInline` - The SQL query with inlined parameter values (for logging)
            /// - `float $durationMs` - Execution time in milliseconds
            ///
            /// # Example
            /// ```php
            /// $driver->onQuery(function(string $sql, string $sqlInline, float $durationMs) {
            ///     Logger::debug("Query took {$durationMs}ms: $sqlInline");
            /// });
            ///
            /// // Disable the hook
            /// $driver->onQuery(null);
            /// ```
            ///
            /// # Performance
            /// When no hook is registered, there is zero overhead. When a hook is active,
            /// timing measurements are taken and the callback is invoked after each query.
            ///
            /// # Notes
            /// - Exceptions thrown by the callback are silently ignored to avoid disrupting query execution
            /// - The hook applies to all query methods: `query*`, `execute`, `insert`
            /// - The hook is NOT inherited by prepared queries or query builders
            pub fn on_query(&self, callback: &Zval) {
                if callback.is_null() {
                    self.driver_inner.query_hook.clear();
                } else {
                    self.driver_inner.query_hook.set(callback.shallow_clone());
                }
            }

            /// Sets the application name for this connection.
            ///
            /// This helps identify the connection in database monitoring tools:
            /// - `PostgreSQL`: Visible in `pg_stat_activity.application_name`
            /// - `MySQL`: Stored in session variable `@sqlx_application_name`
            /// - `MSSQL`: Stored in session context via `sp_set_session_context`
            ///
            /// # Example
            /// ```php
            /// $driver->setApplicationName('order-service');
            /// ```
            ///
            /// # Exceptions
            /// Throws an exception if the query fails to execute.
            pub fn set_application_name(&self, name: &str) -> $crate::error::Result<()> {
                self.driver_inner.set_application_name(name)
            }

            /// Sets client metadata for this connection.
            ///
            /// The metadata is formatted and appended to the application name,
            /// making it visible in database monitoring tools. This is useful for tracking
            /// request IDs, user IDs, or other debugging information.
            ///
            /// # Example
            /// ```php
            /// $driver->setClientInfo('order-service', ['request_id' => $requestId, 'user_id' => $userId]);
            /// // In pg_stat_activity: "order-service {request_id='abc123',user_id=42}"
            /// ```
            ///
            /// # Exceptions
            /// Throws an exception if the query fails to execute.
            pub fn set_client_info(
                &self,
                application_name: &str,
                info: BTreeMap<String, ParameterValue>,
            ) -> $crate::error::Result<()> {
                self.driver_inner.set_client_info(application_name, &info)
            }

            /// Returns true if read replicas are configured for this driver.
            ///
            /// When read replicas are configured, SELECT queries are automatically
            /// routed to replicas using round-robin selection, while write operations
            /// (INSERT/UPDATE/DELETE) always go to the primary.
            ///
            /// # Example
            /// ```php
            /// if ($driver->hasReadReplicas()) {
            ///     echo "Read queries will be load balanced across replicas";
            /// }
            /// ```
            pub fn has_read_replicas(&self) -> bool {
                self.driver_inner.has_read_replicas()
            }

            /// Begins a SQL transaction, optionally executing a callable within it.
            ///
            /// This method supports two modes of operation:
            ///
            /// **Mode 1: Callback-based (automatic commit/rollback)**
            /// ```php
            /// $driver->begin(function($driver) {
            ///     $driver->execute('INSERT INTO users (name) VALUES (?)', ['John']);
            ///     return true; // true = commit, false = rollback
            /// });
            /// ```
            ///
            /// **Mode 2: Imperative (manual commit/rollback)**
            /// ```php
            /// $driver->begin();
            /// try {
            ///     $driver->execute('INSERT INTO users (name) VALUES (?)', ['John']);
            ///     $driver->commit();
            /// } catch (\Exception $e) {
            ///     $driver->rollback();
            ///     throw $e;
            /// }
            /// ```
            ///
            /// # Parameters
            /// - `callable`: Optional PHP callable receiving this Driver instance.
            ///
            /// # Behavior (with callable)
            /// - Starts a transaction.
            /// - Invokes `callable($this)`.
            /// - If the callable returns false, rolls back; commits otherwise.
            /// - On exception or callable error, rolls back and rethrows.
            ///
            /// # Behavior (without callable)
            /// - Starts a transaction and leaves it active on the transaction stack.
            /// - You must manually call `commit()` or `rollback()` to finish.
            ///
            /// # Exceptions
            /// Throws an exception if transaction start, commit, rollback,
            /// or callable invocation fails.
            ///
            pub fn begin(&self, callable: Option<ZendCallable>) -> PhpResult<bool> {
                self.driver_inner.begin()?;

                // If no callable provided, leave transaction active for manual control
                if callable.is_none() {
                    return Ok(true);
                }

                // Callback-based transaction with automatic commit/rollback
                let callable = callable.unwrap();
                let callable_ret = callable.try_call(vec![self]);
                let tx = self.driver_inner.retrieve_ongoing_transaction().unwrap();
                match callable_ret {
                    Ok(value) => {
                        if value.is_false() {
                            $crate::RUNTIME
                                .block_on(tx.rollback())
                                .map_err(SqlxError::rollback_failed)?;
                        } else {
                            $crate::RUNTIME
                                .block_on(tx.commit())
                                .map_err(SqlxError::commit_failed)?;
                        }
                        Ok(true)
                    }
                    Err(err) => {
                        $crate::RUNTIME
                            .block_on(tx.rollback())
                            .map_err(SqlxError::rollback_failed)?;
                        match err {
                            ext_php_rs::error::Error::Exception(exception) => Err(exception
                                .properties_table[0]
                                .string()
                                .as_ref()
                                .map(String::as_str)
                                .unwrap_or("Unknown error inside callback.")
                                .into()),
                            _ => Err(err.into()),
                        }
                    }
                }
            }

            /// Creates a transaction savepoint with the given name.
            ///
            /// # Parameters
            /// - `savepoint`: Name of the savepoint to create.
            ///
            /// # Exceptions
            /// Throws an exception if the driver fails to create the savepoint.
            pub fn savepoint(&self, savepoint: &str) -> $crate::error::Result<()> {
                self.driver_inner.savepoint(savepoint)
            }

            /// Rolls back the current transaction to a previously created savepoint.
            ///
            /// # Parameters
            /// - `savepoint`: Name of the savepoint to rollback to.
            ///
            /// # Exceptions
            /// Throws an exception if rollback to the savepoint fails.
            pub fn rollback_to_savepoint(&self, savepoint: &str) -> $crate::error::Result<()> {
                self.driver_inner.rollback_to_savepoint(savepoint)
            }

            /// Releases a previously created savepoint, making it no longer available.
            ///
            /// # Parameters
            /// - `savepoint`: Name of the savepoint to release.
            ///
            /// # Exceptions
            /// Throws an exception if releasing the savepoint fails.
            pub fn release_savepoint(&self, savepoint: &str) -> $crate::error::Result<()> {
                self.driver_inner.release_savepoint(savepoint)
            }

            /// Commits the current ongoing transaction.
            ///
            /// This method should be called after `begin()` was called without a callable.
            /// It commits all changes made during the transaction and removes the transaction
            /// from the stack.
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - no transaction is currently active
            /// - the commit operation fails
            ///
            /// # Example
            /// ```php
            /// $driver->begin();
            /// try {
            ///     $driver->execute('INSERT INTO users (name) VALUES (?)', ['John']);
            ///     $driver->execute('INSERT INTO logs (action) VALUES (?)', ['user_created']);
            ///     $driver->commit();
            /// } catch (\Exception $e) {
            ///     $driver->rollback();
            ///     throw $e;
            /// }
            /// ```
            pub fn commit(&self) -> $crate::error::Result<()> {
                self.driver_inner.commit()
            }

            /// Rolls back the current ongoing transaction.
            ///
            /// This method should be called after `begin()` was called without a callable.
            /// It discards all changes made during the transaction and removes the transaction
            /// from the stack.
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - no transaction is currently active
            /// - the rollback operation fails
            ///
            /// # Example
            /// ```php
            /// $driver->begin();
            /// try {
            ///     $driver->execute('INSERT INTO users (name) VALUES (?)', ['John']);
            ///     $driver->execute('INSERT INTO logs (action) VALUES (?)', ['user_created']);
            ///     $driver->commit();
            /// } catch (\Exception $e) {
            ///     $driver->rollback();
            ///     throw $e;
            /// }
            /// ```
            pub fn rollback(&self) -> $crate::error::Result<()> {
                self.driver_inner.rollback()
            }

            /// Executes a callback with a pinned connection from the pool.
            ///
            /// All queries executed within the callback will use the same database connection,
            /// which is required for session-scoped operations like:
            /// - `LAST_INSERT_ID()` in MySQL
            /// - Temporary tables
            /// - Session variables
            /// - Advisory locks
            ///
            /// Unlike `begin()`, this does NOT start a database transaction - each query is
            /// auto-committed. Use `begin()` if you need transactional semantics.
            ///
            /// # Parameters
            /// - `callable`: A callback function that receives the driver and executes queries.
            ///
            /// # Returns
            /// The value returned by the callback.
            ///
            /// # Example
            /// ```php
            /// $lastId = $driver->withConnection(function($driver) {
            ///     $driver->execute("INSERT INTO users (name) VALUES ('Alice')");
            ///     return $driver->queryValue('SELECT LAST_INSERT_ID()');
            /// });
            /// ```
            ///
            /// # Exceptions
            /// Throws an exception if connection acquisition fails or the callback throws.
            pub fn with_connection(&self, callable: ZendCallable) -> PhpResult<Zval> {
                self.driver_inner.pin_connection()?;

                let callable_ret = callable.try_call(vec![self]);

                // Always unpin, even if callback failed
                let _ = self.driver_inner.unpin_connection();

                match callable_ret {
                    Ok(value) => Ok(value),
                    Err(err) => {
                        match err {
                            ext_php_rs::error::Error::Exception(exception) => Err(exception
                                .properties_table[0]
                                .string()
                                .as_ref()
                                .map(String::as_str)
                                .unwrap_or("Unknown error inside callback.")
                                .into()),
                            _ => Err(err.into()),
                        }
                    }
                }
            }
        }
    };

    ( $( $t:tt )* ) => {
        compile_error!(
            "php_sqlx_impl_driver! accepts 6 arguments: \
             (DriverType, $className, InnerDriverType, PreparedQueryType, ReadQueryBuilder, WriteQueryBuilder)"
        );
    };
}
