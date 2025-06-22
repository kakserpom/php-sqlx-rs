pub mod ast;
mod conversion;
pub mod prepared_query;

use crate::postgres::inner::PgDriverInner;
pub use crate::postgres::prepared_query::PgPreparedQuery;
use crate::utils::ColumnArgument;

use crate::paramvalue::ParameterValue;
use dashmap::DashMap;
use ext_php_rs::builders::ModuleBuilder;
use ext_php_rs::types::Zval;
use ext_php_rs::{php_class, php_impl};
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use crate::options::DriverOptionsArg;

pub mod inner;

static PERSISTENT_DRIVER_REGISTRY: LazyLock<DashMap<String, Arc<PgDriverInner>>> =
    LazyLock::new(DashMap::new);

/// A Postgres driver using SQLx with query helpers and AST cache.
///
/// This class supports prepared queries, persistent connections, and augmented SQL.
#[php_class]
#[php(name = "Sqlx\\PgDriver")]
#[php(rename = "none")]
pub struct PgDriver {
    pub driver_inner: Arc<PgDriverInner>,
}

#[php_impl]
impl PgDriver {
    /// Constructs a new SQLx driver instance.
    ///
    /// # Arguments
    /// - `options`: Connection URL as string or associative array with options:
    ///   - `url`: (string) database connection string (required)
    ///   - `ast_cache_shard_count`: (int) number of AST cache shards (default: 8)
    ///   - `ast_cache_shard_size`: (int) size per shard (default: 256)
    ///   - `persistent_name`: (string) name of persistent connection
    ///   - `assoc_arrays`: (bool) return associative arrays instead of objects
    pub fn __construct(options: DriverOptionsArg) -> anyhow::Result<Self> {
        let options = options.parse()?;

        if let Some(name) = options.persistent_name.as_ref() {
            if let Some(driver_inner) = PERSISTENT_DRIVER_REGISTRY.get(name) {
                return Ok(Self {
                    driver_inner: driver_inner.clone(),
                });
            }
        }
        let persistent_name = options.persistent_name.clone();
        let driver_inner = Arc::new(PgDriverInner::new(options)?);
        if let Some(name) = persistent_name {
            PERSISTENT_DRIVER_REGISTRY.insert(name, driver_inner.clone());
        }
        Ok(Self { driver_inner })
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_value(query, parameters, column, None)
    }

    pub fn query_value_assoc(
        &self,
        query: &str,
        parameters: Option<HashMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> anyhow::Result<Zval> {
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
    /// The value from the specified column of the first row as a PHP value`, or `null` if no row was found.
    ///
    /// # Exceptions
    /// Throws an exception if:
    /// - the query is invalid or fails to execute;
    /// - the specified column does not exist;
    /// - the value cannot be converted to a PHP-compatible type.
    pub fn query_maybe_value(
        &self,
        query: &str,
        parameters: Option<HashMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> anyhow::Result<Vec<Zval>> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> anyhow::Result<Vec<Zval>> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> anyhow::Result<Vec<Zval>> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Vec<Zval>> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Vec<Zval>> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Vec<Zval>> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_column_dictionary(query, parameters, Some(false))
    }

    /// Same as `queryGroupedColumnDictionary()`, but forces associative arrays
    /// for the second column if it's a JSON object.
    pub fn query_grouped_column_dictionary_assoc(
        &self,
        query: &str,
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_grouped_column_dictionary(query, parameters, Some(true))
    }

    /// Same as `queryGroupedColumnDictionary()`, but forces PHP objects
    /// for the second column if it's a JSON object.
    pub fn query_grouped_column_dictionary_obj(
        &self,
        query: &str,
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Zval> {
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_grouped_column_dictionary(query, parameters, None)
    }
    /// Creates a prepared query object with the given SQL string.
    ///
    /// # Arguments
    /// - `query`: SQL query string to prepare
    ///
    /// # Returns
    /// Prepared query object
    #[must_use]
    pub fn prepare(&self, query: &str) -> PgPreparedQuery {
        PgPreparedQuery {
            driver_inner: self.driver_inner.clone(),
            query: query.to_owned(),
        }
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<u64> {
        self.driver_inner.execute(query, parameters)
    }

    /// Inserts a row into the given table using a map of fields.
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
    pub fn insert(&self, table: &str, row: HashMap<String, ParameterValue>) -> anyhow::Result<u64> {
        self.execute(
            &format!(
                "INSERT INTO {table} ({}) VALUES ({})",
                row.keys().join(", "),
                row.keys().map(|k| format!("${k}")).join(", ")
            ),
            Some(row),
        )
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
        parameters: Option<HashMap<String, ParameterValue>>,
    ) -> anyhow::Result<Vec<Zval>> {
        self.driver_inner.dry(query, parameters)
    }
}

pub fn build(module: ModuleBuilder) -> ModuleBuilder {
    module.class::<PgDriver>().class::<PgPreparedQuery>()
}
