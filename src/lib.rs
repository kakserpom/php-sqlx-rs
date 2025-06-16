#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]
#![cfg_attr(windows, feature(abi_vectorcall))]

mod ast;
mod tests;

use crate::ast::{Ast, Value};
use anyhow::{anyhow, bail};
use dashmap::DashMap;
use ext_php_rs::binary::Binary;
use ext_php_rs::boxed::ZBox;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::ffi::{zend_array, zend_object};
use ext_php_rs::{prelude::*, types::Zval};
use itertools::Itertools;
use ordermap::OrderMap;
use sqlx::postgres::{PgColumn, PgPoolOptions, PgRow};
use sqlx::{Column, Row};
use sqlx_core::Error;
use sqlx_core::database::Database;
use sqlx_core::decode::Decode;
use sqlx_core::encode::Encode;
use sqlx_core::query::Query;
use sqlx_core::type_info::TypeInfo;
use sqlx_core::types::Type;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use threadsafe_lru::LruCache;
use tokio::runtime::Runtime;

/// Global runtime for executing async `SQLx` queries from sync context.
static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());
static PERSISTENT_DRIVER_REGISTRY: LazyLock<DashMap<String, Arc<DriverInner>>> =
    LazyLock::new(DashMap::new);

const DEFAULT_AST_CACHE_SHARD_COUNT: usize = 8;
const DEFAULT_AST_CACHE_SHARD_SIZE: usize = 256;
#[php_class(name = "Sqlx\\OrderBy")]
pub struct OrderBy {
    pub(crate) defined_fields: HashMap<String, Option<String>>,
}

#[derive(ZvalConvert, Debug)]
pub enum OrderFieldDefinition {
    Full(Vec<String>),
    Short(String),
}
impl OrderBy {
    /// Ascending order (A to Z)
    const _ASC: &'static str = "ASC";
    /// Descending order (Z to A)
    const _DESC: &'static str = "DESC";
}

impl OrderBy {
    pub fn new<K, V>(defined_fields: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            defined_fields: defined_fields
                .into_iter()
                .map(|(key, value)| {
                    let key: String = key.into();
                    let value: String = value.into();
                    if key.parse::<u32>().is_ok() {
                        (value, None)
                    } else {
                        (key, Some(value))
                    }
                })
                .collect(),
        }
    }
}

#[php_impl]
impl OrderBy {
    /// Ascending order (A to Z)
    const ASC: &'static str = "ASC";
    /// Descending order (Z to A)
    const DESC: &'static str = "DESC";

    /// Constructs an OrderBy helper with allowed sortable fields.
    ///
    /// # Arguments
    /// - `defined_fields`: Map of allowed sort fields (key = user input, value = SQL expression)
    ///
    /// # Example
    /// ```php
    /// $order_by = new Sqlx\OrderBy([
    ///     "name",
    ///     "age",
    ///     "total_posts" => "COUNT(posts.*)"
    /// ]);
    /// ```

    pub fn __construct(defined_fields: HashMap<String, String>) -> anyhow::Result<Self> {
        Ok(OrderBy::new(defined_fields))
    }

    /// __invoke magic for apply()

    #[must_use]
    pub fn __invoke(&self, order_by: Vec<OrderFieldDefinition>) -> RenderedOrderBy {
        self.internal_apply(order_by)
    }

    /// Applies ordering rules to a user-defined input.
    ///
    /// # Arguments
    /// - `order_by`: List of fields (as strings or [field, direction] arrays)
    ///
    /// # Returns
    /// A `RenderedOrderBy` object containing validated SQL ORDER BY clauses
    /// The returning value is to be used as a placeholder value
    ///
    /// # Exceptions
    /// This method does not return an error but silently ignores unknown fields.
    /// Use validation separately if strict input is required.
    #[must_use]
    pub fn apply(&self, order_by: Vec<OrderFieldDefinition>) -> RenderedOrderBy {
        self.internal_apply(order_by)
    }
}
impl OrderBy {
    #[must_use]
    pub fn internal_apply(&self, order_by: Vec<OrderFieldDefinition>) -> RenderedOrderBy {
        RenderedOrderBy {
            __inner: order_by
                .into_iter()
                .filter_map(|definition| {
                    let (field, dir) = match definition {
                        OrderFieldDefinition::Short(name) => (name, OrderBy::ASC),
                        OrderFieldDefinition::Full(vec) => (
                            vec.first()?.clone(),
                            match vec.get(1) {
                                Some(str) if str.trim().eq_ignore_ascii_case("DESC") => {
                                    OrderBy::DESC
                                }
                                _ => OrderBy::ASC,
                            },
                        ),
                    };
                    let field = field.trim();
                    if let Some(definition) = self.defined_fields.get(field) {
                        if let Some(right_side) = definition {
                            Some(format!("{right_side} {dir}"))
                        } else {
                            Some(format!("`{field}` {dir}"))
                        }
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}
#[derive(ZvalConvert)]
pub struct ApplyOrderBy {
    inner: Vec<Vec<String>>,
}
/// A rendered ORDER BY clause result for use in query generation.
#[derive(Clone, PartialEq, Debug, ZvalConvert)]
pub struct RenderedOrderBy {
    // @TODO: make it impossible to alter RenderedOrderBy from PHP side
    pub(crate) __inner: Vec<String>,
}

impl RenderedOrderBy {
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.__inner.is_empty()
    }
}

/// A database driver using SQLx with query helpers and AST cache.
///
/// This class supports prepared queries, persistent connections, and augmented SQL.
#[php_class(name = "Sqlx\\Driver")]
pub struct Driver {
    pub driver_inner: Arc<DriverInner>,
}
pub struct DriverInner {
    pub pool: sqlx::PgPool,
    pub ast_cache: LruCache<String, Ast>,
    pub options: DriverOptions,
}

#[derive(Debug, ZvalConvert)]
pub enum ColumnArgument<'a> {
    Index(usize),
    Name(&'a str),
}

impl DriverInner {
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
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<u64> {
        let (query, values) = self.render_query(query, parameters)?;
        Ok(RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).execute(&self.pool))?
            .rows_affected())
    }

    /// Render the final SQL query and parameters using the AST cache.
    fn render_query(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<(String, Vec<Value>)> {
        let parameters = parameters.unwrap_or_default();
        if let Some(ast) = self.ast_cache.get(query) {
            ast.render(parameters)
        } else {
            let ast = Ast::parse(query).unwrap();
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
        parameters: Option<HashMap<String, Value>>,
        #[allow(clippy::needless_pass_by_value)] column: Option<ColumnArgument>,
        associative_arrays: Option<bool>,
    ) -> anyhow::Result<Zval> {
        let (query, values) = self.render_query(query, parameters)?;
        let row =
            RUNTIME.block_on(bind_values(sqlx::query(&query), &values).fetch_one(&self.pool))?;
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
        (&row).column_value_into_zval(
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
        parameters: Option<HashMap<String, Value>>,
        #[allow(clippy::needless_pass_by_value)] column: Option<ColumnArgument>,
        associative_arrays: Option<bool>,
    ) -> anyhow::Result<Vec<Zval>> {
        let (query, values) = self.render_query(query, parameters)?;
        let mut it = RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_all(&self.pool))?
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
            (&row).column_value_into_zval(
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
        parameters: Option<HashMap<String, Value>>,
        #[allow(clippy::needless_pass_by_value)] column: Option<ColumnArgument>,
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
                (&row).column_value_into_zval(
                    row.try_column(column_idx)?,
                    associative_arrays.unwrap_or(self.options.associative_arrays),
                )
            })
            .transpose()?
            .unwrap_or_else(|| {
                let mut null = Zval::new();
                null.set_null();
                null
            }))
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
        parameters: Option<HashMap<String, Value>>,
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
    /// - the row cannot be converted into a PHP value (e.g., unsupported Postgres types).
    pub fn query_maybe_row(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
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
            .unwrap_or_else(|| {
                let mut null = Zval::new();
                null.set_null();
                null
            }))
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
        associative_arrays: Option<bool>,
    ) -> anyhow::Result<Zval> {
        let (query, values) = self.render_query(query, parameters)?;
        let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);
        let mut it = RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_all(&self.pool))?
            .into_iter()
            .map(|row| {
                if let Some(key) = (&row)
                    .column_value_into_zval(row.column(0), false)?
                    .string()
                {
                    Ok((key, row.into_zval(assoc)?))
                } else {
                    bail!("First column must be convertible to string")
                }
            });

        Ok(if assoc {
            it.try_fold(
                zend_array::new(),
                |mut array, item| -> anyhow::Result<ZBox<zend_array>> {
                    let (key, value) = item?;
                    array
                        .insert(&key, value)
                        .map_err(|err| anyhow!("{err:?}"))?;
                    Ok(array)
                },
            )?
            .into_zval(false)
            .map_err(|err| anyhow!("{err:?}"))?
        } else {
            it.try_fold(
                zend_object::new_stdclass(),
                |mut object, item| -> anyhow::Result<ZBox<zend_object>> {
                    let (key, value) = item?;
                    object
                        .set_property(&key, value)
                        .map_err(|err| anyhow!("{err:?}"))?;
                    Ok(object)
                },
            )?
            .into_zval(false)
            .map_err(|err| anyhow!("{err:?}"))?
        })
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
        parameters: Option<HashMap<String, Value>>,
        associative_arrays: Option<bool>,
    ) -> anyhow::Result<Zval> {
        let (query, values) = self.render_query(query, parameters)?;
        let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);
        RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_all(&self.pool))?
            .into_iter()
            .map(|row| {
                if let Some(key) = (&row)
                    .column_value_into_zval(row.column(0), false)?
                    .string()
                {
                    Ok((key, row.into_zval(assoc)?))
                } else {
                    bail!("First column must be convertible to string")
                }
            })
            .try_fold(
                OrderMap::<String, Vec<Zval>>::new(),
                |mut map, item| -> anyhow::Result<_> {
                    let (key, value) = item?;
                    map.entry(key).or_default().push(value);
                    Ok(map)
                },
            )?
            .into_iter()
            .try_fold(
                zend_array::new(),
                |mut array, (key, value)| -> anyhow::Result<ZBox<zend_array>> {
                    array
                        .insert(&key, value)
                        .map_err(|err| anyhow!("{err:?}"))?;
                    Ok(array)
                },
            )?
            .into_zval(false)
            .map_err(|err| anyhow!("{err:?}"))
    }

    /// Executes the given SQL query and returns a grouped dictionary where:
    /// - the **key** is the value of the **first column** (must be convertible to string),
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
        parameters: Option<HashMap<String, Value>>,
        associative_arrays: Option<bool>,
    ) -> anyhow::Result<Zval> {
        let (query, values) = self.render_query(query, parameters)?;
        let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);
        RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_all(&self.pool))?
            .into_iter()
            .map(|row| {
                if let Some(key) = (&row)
                    .column_value_into_zval(row.column(0), false)?
                    .string()
                {
                    Ok((key, (&row).column_value_into_zval(row.column(1), assoc)?))
                } else {
                    bail!("First column must be convertible to string")
                }
            })
            .try_fold(
                OrderMap::<String, Vec<Zval>>::new(),
                |mut map, item| -> anyhow::Result<_> {
                    let (key, value) = item?;
                    map.entry(key).or_default().push(value);
                    Ok(map)
                },
            )?
            .into_iter()
            .try_fold(
                zend_array::new(),
                |mut array, (key, value)| -> anyhow::Result<ZBox<zend_array>> {
                    array
                        .insert(&key, value)
                        .map_err(|err| anyhow!("{err:?}"))?;
                    Ok(array)
                },
            )?
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
        parameters: Option<HashMap<String, Value>>,
        associative_arrays: Option<bool>,
    ) -> anyhow::Result<Zval> {
        let (query, values) = self.render_query(query, parameters)?;
        let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);
        let mut it = RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_all(&self.pool))?
            .into_iter()
            .map(|row| {
                if let Some(key) = (&row)
                    .column_value_into_zval(row.column(0), false)?
                    .string()
                {
                    Ok((key, (&row).column_value_into_zval(row.column(1), assoc)?))
                } else {
                    bail!("First column must be convertible to string")
                }
            });
        Ok(if assoc {
            it.try_fold(
                zend_array::new(),
                |mut array, item| -> anyhow::Result<ZBox<zend_array>> {
                    let (key, value) = item?;
                    array
                        .insert(&key, value)
                        .map_err(|err| anyhow!("{err:?}"))?;
                    Ok(array)
                },
            )?
            .into_zval(false)
            .map_err(|err| anyhow!("{err:?}"))?
        } else {
            it.try_fold(
                zend_object::new_stdclass(),
                |mut object, item| -> anyhow::Result<ZBox<zend_object>> {
                    let (key, value) = item?;
                    object
                        .set_property(&key, value)
                        .map_err(|err| anyhow!("{err:?}"))?;
                    Ok(object)
                },
            )?
            .into_zval(false)
            .map_err(|err| anyhow!("{err:?}"))?
        })
    }
}

/// Trait to convert a row into a PHP value.
trait RowToZval: Row {
    /// Convert the row into a PHP associative array.
    fn into_zval(self, associative_arrays: bool) -> anyhow::Result<Zval>;
}

/// Converts a JSON value into a PHP value, respecting associative array settings.
///
/// # Arguments
/// - `value`: A `serde_json::Value` to convert
/// - `associative_arrays`: Whether to convert objects into PHP associative arrays or `stdClass`
///
/// # Returns
/// Converted `Zval` or an error if conversion fails
fn json_into_zval(value: serde_json::Value, associative_arrays: bool) -> anyhow::Result<Zval> {
    match value {
        serde_json::Value::String(str) => str
            .into_zval(false)
            .map_err(|err| anyhow!("String conversion: {err:?}")),
        serde_json::Value::Number(number) => number
            .to_string()
            .into_zval(false)
            .map_err(|err| anyhow!("Number conversion: {err:?}")),
        serde_json::Value::Bool(bool) => bool
            .into_zval(false)
            .map_err(|err| anyhow!("Bool conversion: {err:?}")),
        serde_json::Value::Null => {
            let mut null = Zval::new();
            null.set_null();
            Ok(null)
        }
        serde_json::Value::Array(array) => Ok(array
            .into_iter()
            .map(|x| json_into_zval(x, associative_arrays))
            .collect::<anyhow::Result<Vec<Zval>>>()?
            .into_zval(false)
            .map_err(|err| anyhow!("Array conversion: {err:?}"))?),
        serde_json::Value::Object(object) => {
            if associative_arrays {
                Ok(object
                    .into_iter()
                    .map(|(key, value)| Ok((key, json_into_zval(value, associative_arrays)?)))
                    .collect::<anyhow::Result<HashMap<String, Zval>>>()?
                    .into_zval(false)
                    .map_err(|err| anyhow!("Object conversion: {err:?}"))?)
            } else {
                Ok(object
                    .into_iter()
                    .try_fold(
                        zend_object::new_stdclass(),
                        |mut std_object, (key, value)| {
                            std_object
                                .set_property(&key, json_into_zval(value, associative_arrays))
                                .map(|()| std_object)
                                .map_err(|err| anyhow!("Object conversion: {:?}", err))
                        },
                    )?
                    .into_zval(false)
                    .map_err(|err| anyhow!("Object conversion: {err:?}"))?)
            }
        }
    }
}
/// Trait to convert a column value into a PHP value.
trait ColumnToZval {
    /// Converts a specific column from a row to a PHP value.
    ///
    /// # Arguments
    /// - `column`: Reference to the column in the row.
    /// - `associative_arrays`: Whether to render complex types as associative arrays
    ///
    /// # Returns
    /// A PHP-compatible `Zval` value
    fn column_value_into_zval(
        &self,
        column: &PgColumn,
        associative_arrays: bool,
    ) -> anyhow::Result<Zval>;
}
impl ColumnToZval for &PgRow {
    fn column_value_into_zval(
        &self,
        column: &PgColumn,
        associative_arrays: bool,
    ) -> anyhow::Result<Zval> {
        fn try_cast_into_zval<'r, T>(row: &'r PgRow, name: &str) -> anyhow::Result<Zval>
        where
            T: Decode<'r, <PgRow as Row>::Database> + Type<<PgRow as Row>::Database>,
            T: IntoZval,
        {
            row.try_get::<'r, T, _>(name)
                .map_err(|err| anyhow!("{err:?}"))?
                .into_zval(false)
                .map_err(|err| anyhow!("{err:?}"))
        }

        let column_name = column.name();
        let column_type = column.type_info().name();
        Ok(match column_type {
            "BOOL" => try_cast_into_zval::<bool>(self, column_name)?,
            "BYTEA" | "BINARY" => (self)
                .try_get::<&[u8], _>(column_name)
                .map_err(|err| anyhow!("{err:?}"))
                .map(|x| x.iter().copied().collect::<Binary<_>>())?
                .into_zval(false)
                .map_err(|err| anyhow!("{err:?}"))?,
            "CHAR" | "NAME" | "TEXT" | "BPCHAR" | "VARCHAR" => {
                try_cast_into_zval::<String>(self, column_name)?
            }
            "INT2" => try_cast_into_zval::<i16>(self, column_name)?,
            "INT4" | "INT" => try_cast_into_zval::<i32>(self, column_name)?,
            "INT8" => try_cast_into_zval::<i64>(self, column_name)?,
            "OID" => try_cast_into_zval::<i32>(self, column_name)?,
            "FLOAT4" => try_cast_into_zval::<f32>(self, column_name)?,
            "FLOAT8" | "F64" => try_cast_into_zval::<f64>(self, column_name)?,
            "NUMERIC" | "MONEY" => try_cast_into_zval::<String>(self, column_name)?,
            "UUID" => try_cast_into_zval::<String>(self, column_name)?,
            "JSON" | "JSONB" => self
                .try_get::<serde_json::Value, _>(column_name)
                .map_err(|err| anyhow!("{err:?}"))
                .map(|x| json_into_zval(x, associative_arrays))?
                .into_zval(false)
                .map_err(|err| anyhow!("{err:?}"))?,
            "_JSON" | "_JSONB" => self
                .try_get::<Vec<serde_json::Value>, _>(column_name)
                .map_err(|err| anyhow!("{err:?}"))
                .map(|x| {
                    x.into_iter()
                        .map(|x| json_into_zval(x, associative_arrays))
                        .collect::<Vec<_>>()
                })?
                .into_zval(false)
                .map_err(|err| anyhow!("{err:?}"))?,
            "DATE" | "TIME" | "TIMESTAMP" | "TIMESTAMPTZ" | "INTERVAL" | "TIMETZ" => {
                try_cast_into_zval::<String>(self, column_name)?
            }
            "INET" | "CIDR" | "MACADDR" | "MACADDR8" => {
                try_cast_into_zval::<String>(self, column_name)?
            }
            "BIT" | "VARBIT" => try_cast_into_zval::<String>(self, column_name)?,
            "POINT" | "LSEG" | "PATH" | "BOX" | "POLYGON" | "LINE" | "CIRCLE" => {
                try_cast_into_zval::<String>(self, column_name)?
            }
            "INT4RANGE" | "NUMRANGE" | "TSRANGE" | "TSTZRANGE" | "DATERANGE" | "INT8RANGE" => {
                try_cast_into_zval::<String>(self, column_name)?
            }
            "RECORD" => try_cast_into_zval::<String>(self, column_name)?,
            "JSONPATH" => try_cast_into_zval::<String>(self, column_name)?,

            // массивы
            "_BOOL" => try_cast_into_zval::<Vec<bool>>(self, column_name)?,
            "_BYTEA" => try_cast_into_zval::<Vec<Vec<u8>>>(self, column_name)?,
            "_CHAR" | "_NAME" | "_TEXT" | "_BPCHAR" | "_VARCHAR" => {
                try_cast_into_zval::<Vec<String>>(self, column_name)?
            }
            "_INT2" => try_cast_into_zval::<Vec<i16>>(self, column_name)?,
            "_INT4" => try_cast_into_zval::<Vec<i32>>(self, column_name)?,
            "_INT8" => try_cast_into_zval::<Vec<i64>>(self, column_name)?,
            "_OID" => try_cast_into_zval::<Vec<i32>>(self, column_name)?,
            "_FLOAT4" => try_cast_into_zval::<Vec<f32>>(self, column_name)?,
            "_FLOAT8" => try_cast_into_zval::<Vec<f64>>(self, column_name)?,
            "_NUMERIC" | "_MONEY" => try_cast_into_zval::<Vec<String>>(self, column_name)?,
            "_UUID" => try_cast_into_zval::<Vec<String>>(self, column_name)?,
            "_DATE" | "_TIME" | "_TIMESTAMP" | "_TIMESTAMPTZ" | "_INTERVAL" | "_TIMETZ" => {
                try_cast_into_zval::<Vec<String>>(self, column_name)?
            }
            "_INET" | "_CIDR" | "_MACADDR" | "_MACADDR8" => {
                try_cast_into_zval::<Vec<String>>(self, column_name)?
            }
            "_BIT" | "_VARBIT" => try_cast_into_zval::<Vec<String>>(self, column_name)?,
            "_POINT" | "_LSEG" | "_PATH" | "_BOX" | "_POLYGON" | "_LINE" | "_CIRCLE" => {
                try_cast_into_zval::<Vec<String>>(self, column_name)?
            }
            "_INT4RANGE" | "_NUMRANGE" | "_TSRANGE" | "_TSTZRANGE" | "_DATERANGE"
            | "_INT8RANGE" => try_cast_into_zval::<Vec<String>>(self, column_name)?,
            "_RECORD" => try_cast_into_zval::<Vec<String>>(self, column_name)?,
            "_JSONPATH" => try_cast_into_zval::<Vec<String>>(self, column_name)?,

            _ => bail!("unsupported type: {column_type}"),
        })
    }
}
impl RowToZval for PgRow {
    fn into_zval(self, associative_arrays: bool) -> anyhow::Result<Zval> {
        if associative_arrays {
            Ok(self
                .columns()
                .iter()
                .try_fold(
                    zend_array::new(),
                    |mut array, column| -> anyhow::Result<ZBox<zend_array>> {
                        array
                            .insert(
                                column.name(),
                                (&self).column_value_into_zval(column, associative_arrays)?,
                            )
                            .map_err(|err| anyhow!("{err:?}"))?;
                        Ok(array)
                    },
                )?
                .into_zval(false)
                .map_err(|err| anyhow!("{err:?}"))?)
        } else {
            Ok(self
                .columns()
                .iter()
                .try_fold(zend_object::new_stdclass(), |mut object, column| {
                    object
                        .set_property(
                            column.name(),
                            (&self).column_value_into_zval(column, associative_arrays)?,
                        )
                        .map(|()| object)
                        .map_err(|err| anyhow!("{:?}", err))
                })?
                .into_zval(false)
                .map_err(|err| anyhow!("{:?}", err))?)
        }
    }
}

/// Binds a list of `Value` arguments to an `SQLx` query.
fn bind_values<'a, D: Database>(
    query: Query<'a, D, <D>::Arguments<'a>>,
    values: &'a [Value],
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
        value: &'a Value,
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
            Value::Str(s) => q.bind(s),
            Value::Int(s) => q.bind(s),
            Value::Bool(s) => q.bind(s),
            Value::Float(s) => q.bind(s),
            Value::Array(s) => s.iter().fold(q, walker),
            // @TODO: values()?
            Value::Object(s) => s.values().fold(q, walker),
            Value::RenderedOrderBy(_) => unimplemented!(),
        }
    }

    values.iter().fold(query, walker)
}

#[derive(ZvalConvert)]
pub enum DriverConstructorOptions {
    Url(String),
    Options(HashMap<String, Value>),
}

pub struct DriverOptions {
    url: Option<String>,
    ast_cache_shard_count: usize,
    ast_cache_shard_size: usize,
    persistent_name: Option<String>,
    associative_arrays: bool,
}
impl Default for DriverOptions {
    fn default() -> Self {
        Self {
            url: None,
            ast_cache_shard_count: DEFAULT_AST_CACHE_SHARD_COUNT,
            ast_cache_shard_size: DEFAULT_AST_CACHE_SHARD_SIZE,
            persistent_name: None,
            associative_arrays: false,
        }
    }
}

#[php_impl]
impl Driver {
    const OPT_URL: &'static str = "url";
    const OPT_AST_CACHE_SHARD_COUNT: &'static str = "ast_cache_shard_count";

    const OPT_AST_CACHE_SHARD_SIZE: &'static str = "ast_cache_shard_size";

    const OPT_PERSISTENT_NAME: &'static str = "persistent_name";
    const OPT_ASSOC_ARRAYS: &'static str = "assoc_arrays";

    /// Constructs a new SQLx driver instance.
    ///
    /// # Arguments
    /// - `options`: Connection URL as string or associative array with options:
    ///   - `url`: (string) database connection string (required)
    ///   - `ast_cache_shard_count`: (int) number of AST cache shards (default: 8)
    ///   - `ast_cache_shard_size`: (int) size per shard (default: 256)
    ///   - `persistent_name`: (string) name of persistent connection
    ///   - `assoc_arrays`: (bool) return associative arrays instead of objects
    pub fn __construct(options: DriverConstructorOptions) -> anyhow::Result<Self> {
        let options = match options {
            DriverConstructorOptions::Url(url) => DriverOptions {
                url: Some(url),
                ..Default::default()
            },
            DriverConstructorOptions::Options(kv) => DriverOptions {
                url: Some(
                    kv.get(Self::OPT_URL)
                        .ok_or_else(|| anyhow!("missing {}", Self::OPT_URL))
                        .and_then(|value| {
                            if let Value::Str(str) = value {
                                Ok(str.clone())
                            } else {
                                Err(anyhow!("{} must be a string", Self::OPT_URL))
                            }
                        })?,
                ),
                associative_arrays: kv.get(Self::OPT_ASSOC_ARRAYS).map_or(Ok(false), |value| {
                    if let Value::Bool(bool) = value {
                        Ok(*bool)
                    } else {
                        Err(anyhow!("{} must be a string", Self::OPT_ASSOC_ARRAYS))
                    }
                })?,
                ast_cache_shard_count: kv.get(Self::OPT_AST_CACHE_SHARD_COUNT).map_or(
                    Ok(DEFAULT_AST_CACHE_SHARD_COUNT),
                    |value| {
                        if let Value::Int(n) = value {
                            Ok(usize::try_from(*n)?)
                        } else {
                            Err(anyhow!(
                                "{} must be an integer",
                                Self::OPT_AST_CACHE_SHARD_COUNT
                            ))
                        }
                    },
                )?,
                ast_cache_shard_size: kv.get(Self::OPT_AST_CACHE_SHARD_SIZE).map_or(
                    Ok(DEFAULT_AST_CACHE_SHARD_SIZE),
                    |value| {
                        if let Value::Int(n) = value {
                            Ok(usize::try_from(*n)?)
                        } else {
                            Err(anyhow!(
                                "{} must be an integer",
                                Self::OPT_AST_CACHE_SHARD_SIZE
                            ))
                        }
                    },
                )?,
                persistent_name: match kv.get(Self::OPT_PERSISTENT_NAME) {
                    None => None,
                    Some(value) => {
                        if let Value::Str(str) = value {
                            Some(str.clone())
                        } else {
                            return Err(anyhow!(
                                "{} must be an integer",
                                Self::OPT_PERSISTENT_NAME
                            ));
                        }
                    }
                },
            },
        };

        if let Some(name) = options.persistent_name.as_ref() {
            if let Some(driver_inner) = PERSISTENT_DRIVER_REGISTRY.get(name) {
                return Ok(Self {
                    driver_inner: driver_inner.clone(),
                });
            }
        }
        let persistent_name = options.persistent_name.clone();
        let pool = crate::RUNTIME.block_on(
            PgPoolOptions::new().max_connections(5).connect(
                options
                    .url
                    .clone()
                    .ok_or_else(|| anyhow!("URL must be set"))?
                    .as_str(),
            ),
        )?;
        let driver_inner = Arc::new(DriverInner {
            pool,
            ast_cache: LruCache::new(options.ast_cache_shard_count, options.ast_cache_shard_size),
            options,
        });
        if let Some(name) = persistent_name {
            PERSISTENT_DRIVER_REGISTRY.insert(name, driver_inner.clone());
        }
        Ok(Self { driver_inner })
    }

    /// Returns whether results are returned as associative arrays.
    ///
    /// If true, result rows are returned as PHP associative arrays (key-value pairs).
    /// If false, result rows are returned as PHP `stdClass` objects.
    #[getter]
    fn assoc_arrays(&self) -> bool {
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
        column: Option<ColumnArgument>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_value(query, parameters, column, None)
    }

    pub fn query_value_assoc(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
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
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_column_dictionary(query, parameters, Some(false))
    }

    /// Same as `queryGroupedColumnDictionary()`, but forces associative arrays
    /// for the second column if it's a JSON object.
    pub fn query_grouped_column_dictionary_assoc(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_grouped_column_dictionary(query, parameters, Some(true))
    }

    /// Same as `queryGroupedColumnDictionary()`, but forces PHP objects
    /// for the second column if it's a JSON object.
    pub fn query_grouped_column_dictionary_obj(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_grouped_column_dictionary(query, parameters, Some(false))
    }

    /// Executes an SQL query and returns a grouped dictionary where:
    /// - the key is the **first column** (must be convertible to string),
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
        parameters: Option<HashMap<String, Value>>,
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
    pub fn prepare(&self, query: &str) -> PreparedQuery {
        PreparedQuery {
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
        parameters: Option<HashMap<String, Value>>,
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
    pub fn insert(&self, table: &str, row: HashMap<String, Value>) -> anyhow::Result<u64> {
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
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Vec<Zval>> {
        self.driver_inner.dry(query, parameters)
    }
}

/// A reusable prepared SQL query with parameter support.
///
/// Created using `Driver::prepare()`, shares context with original driver.
#[php_class(name = "Sqlx\\PreparedQuery")]
pub struct PreparedQuery {
    query: String,
    driver_inner: Arc<DriverInner>,
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

/// Registers the PHP module.
#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
