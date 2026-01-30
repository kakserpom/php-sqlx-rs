//! Native Rust interface definitions for php-sqlx.
//!
//! This module defines PHP interfaces using ext-php-rs's `#[php_interface]` macro.
//!
//! # Interfaces
//!
//! - [`DriverInterface`]: Main database driver contract
//! - [`PreparedQueryInterface`]: Prepared statement contract
//! - [`ReadQueryBuilderInterface`]: Read-only query builder contract
//! - [`WriteQueryBuilderInterface`]: Full query builder contract

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use std::collections::BTreeMap;

use crate::param_value::ParameterValue;
use crate::utils::types::ColumnArgument;

/// Interface for database drivers.
///
/// This interface defines the contract that all database drivers must implement,
/// providing methods for querying, executing statements, and transactions.
///
/// Implementing classes: `PgDriver`, `MySqlDriver`, `MssqlDriver`
#[php_interface]
#[php(name = "Sqlx\\DriverInterface")]
pub trait DriverInterface {
    /// Closes the connection pool and releases all database connections.
    fn close(&self);

    /// Returns true if the driver has been closed.
    fn is_closed(&self) -> bool;

    /// Returns whether results are returned as associative arrays.
    fn assoc_arrays(&self) -> bool;

    /// Quotes a single scalar value for safe embedding into SQL.
    fn quote(&self, param: ParameterValue) -> crate::error::Result<String>;

    /// Quotes a string for use in a LIKE/ILIKE pattern.
    fn quote_like(&self, param: ParameterValue) -> crate::error::Result<String>;

    /// Quotes an identifier (table name, column name).
    fn quote_identifier(&self, name: &str) -> String;

    /// Executes an SQL query and returns a single row.
    fn query_row(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Zval>;

    /// Executes an SQL query and returns a single row as associative array.
    fn query_row_assoc(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Zval>;

    /// Executes an SQL query and returns a single row as object.
    fn query_row_obj(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Zval>;

    /// Executes an SQL query and returns a single row or null.
    fn query_maybe_row(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Zval>;

    /// Executes an SQL query and returns a single row as associative array or null.
    fn query_maybe_row_assoc(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Zval>;

    /// Executes an SQL query and returns a single row as object or null.
    fn query_maybe_row_obj(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Zval>;

    /// Executes an SQL query and returns a single column value.
    fn query_value(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> crate::error::Result<Zval>;

    /// Executes an SQL query and returns a single column value as associative array.
    fn query_value_assoc(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> crate::error::Result<Zval>;

    /// Executes an SQL query and returns a single column value as object.
    fn query_value_obj(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> crate::error::Result<Zval>;

    /// Executes an SQL query and returns a single column value or null.
    fn query_maybe_value(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> crate::error::Result<Zval>;

    /// Executes an SQL query and returns a single column value as associative array or null.
    fn query_maybe_value_assoc(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> crate::error::Result<Zval>;

    /// Executes an SQL query and returns a single column value as object or null.
    fn query_maybe_value_obj(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> crate::error::Result<Zval>;

    /// Executes an SQL query and returns all rows.
    fn query_all(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Vec<Zval>>;

    /// Executes an SQL query and returns all rows as associative arrays.
    fn query_all_assoc(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Vec<Zval>>;

    /// Executes an SQL query and returns all rows as objects.
    fn query_all_obj(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Vec<Zval>>;

    /// Executes an SQL query and returns a column from all rows.
    fn query_column(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> crate::error::Result<Vec<Zval>>;

    /// Executes an SQL query and returns a column from all rows as associative arrays.
    fn query_column_assoc(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> crate::error::Result<Vec<Zval>>;

    /// Executes an SQL query and returns a column from all rows as objects.
    fn query_column_obj(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> crate::error::Result<Vec<Zval>>;

    /// Executes an SQL query and returns a dictionary indexed by the first column.
    fn query_dictionary(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Zval>;

    /// Executes an SQL query and returns a dictionary with rows as associative arrays.
    fn query_dictionary_assoc(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Zval>;

    /// Executes an SQL query and returns a dictionary with rows as objects.
    fn query_dictionary_obj(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Zval>;

    /// Executes an INSERT/UPDATE/DELETE statement and returns affected rows.
    fn execute(
        &self,
        query: &str,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<u64>;
}

/// Interface for prepared queries.
///
/// Implementing classes: `PgPreparedQuery`, `MySqlPreparedQuery`, `MssqlPreparedQuery`
#[php_interface]
#[php(name = "Sqlx\\PreparedQueryInterface")]
pub trait PreparedQueryInterface {
    /// Executes the prepared statement and returns affected rows.
    fn execute(
        &self,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<u64>;

    /// Executes the prepared query and returns a single row.
    fn query_row(
        &self,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Zval>;

    /// Executes the prepared query and returns a single row as associative array.
    fn query_row_assoc(
        &self,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Zval>;

    /// Executes the prepared query and returns a single row as object.
    fn query_row_obj(
        &self,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Zval>;

    /// Executes the prepared query and returns a single row or null.
    fn query_maybe_row(
        &self,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Zval>;

    /// Executes the prepared query and returns all rows.
    fn query_all(
        &self,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Vec<Zval>>;

    /// Executes the prepared query and returns all rows as associative arrays.
    fn query_all_assoc(
        &self,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Vec<Zval>>;

    /// Executes the prepared query and returns all rows as objects.
    fn query_all_obj(
        &self,
        parameters: Option<BTreeMap<String, ParameterValue>>,
    ) -> crate::error::Result<Vec<Zval>>;

    /// Executes the prepared query and returns a single column value.
    fn query_value(
        &self,
        parameters: Option<BTreeMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> crate::error::Result<Zval>;

    /// Executes the prepared query and returns a column from all rows.
    fn query_column(
        &self,
        parameters: Option<BTreeMap<String, ParameterValue>>,
        column: Option<ColumnArgument>,
    ) -> crate::error::Result<Vec<Zval>>;
}

/// Interface for read-only query builders.
///
/// Implementing classes: `PgReadQueryBuilder`, `MySqlReadQueryBuilder`, `MssqlReadQueryBuilder`
#[php_interface]
#[php(name = "Sqlx\\ReadQueryBuilderInterface")]
pub trait ReadQueryBuilderInterface {}

/// Interface for write query builders.
///
/// Implementing classes: `PgWriteQueryBuilder`, `MySqlWriteQueryBuilder`, `MssqlWriteQueryBuilder`
#[php_interface]
#[php(name = "Sqlx\\WriteQueryBuilderInterface")]
pub trait WriteQueryBuilderInterface {}
