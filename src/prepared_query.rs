use crate::ColumnArgument;
use crate::DriverInner;
use crate::Value;
use ext_php_rs::{prelude::*, types::Zval};
use std::collections::HashMap;
use std::sync::Arc;
/// A reusable prepared SQL query with parameter support.
///
/// Created using `Driver::prepare()`, shares context with original driver.
#[php_class(name = "Sqlx\\PreparedQuery")]
pub struct PreparedQuery {
    pub(crate) query: String,
    pub(crate) driver_inner: Arc<DriverInner>,
}

#[php_impl]
impl PreparedQuery {
    /// Executes the prepared query and returns a dictionary mapping the first column to the second column.
    ///
    /// This method expects each result row to contain at least two columns. It converts the first column
    /// into a PHP string (used as the key), and the second column into a PHP value (used as the value).
    ///
    /// # Parameters
    /// - `parameters`: Optional array of indexed/named parameters to bind.
    ///
    /// # Returns
    /// An associative array (`array<string, mixed>`) where each key is the first column (as string),
    /// and the value is the second column.
    ///
    /// # Errors
    /// Returns an error if:
    /// - the query fails to execute;
    /// - the first column cannot be converted to a PHP string;
    /// - the second column cannot be converted to a PHP value.
    ///
    /// # Notes
    /// - The order of dictionary entries is preserved.
    /// - The query must return at least two columns per row.
    pub fn query_column_dictionary(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_column_dictionary(&self.query, parameters, None)
    }

    /// Executes the prepared query and returns a dictionary in associative array mode.
    ///
    /// Same as `query_column_dictionary`, but forces JSON objects to be represented as associative arrays.
    ///
    /// # Parameters
    /// - `parameters`: Optional array of indexed/named parameters to bind.
    ///
    /// # Returns
    /// A dictionary where each key is the first column (as string),
    /// and each value is the second column as an associative PHP array.
    ///
    /// # Errors
    /// Same as `query_column_dictionary`.
    pub fn query_column_dictionary_assoc(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_column_dictionary(&self.query, parameters, Some(true))
    }

    /// Executes the prepared query and returns a dictionary in object mode.
    ///
    /// Same as `query_column_dictionary`, but forces JSON objects to be represented as objects.
    ///
    /// # Parameters
    /// - `parameters`: Optional array of indexed/named parameters to bind.
    ///
    /// # Returns
    /// A dictionary where each key is the first column (as string),
    /// and each value is the second column as a PHP object.
    ///
    /// # Errors
    /// Same as `query_column_dictionary`.
    pub fn query_column_dictionary_obj(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_column_dictionary(&self.query, parameters, Some(false))
    }

    /// Executes the prepared query and returns a dictionary (map) indexed by the first column of each row.
    ///
    /// The result is a `HashMap` where the key is the stringified first column from each row,
    /// and the value is the full row, returned as array or object depending on config.
    ///
    /// # Parameters
    /// - `parameters`: Optional map of named parameters to bind.
    ///
    /// # Returns
    /// A map from the first column (as string) to the corresponding row.
    ///
    /// # Errors
    /// Returns an error if:
    /// - the query fails to execute;
    /// - the first column cannot be converted to a string;
    /// - any row cannot be decoded or converted to a PHP value.
    ///
    /// # Notes
    /// - The iteration order of the returned map is **not** guaranteed.
    pub fn query_dictionary(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_dictionary(&self.query, parameters, None)
    }

    /// Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
    /// with each row returned as an associative array.
    ///
    /// # Parameters
    /// - `parameters`: Optional map of named parameters to bind.
    ///
    /// # Returns
    /// A map from the first column (as string) to the corresponding row as an associative array.
    ///
    /// # Errors
    /// Returns an error if:
    /// - the query fails to execute;
    /// - the first column cannot be converted to a string;
    /// - any row cannot be decoded or converted to a PHP value.
    ///
    /// # Notes
    /// - The iteration order of the returned map is **not** guaranteed.
    pub fn query_dictionary_assoc(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_dictionary(&self.query, parameters, Some(true))
    }

    /// Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
    /// with each row returned as an object (`stdClass`).
    ///
    /// # Parameters
    /// - `parameters`: Optional map of named parameters to bind.
    ///
    /// # Returns
    /// A map from the first column (as string) to the corresponding row as an object.
    ///
    /// # Errors
    /// Returns an error if:
    /// - the query fails to execute;
    /// - the first column cannot be converted to a string;
    /// - any row cannot be decoded or converted to a PHP value.
    ///
    /// # Notes
    /// - The iteration order of the returned map is **not** guaranteed.
    pub fn query_dictionary_obj(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_dictionary(&self.query, parameters, Some(false))
    }

    /// Executes a query and returns a grouped dictionary (Vec of rows per key).
    ///
    /// Same as [`queryGroupedDictionary`](crate::Driver::query_grouped_dictionary), but works on a prepared query.
    ///
    /// The first column is used as the key (must be convertible to string),
    /// and each resulting row is appended to the corresponding key's Vec.
    ///
    /// # Errors
    /// Fails if the query fails, or the first column is not string-convertible.
    pub fn query_grouped_dictionary(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_grouped_dictionary(&self.query, parameters, None)
    }

    /// Same as `query_grouped_dictionary`, but forces rows to be decoded as associative arrays.
    pub fn query_grouped_dictionary_assoc(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_grouped_dictionary(&self.query, parameters, Some(true))
    }

    /// Same as `query_grouped_dictionary`, but forces rows to be decoded as PHP objects.
    pub fn query_grouped_dictionary_obj(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_grouped_dictionary(&self.query, parameters, Some(false))
    }

    /// Executes the prepared query and returns a grouped dictionary where:
    /// - the key is the **first column** (must be convertible to string),
    /// - the value is a list of values from the **second column** for each group.
    ///
    /// This variant uses the driver's default associative array option for JSON values.
    ///
    /// # Errors
    /// Returns an error if the first column is not convertible to a string.
    pub fn query_grouped_column_dictionary(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_grouped_column_dictionary(&self.query, parameters, None)
    }

    /// Same as `queryGroupedColumnDictionary()`, but forces associative arrays
    /// for the second column if it contains JSON objects.
    ///
    /// # Errors
    /// Returns an error if the first column is not convertible to a string.
    pub fn query_grouped_column_dictionary_assoc(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_grouped_column_dictionary(&self.query, parameters, Some(true))
    }

    /// Same as `queryGroupedColumnDictionary()`, but forces PHP objects
    /// for the second column if it contains JSON objects.
    ///
    /// # Errors
    /// Returns an error if the first column is not convertible to a string.
    pub fn query_grouped_column_dictionary_obj(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_grouped_column_dictionary(&self.query, parameters, Some(false))
    }

    /// Executes the prepared query with optional parameters.
    ///
    /// # Arguments
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
    pub fn execute(&self, parameters: Option<HashMap<String, Value>>) -> anyhow::Result<u64> {
        self.driver_inner.execute(self.query.as_str(), parameters)
    }

    /// Executes the prepared query and returns a single result.
    ///
    /// # Arguments
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
    pub fn query_row(&self, parameters: Option<HashMap<String, Value>>) -> anyhow::Result<Zval> {
        self.driver_inner.query_row(&self.query, parameters, None)
    }

    /// Executes the prepared query and returns one row as an associative array.
    ///
    /// # Arguments
    /// - `parameters`: Optional array of indexed/named parameters to bind.
    pub fn query_row_assoc(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_row(&self.query, parameters, Some(true))
    }

    /// Executes the prepared query and returns one row as an object.
    ///
    /// # Arguments
    /// - `parameters`: Optional array of indexed/named parameters to bind.
    pub fn query_row_obj(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_row(&self.query, parameters, Some(false))
    }

    /// Executes an SQL query and returns a single result, or `null` if no row matched.
    ///
    /// # Arguments
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
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_maybe_row(&self.query, parameters, None)
    }

    /// Executes the SQL query and returns a single row as a PHP associative array, or `null` if no row matched.
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
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_maybe_row(&self.query, parameters, Some(true))
    }

    /// Executes an SQL query and returns a single row as a PHP object, or `null` if no row matched.
    ///
    /// # Arguments
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
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_maybe_row(&self.query, parameters, Some(false))
    }

    /// Executes the SQL query and returns the specified column values from all result rows.
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
        parameters: Option<HashMap<String, Value>>,
        column: Option<ColumnArgument>,
    ) -> anyhow::Result<Vec<Zval>> {
        self.driver_inner
            .query_column(&self.query, parameters, column, None)
    }

    /// Executes the SQL query and returns the specified column values from all rows in associative array mode.
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
        parameters: Option<HashMap<String, Value>>,
        column: Option<ColumnArgument>,
    ) -> anyhow::Result<Vec<Zval>> {
        self.driver_inner
            .query_column(&self.query, parameters, column, Some(true))
    }

    /// Executes the SQL query and returns the specified column values from all rows in object mode.
    ///
    /// # Arguments
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
        parameters: Option<HashMap<String, Value>>,
        column: Option<ColumnArgument>,
    ) -> anyhow::Result<Vec<Zval>> {
        self.driver_inner
            .query_column(&self.query, parameters, column, Some(false))
    }

    /// Executes the prepared query and returns all rows.
    ///
    /// # Arguments
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
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Vec<Zval>> {
        self.driver_inner.query_all(&self.query, parameters, None)
    }

    /// Executes the prepared query and returns all rows as associative arrays.
    ///
    /// # Arguments
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
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Vec<Zval>> {
        self.driver_inner
            .query_all(&self.query, parameters, Some(true))
    }

    /// Executes the prepared query and returns all rows as objects.
    ///
    /// # Arguments
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
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Vec<Zval>> {
        self.driver_inner
            .query_all(&self.query, parameters, Some(false))
    }
}
