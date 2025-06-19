#![allow(clippy::needless_borrow)]
#![allow(clippy::needless_pass_by_value)]

use crate::driver::conversion::Conversion;
use crate::driver::mysql::ast::{MySqlAst, MySqlParameterValue};
use crate::driver::mysql::options::MySqlDriverInnerOptions;
use crate::utils::{fold_into_zend_hashmap, fold_into_zend_hashmap_grouped};
use crate::{ColumnArgument, RUNTIME, utils::ZvalNull};
use anyhow::{anyhow, bail};
use ext_php_rs::convert::IntoZval;
use ext_php_rs::ffi::zend_array;
use ext_php_rs::types::Zval;
use itertools::Itertools;
use sqlx::Row;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::pool::Pool;
use sqlx::query::Query;
use sqlx::{Column, MySql};
use sqlx::{Database, Encode, Error, Type};
use std::collections::HashMap;
use threadsafe_lru::LruCache;

pub struct MySqlDriverInner {
    pub pool: Pool<MySql>,
    pub ast_cache: LruCache<String, MySqlAst>,
    pub options: MySqlDriverInnerOptions,
}

impl MySqlDriverInner {
    pub fn new(options: MySqlDriverInnerOptions) -> anyhow::Result<Self> {
        let pool = RUNTIME.block_on(
            MySqlPoolOptions::new()
                .max_connections(options.max_connections.into())
                .connect(
                    options
                        .url
                        .clone()
                        .ok_or_else(|| anyhow!("URL must be set"))?
                        .as_str(),
                ),
        )?;
        Ok(MySqlDriverInner {
            pool,
            ast_cache: LruCache::new(options.ast_cache_shard_count, options.ast_cache_shard_size),
            options,
        })
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
        parameters: Option<HashMap<String, MySqlParameterValue>>,
    ) -> anyhow::Result<u64> {
        let (query, values) = self.render_query(query, parameters)?;
        Ok(RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).execute(&self.pool))
            .map_err(|err| anyhow!("{err}\n\nQuery: {query}\n\nValues: {values:?}\n\n"))?
            .rows_affected())
    }

    /// Render the final SQL query and parameters using the AST cache.
    fn render_query(
        &self,
        query: &str,
        parameters: Option<HashMap<String, MySqlParameterValue>>,
    ) -> anyhow::Result<(String, Vec<MySqlParameterValue>)> {
        let parameters = parameters.unwrap_or_default();
        if let Some(ast) = self.ast_cache.get(query) {
            ast.render(parameters)
        } else {
            let ast = MySqlAst::parse(query, self.options.collapsible_in)?;
            let rendered = ast.render(parameters)?;
            self.ast_cache.insert(query.to_owned(), ast);
            Ok(rendered)
        }
    }

    /// Executes an SQL query and returns a single column from the first row.
    ///
    /// # Arguments
    /// - `query`: SQL query string
    /// - `parameters`: Optional array of indexed/named parameters to bind.
    /// - `column`: Optional column index or name to retrieve
    /// - `associative_arrays`: Whether to use associative arrays for complex values (optional)
    ///
    /// # Returns
    /// A single column value.
    ///
    /// # Exceptions
    /// Throws an exception if the query fails, column doesn't exist, or conversion fails
    pub fn query_value(
        &self,
        query: &str,
        parameters: Option<HashMap<String, MySqlParameterValue>>,
        #[allow(clippy::needless_pass_by_value)] column: Option<ColumnArgument>,
        associative_arrays: Option<bool>,
    ) -> anyhow::Result<Zval> {
        let (query, values) = self.render_query(query, parameters)?;
        let row = RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_one(&self.pool))
            .map_err(|err| anyhow!("{err}\n\nQuery: {query}\n\nValues: {values:?}\n\n"))?;
        let column_idx: usize = match column {
            Some(ColumnArgument::Index(i)) => i,
            Some(ColumnArgument::Name(column_name)) => {
                if let Some((column_idx, _)) = row
                    .columns()
                    .iter()
                    .enumerate()
                    .find(|(_, column)| column.name().eq(column_name))
                {
                    column_idx
                } else {
                    bail!("Column {} not found", column_name);
                }
            }
            None => 0,
        };
        row.column_value_into_zval(
            row.try_column(column_idx)?,
            associative_arrays.unwrap_or(self.options.associative_arrays),
        )
    }

    /// Executes an SQL query and returns a single column across all rows.
    ///
    /// # Arguments
    /// - `query`: SQL query string
    /// - `parameters`: Optional array of indexed/named parameters to bind.
    /// - `column`: Optional column index or name to retrieve. Defaults to the first column.
    /// - `associative_arrays`: Whether to use associative arrays for complex values (optional)
    ///
    /// # Returns
    /// An array containing values from the specified column
    ///
    /// # Exceptions
    /// Throws an exception if the query fails or conversion fails
    pub fn query_column(
        &self,
        query: &str,
        parameters: Option<HashMap<String, MySqlParameterValue>>,
        #[allow(clippy::needless_pass_by_value)] column: Option<ColumnArgument>,
        associative_arrays: Option<bool>,
    ) -> anyhow::Result<Vec<Zval>> {
        let (query, values) = self.render_query(query, parameters)?;
        let mut it = RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_all(&self.pool))
            .map_err(|err| anyhow!("{err}\n\nQuery: {query}\n\nValues: {values:?}\n\n"))?
            .into_iter()
            .peekable();
        let Some(row) = it.peek() else {
            return Ok(vec![]);
        };
        let column_idx: usize = match column {
            Some(ColumnArgument::Index(i)) => {
                if row.try_column(i).is_err() {
                    bail!("Column {} not found", i);
                }
                i
            }
            Some(ColumnArgument::Name(column_name)) => {
                if let Some((column_idx, _)) = row
                    .columns()
                    .iter()
                    .enumerate()
                    .find(|(_, column)| column.name().eq(column_name))
                {
                    column_idx
                } else {
                    bail!("Column {} not found", column_name);
                }
            }
            None => 0,
        };
        it.map(|row| {
            row.column_value_into_zval(
                row.column(column_idx),
                associative_arrays.unwrap_or(self.options.associative_arrays),
            )
        })
        .try_collect()
    }

    /// Executes an SQL query and returns a single column from the first row or null.
    ///
    /// # Arguments
    /// - `query`: SQL query string
    /// - `parameters`: Optional array of indexed/named parameters to bind.
    /// - `column`: Optional column index or name to retrieve. Defaults to the first column.
    /// - `associative_arrays`: Whether to use associative arrays for complex values (optional)
    ///
    /// # Returns
    /// A single column value as `Zval` or null if no row is found
    ///
    /// # Exceptions
    /// Throws an exception if the query fails, column doesn't exist, or conversion fails
    pub fn query_maybe_value(
        &self,
        query: &str,
        parameters: Option<HashMap<String, MySqlParameterValue>>,
        #[allow(clippy::needless_pass_by_value)] column: Option<ColumnArgument>,
        associative_arrays: Option<bool>,
    ) -> anyhow::Result<Zval> {
        let (query, values) = self.render_query(query, parameters)?;
        Ok(RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_one(&self.pool))
            .map(Some)
            .or_else(|err: Error| match err {
                Error::RowNotFound => Ok(None),
                _ => Err(anyhow!("{err}\n\nQuery: {query}\n\nValues: {values:?}\n\n")),
            })?
            .map(|row| {
                let column_idx: usize = match column {
                    Some(ColumnArgument::Index(i)) => i,
                    Some(ColumnArgument::Name(column_name)) => {
                        if let Some((column_idx, _)) = row
                            .columns()
                            .iter()
                            .enumerate()
                            .find(|(_, column)| column.name().eq(column_name))
                        {
                            column_idx
                        } else {
                            bail!("Column {} not found", column_name);
                        }
                    }
                    None => 0,
                };
                row.column_value_into_zval(
                    row.try_column(column_idx)?,
                    associative_arrays.unwrap_or(self.options.associative_arrays),
                )
            })
            .transpose()?
            .unwrap_or_else(Zval::null))
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
    pub fn query_row(
        &self,
        query: &str,
        parameters: Option<HashMap<String, MySqlParameterValue>>,
        associative_arrays: Option<bool>,
    ) -> anyhow::Result<Zval> {
        let (query, values) = self.render_query(query, parameters)?;
        RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_one(&self.pool))?
            .into_zval(associative_arrays.unwrap_or(self.options.associative_arrays))
    }

    /// Executes an SQL query and returns a single row if available, or `null` if no rows are returned.
    ///
    /// # Arguments
    /// - `query`: SQL query string to execute.
    /// - `parameters`: Optional array of indexed/named parameters to bind.
    /// - `associative_arrays`: Whether to return the row as an associative array (`true`)
    ///   or as a PHP object (`false`). If `None`, the default configuration is used.
    ///
    /// # Returns
    /// A PHP value representing the result row or `null` if no row matched the query.
    ///
    /// # Exceptions
    /// Throws an exception if:
    /// - the query is invalid or cannot be executed;
    /// - parameter binding fails due to incorrect types or unsupported values;
    /// - the row cannot be converted into a PHP value (e.g., unsupported database types).
    pub fn query_maybe_row(
        &self,
        query: &str,
        parameters: Option<HashMap<String, MySqlParameterValue>>,
        associative_arrays: Option<bool>,
    ) -> anyhow::Result<Zval> {
        let (query, values) = self.render_query(query, parameters)?;
        Ok(RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_one(&self.pool))
            .map(Some)
            .or_else(|err: Error| match err {
                Error::RowNotFound => Ok(None),
                _ => Err(anyhow!("{:?}", err)),
            })?
            .map(|x| x.into_zval(associative_arrays.unwrap_or(self.options.associative_arrays)))
            .transpose()?
            .unwrap_or_else(Zval::null))
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
        parameters: Option<HashMap<String, MySqlParameterValue>>,
        associative_arrays: Option<bool>,
    ) -> anyhow::Result<Vec<Zval>> {
        let (query, values) = self.render_query(query, parameters)?;
        let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);
        RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_all(&self.pool))?
            .into_iter()
            .map(|row| row.into_zval(assoc))
            .try_collect()
    }

    /// Returns the rendered query and its parameters.
    ///
    /// This method does not execute the query but returns the SQL string with placeholders
    /// and the bound parameter values for debugging or logging purposes.
    ///
    /// # Arguments
    /// - `query`: SQL query string
    /// - `parameters`: Optional array of indexed/named parameters to bind.
    ///
    /// # Returns
    /// An array where the first element is the rendered SQL query (string), and the second is an array of bound values
    ///
    /// # Exceptions
    /// Throws an exception if the query can't be parsed, rendered, or if parameters
    /// cannot be converted from PHP values.
    pub fn dry(
        &self,
        query: &str,
        parameters: Option<HashMap<String, MySqlParameterValue>>,
    ) -> anyhow::Result<Vec<Zval>> {
        let (query, values) = self.render_query(query, parameters)?;
        Ok(vec![
            query.into_zval(false).map_err(|err| anyhow!("{err:?}"))?,
            values.into_zval(false).map_err(|err| anyhow!("{err:?}"))?,
        ])
    }

    /// Executes an SQL query and returns a dictionary (map) indexed by the first column of each row.
    ///
    /// The resulting `HashMap<String, Zval>` maps the stringified value of the first column to the full row,
    /// which is converted into a PHP value (either associative array or object).
    ///
    /// # Parameters
    /// - `query`: SQL query string with placeholders (`$name`, `:name`, `?`, etc.).
    /// - `parameters`: Optional map of named parameters to bind into the query.
    /// - `associative_arrays`: If `Some(true)`, rows are returned as associative arrays; if `Some(false)`, as objects.
    ///   If `None`, the default behavior is taken from `self.options.associative_arrays`.
    ///
    /// # Returns
    /// A map from the first column (as `String`) to each corresponding row in the result set.
    ///
    /// # Errors
    /// Returns an error if:
    /// - the query cannot be rendered or executed;
    /// - parameter binding fails;
    /// - the first column of any row cannot be converted to a PHP string;
    /// - the result rows cannot be converted into PHP values.
    ///
    /// # Notes
    /// - The iteration order is preserved.
    /// - If the first column is `NULL`, a non-string type, or fails to convert to a PHP string, an error is returned.
    /// - This is useful for loading lookup tables, settings, or deduplicated result sets.
    pub fn query_dictionary(
        &self,
        query: &str,
        parameters: Option<HashMap<String, MySqlParameterValue>>,
        associative_arrays: Option<bool>,
    ) -> anyhow::Result<Zval> {
        let (query, values) = self.render_query(query, parameters)?;
        let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);
        RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_all(&self.pool))?
            .into_iter()
            .map(|row| {
                Ok((
                    row.column_value_into_zval(row.column(0), false)?,
                    row.into_zval(assoc)?,
                ))
            })
            .try_fold(zend_array::new(), fold_into_zend_hashmap)?
            .into_zval(false)
            .map_err(|err| anyhow!("{err:?}"))
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
    ///   If `None`, falls back to the value in `self.options.associative_arrays`.
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
    /// - Row order within each group is preserved.
    /// - The outer dictionary order is preserved.
    /// - Use this method when your result naturally groups by a field, e.g., for building nested structures or aggregations.
    pub fn query_grouped_dictionary(
        &self,
        query: &str,
        parameters: Option<HashMap<String, MySqlParameterValue>>,
        associative_arrays: Option<bool>,
    ) -> anyhow::Result<Zval> {
        let (query, values) = self.render_query(query, parameters)?;
        let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);

        RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_all(&self.pool))
            .map_err(|err| anyhow!("{err}\n\nQuery: {query}\n\nValues: {values:?}\n\n"))?
            .into_iter()
            .map(|row| {
                Ok((
                    row.column_value_into_zval(row.column(0), false)?,
                    row.into_zval(assoc)?,
                ))
            })
            .try_fold(zend_array::new(), fold_into_zend_hashmap_grouped)?
            .into_zval(false)
            .map_err(|err| anyhow!("{err:?}"))
    }

    /// Executes the given SQL query and returns a grouped dictionary where:
    /// - the **key** is the value of the **first column** (must be scalar),
    /// - the **value** is a `Vec<Zval>` containing values from the **second column** for each row with that key.
    ///
    /// This method is useful for aggregating results by a common key.
    ///
    /// If the first column cannot be converted to a PHP string, an error is returned.
    ///
    /// # Arguments
    ///
    /// - `query`: SQL string with optional placeholders.
    /// - `parameters`: Optional map of parameters to bind to the query.
    /// - `associative_arrays`: Optional override for array/object return mode (e.g., when dealing with JSON).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The SQL execution fails,
    /// - The first column cannot be converted to a string,
    /// - Binding or decoding values fails.
    ///
    /// # Example SQL
    ///
    /// ```sql
    /// SELECT department, name FROM employees
    /// ```
    ///
    /// Might return:
    ///
    /// ```php
    /// [
    ///   "Sales" => ["Alice", "Bob"],
    ///   "Engineering" => ["Carol"]
    /// ]
    /// ```
    pub fn query_grouped_column_dictionary(
        &self,
        query: &str,
        parameters: Option<HashMap<String, MySqlParameterValue>>,
        associative_arrays: Option<bool>,
    ) -> anyhow::Result<Zval> {
        let (query, values) = self.render_query(query, parameters)?;
        let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);
        RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_all(&self.pool))
            .map_err(|err| anyhow!("{err}\n\nQuery: {query}\n\nValues: {values:?}\n\n"))?
            .into_iter()
            .map(|row| {
                Ok((
                    row.column_value_into_zval(row.column(0), false)?,
                    row.column_value_into_zval(row.column(1), assoc)?,
                ))
            })
            .try_fold(zend_array::new(), fold_into_zend_hashmap_grouped)?
            .into_zval(false)
            .map_err(|err| anyhow!("{err:?}"))
    }

    /// Executes an SQL query and returns a dictionary mapping the first column to the second column.
    ///
    /// This method expects each row of the result to contain **at least two columns**.
    /// It constructs a `HashMap<String, Zval>` where:
    /// - the **key** is the value of the first column, converted to a PHP string;
    /// - the **value** is the second column, converted to a PHP value (array or object depending on `associative_arrays`).
    ///
    /// # Parameters
    /// - `query`: SQL query string with optional placeholders (e.g., `$name`, `:name`, `?`, etc.).
    /// - `parameters`: Optional map of named parameters to bind into the query.
    /// - `associative_arrays`: Whether to render complex values as associative arrays (`true`) or objects (`false`).
    ///   If `None`, the default behavior is taken from `self.options.associative_arrays`.
    ///
    /// # Returns
    /// A dictionary (`HashMap<String, Zval>`) mapping first-column keys to second-column values.
    ///
    /// # Errors
    /// Returns an error if:
    /// - the SQL query is invalid, fails to render, or cannot be executed;
    /// - parameter binding fails;
    /// - the first column of any row is `NULL` or cannot be converted into a PHP string;
    /// - the second column cannot be converted to a PHP value.
    ///
    /// # Notes
    /// - The result must contain **at least two columns**, otherwise a runtime panic or undefined behavior may occur.
    /// - The order of items in the resulting map is preserved.
    /// - Useful for loading key–value configurations or lookup tables.
    pub fn query_column_dictionary(
        &self,
        query: &str,
        parameters: Option<HashMap<String, MySqlParameterValue>>,
        associative_arrays: Option<bool>,
    ) -> anyhow::Result<Zval> {
        let (query, values) = self.render_query(query, parameters)?;
        let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);
        RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_all(&self.pool))
            .map_err(|err| anyhow!("{err}\n\nQuery: {query}\n\nValues: {values:?}\n\n"))?
            .into_iter()
            .map(|row| {
                Ok((
                    row.column_value_into_zval(row.column(0), false)?,
                    row.column_value_into_zval(row.column(1), assoc)?,
                ))
            })
            .try_fold(zend_array::new(), fold_into_zend_hashmap)?
            .into_zval(false)
            .map_err(|err| anyhow!("{err:?}"))
    }
}

/// Binds a list of `Value` arguments to an `SQLx` query.
fn bind_values<'a, D: Database>(
    query: Query<'a, D, <D>::Arguments<'a>>,
    values: &'a [MySqlParameterValue],
) -> Query<'a, D, <D>::Arguments<'a>>
where
    f64: Type<D>,
    f64: Encode<'a, D>,
    i64: Type<D>,
    i64: Encode<'a, D>,
    bool: Type<D>,
    bool: Encode<'a, D>,
    String: Type<D>,
    String: Encode<'a, D>,
{
    fn walker<'a, D: Database>(
        q: Query<'a, D, <D>::Arguments<'a>>,
        value: &'a MySqlParameterValue,
    ) -> Query<'a, D, <D>::Arguments<'a>>
    where
        f64: Type<D>,
        f64: Encode<'a, D>,
        i64: Type<D>,
        i64: Encode<'a, D>,
        bool: Type<D>,
        bool: Encode<'a, D>,
        String: Type<D>,
        String: Encode<'a, D>,
    {
        match value {
            MySqlParameterValue::Str(s) => q.bind(s),
            MySqlParameterValue::Int(s) => q.bind(s),
            MySqlParameterValue::Bool(s) => q.bind(s),
            MySqlParameterValue::Float(s) => q.bind(s),
            MySqlParameterValue::Array(s) => s.iter().fold(q, walker),
            // @TODO: values()?
            MySqlParameterValue::Object(s) => s.values().fold(q, walker),
            MySqlParameterValue::RenderedByClause(_)
            | MySqlParameterValue::RenderedSelectClause(_) => unimplemented!(),
        }
    }

    values.iter().fold(query, walker)
}
