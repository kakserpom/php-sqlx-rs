//! Inner database driver implementation macro for php-sqlx.
//!
//! This module provides the [`php_sqlx_impl_driver_inner!`] macro that generates the
//! core database driver logic with connection pooling, query execution, and transaction
//! management. Each database type uses this macro to create its inner driver.
//!
//! The inner driver handles:
//! - Connection pool management with SQLx
//! - AST-based query rendering with LRU caching
//! - Transaction stack for nested transaction support
//! - All query execution methods (`query_row`, `query_all`, etc.)
//!
//! This is separated from the outer driver (in `driver.rs`) to allow the PHP bindings
//! to wrap the inner driver with `Arc` for shared ownership across prepared queries
//! and query builders.

/// Generates the inner database driver implementation.
///
/// This macro creates the core driver struct with connection pooling, query execution,
/// and transaction management for a specific database backend.
///
/// # Arguments
///
/// - `$struct` - The Rust struct name for the inner driver (e.g., `PgInnerDriver`)
/// - `$database` - The SQLx database type (e.g., `Postgres`, `MySql`, `Mssql`)
///
/// # Generated Structure
///
/// The macro generates a struct with:
/// - `pool`: SQLx connection pool
/// - `ast_cache`: LRU cache for parsed AST queries
/// - `options`: Driver configuration
/// - `tx_stack`: Transaction stack for nested transaction support
/// - `settings`: AST rendering settings
#[macro_export]
macro_rules! php_sqlx_impl_driver_inner {
    ( $struct:ident, $database:ident ) => {
        use anyhow::{anyhow, bail};
        use ext_php_rs::{convert::IntoZval, ffi::zend_array, types::Zval};
        use itertools::Itertools;
        use sqlx_oldapi::{$database, Error, Column, Row, Transaction, pool::{Pool, PoolOptions}};
        use std::collections::BTreeMap;
        use std::sync::RwLock;
        use threadsafe_lru::LruCache;
        use $crate::{
            RUNTIME,
            ast::{Ast, Settings},
            conversion::Conversion,
            options::DriverInnerOptions,
            param_value::{ParameterValue, utils::bind_values},
            utils::{
                ident::is_valid_ident,
                hashmap_fold::{fold_into_zend_hashmap, fold_into_zend_hashmap_grouped},
                types::ColumnArgument,
            }
        };
        /// Core database driver containing connection pool and query execution logic.
        ///
        /// This struct is typically wrapped in `Arc` and shared across the outer driver,
        /// prepared queries, and query builders.
        pub struct $struct {
            /// SQLx connection pool for efficient connection reuse.
            pub pool: Pool<$database>,
            /// LRU cache for parsed SQL AST, reducing parse overhead for repeated queries.
            pub ast_cache: LruCache<String, Ast>,
            /// Driver configuration options.
            pub options: DriverInnerOptions,
            /// Stack of active transactions for nested transaction support.
            pub tx_stack: RwLock<Vec<Transaction<'static, $database>>>,
            /// AST rendering settings (placeholder style, collapsible IN, etc.).
            pub settings: Settings,
        }

        impl $struct {
            /// Creates a new inner driver with the given configuration.
            ///
            /// This establishes the connection pool and initializes the AST cache.
            /// The URL must be set in options or an error is returned.
            pub fn new(options: DriverInnerOptions) -> anyhow::Result<Self> {
                let mut pool_options = PoolOptions::<$database>::new()
                    .max_connections(options.max_connections.into())
                    .min_connections(options.min_connections)
                    .max_lifetime(options.max_lifetime)
                    .idle_timeout(options.idle_timeout)
                    .test_before_acquire(options.test_before_acquire);
                if let Some(acquire_timeout) = options.acquire_timeout {
                    pool_options = pool_options.acquire_timeout(acquire_timeout);
                }
                let url = options
                    .url
                    .clone()
                    .ok_or_else(|| anyhow!("URL must be set"))?;
                let pool = RUNTIME.block_on(pool_options.connect(url.as_str()))?;
                let mut settings = SETTINGS.clone();
                settings.collapsible_in_enabled = options.collapsible_in_enabled;
                Ok(Self {
                    tx_stack: RwLock::new(Vec::new()),
                    pool,
                    ast_cache: LruCache::new(
                        options.ast_cache_shard_count,
                        options.ast_cache_shard_size,
                    ),
                    settings,
                    options,
                })
            }

            /// Returns whether this driver is configured for read-only mode.
            ///
            /// Read-only mode is useful for replica connections where writes should be prevented.
            #[inline]
            pub fn is_readonly(&self) -> bool {
                self.options.readonly
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
            ) -> anyhow::Result<u64> {
                let (query, values) = self.render_query(query, parameters)?;

                Ok(if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                    let val = RUNTIME.block_on(
                        bind_values(sqlx_oldapi::query(&query), &values)?.execute(&mut *tx),
                    );
                    self.place_ongoing_transaction(tx);
                    val
                } else {
                    RUNTIME.block_on(
                        bind_values(sqlx_oldapi::query(&query), &values)?.execute(&self.pool),
                    )
                }
                .map_err(|err| anyhow!("{err}\n\nQuery: {query}\n\nValues: {values:?}\n\n"))?
                .rows_affected())
            }

            /// Renders the final SQL query and parameters using the AST cache.
            ///
            /// Looks up the query in the cache; if not found, parses it and caches the AST.
            /// Returns the rendered SQL string with positional placeholders and the parameter values.
            fn render_query(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> anyhow::Result<(String, Vec<ParameterValue>)> {
                let parameters = parameters.unwrap_or_default();
                if let Some(ast) = self.ast_cache.get(query) {
                    ast.render(parameters, &self.settings)
                } else {
                    let ast = Ast::parse(query, &self.settings)?;
                    let rendered = ast.render(parameters, &self.settings)?;
                    self.ast_cache.insert(query.to_owned(), ast);
                    Ok(rendered)
                }
            }

            /// Renders a query with all parameters inlined (no placeholders).
            ///
            /// This is used for debugging or logging purposes. The rendered query
            /// contains literal values instead of positional placeholders.
            fn render_query_inline(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> anyhow::Result<String> {
                let parameters = parameters.unwrap_or_default();

                let mut settings = self.settings.clone();
                settings.max_placeholders = 0;

                if let Some(ast) = self.ast_cache.get(query) {
                    let (query, _) = ast.render(parameters, &settings)?;
                    Ok(query)
                } else {
                    let ast = Ast::parse(query, &self.settings)?;
                    let (query, _) = ast.render(parameters, &settings)?;
                    self.ast_cache.insert(query.to_owned(), ast);
                    Ok(query)
                }
            }

            /// Parses a SQL query into an AST, using the cache if available.
            ///
            /// Returns a cached AST if the query was previously parsed, otherwise
            /// parses the query, caches the result, and returns it.
            pub fn parse_query(&self, query: &str) -> anyhow::Result<Ast> {
                if let Some(ast) = self.ast_cache.get(query) {
                    Ok(ast)
                } else {
                    let ast = Ast::parse(query, &self.settings)?;
                    self.ast_cache.insert(query.to_owned(), ast.clone());
                    Ok(ast)
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
                parameters: Option<BTreeMap<String, ParameterValue>>,
                #[allow(clippy::needless_pass_by_value)] column: Option<ColumnArgument>,
                associative_arrays: Option<bool>,
            ) -> anyhow::Result<Zval> {
                let (query, values) = self.render_query(query, parameters)?;
                let row = RUNTIME
                    .block_on(
                        bind_values(sqlx_oldapi::query(&query), &values)?.fetch_one(&self.pool),
                    )
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
                parameters: Option<BTreeMap<String, ParameterValue>>,
                #[allow(clippy::needless_pass_by_value)] column: Option<ColumnArgument>,
                associative_arrays: Option<bool>,
            ) -> anyhow::Result<Vec<Zval>> {
                let (query, values) = self.render_query(query, parameters)?;
                let mut it = RUNTIME
                    .block_on(
                        bind_values(sqlx_oldapi::query(&query), &values)?.fetch_all(&self.pool),
                    )
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
                parameters: Option<BTreeMap<String, ParameterValue>>,
                #[allow(clippy::needless_pass_by_value)] column: Option<ColumnArgument>,
                associative_arrays: Option<bool>,
            ) -> anyhow::Result<Zval> {
                let (query, values) = self.render_query(query, parameters)?;
                Ok(RUNTIME
                    .block_on(
                        bind_values(sqlx_oldapi::query(&query), &values)?.fetch_one(&self.pool),
                    )
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
                parameters: Option<BTreeMap<String, ParameterValue>>,
                associative_arrays: Option<bool>,
            ) -> anyhow::Result<Zval> {
                let (query, values) = self.render_query(query, parameters)?;
                RUNTIME
                    .block_on(
                        bind_values(sqlx_oldapi::query(&query), &values)?.fetch_one(&self.pool),
                    )?
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
                parameters: Option<BTreeMap<String, ParameterValue>>,
                associative_arrays: Option<bool>,
            ) -> anyhow::Result<Zval> {
                let (query, values) = self.render_query(query, parameters)?;
                Ok(RUNTIME
                    .block_on(
                        bind_values(sqlx_oldapi::query(&query), &values)?.fetch_one(&self.pool),
                    )
                    .map(Some)
                    .or_else(|err: Error| match err {
                        Error::RowNotFound => Ok(None),
                        _ => Err(anyhow!("{:?}", err)),
                    })?
                    .map(|x| {
                        x.into_zval(associative_arrays.unwrap_or(self.options.associative_arrays))
                    })
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
                parameters: Option<BTreeMap<String, ParameterValue>>,
                associative_arrays: Option<bool>,
            ) -> anyhow::Result<Vec<Zval>> {
                let (query, values) = self.render_query(query, parameters)?;
                let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);
                RUNTIME
                    .block_on(
                        bind_values(sqlx_oldapi::query(&query), &values)?.fetch_all(&self.pool),
                    )
                    .map_err(|err| anyhow!("{err}\n\nQuery: {query}\n\nValues: {values:?}\n\n"))?
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
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> anyhow::Result<Vec<Zval>> {
                let (query, values) = self.render_query(query, parameters)?;
                Ok(vec![
                    query.into_zval(false).map_err(|err| anyhow!("{err:?}"))?,
                    values.into_zval(false).map_err(|err| anyhow!("{err:?}"))?,
                ])
            }

            /// Returns the query with all parameters inlined for debugging.
            ///
            /// Unlike `dry()` which returns placeholders and values separately,
            /// this method produces a single SQL string with literal values embedded.
            pub fn dry_inline(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> anyhow::Result<String> {
                self.render_query_inline(query, parameters)
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
                parameters: Option<BTreeMap<String, ParameterValue>>,
                associative_arrays: Option<bool>,
            ) -> anyhow::Result<Zval> {
                let (query, values) = self.render_query(query, parameters)?;
                let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);
                RUNTIME
                    .block_on(
                        bind_values(sqlx_oldapi::query(&query), &values)?.fetch_all(&self.pool),
                    )?
                    .into_iter()
                    .map(|row| {
                        Ok((
                            //row.column_value_into_zval(row.column(0), false)?,
                            row.column_value_into_array_key(row.column(0))?,
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
                parameters: Option<BTreeMap<String, ParameterValue>>,
                associative_arrays: Option<bool>,
            ) -> anyhow::Result<Zval> {
                let (query, values) = self.render_query(query, parameters)?;
                let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);

                if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                    let val = RUNTIME.block_on(
                        bind_values(sqlx_oldapi::query(&query), &values)?.fetch_all(&mut *tx),
                    );
                    self.place_ongoing_transaction(tx);
                    val
                } else {
                    RUNTIME.block_on(
                        bind_values(sqlx_oldapi::query(&query), &values)?.fetch_all(&self.pool),
                    )
                }
                .map_err(|err| anyhow!("{err}\n\nQuery: {query}\n\nValues: {values:?}\n\n"))?
                .into_iter()
                .map(|row| {
                    Ok((
                        row.column_value_into_array_key(row.column(0))?,
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
                parameters: Option<BTreeMap<String, ParameterValue>>,
                associative_arrays: Option<bool>,
            ) -> anyhow::Result<Zval> {
                let (query, values) = self.render_query(query, parameters)?;
                let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);
                RUNTIME
                    .block_on(
                        bind_values(sqlx_oldapi::query(&query), &values)?.fetch_all(&self.pool),
                    )
                    .map_err(|err| anyhow!("{err}\n\nQuery: {query}\n\nValues: {values:?}\n\n"))?
                    .into_iter()
                    .map(|row| {
                        Ok((
                            row.column_value_into_array_key(row.column(0))?,
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
                parameters: Option<BTreeMap<String, ParameterValue>>,
                associative_arrays: Option<bool>,
            ) -> anyhow::Result<Zval> {
                let (query, values) = self.render_query(query, parameters)?;
                let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);
                RUNTIME
                    .block_on(
                        bind_values(sqlx_oldapi::query(&query), &values)?.fetch_all(&self.pool),
                    )
                    .map_err(|err| anyhow!("{err}\n\nQuery: {query}\n\nValues: {values:?}\n\n"))?
                    .into_iter()
                    .map(|row| {
                        Ok((
                            row.column_value_into_array_key(row.column(0))?,
                            row.column_value_into_zval(row.column(1), assoc)?,
                        ))
                    })
                    .try_fold(zend_array::new(), fold_into_zend_hashmap)?
                    .into_zval(false)
                    .map_err(|err| anyhow!("{err:?}"))
            }

            /// Begins a new SQL transaction and places it into the transaction stack.
            ///
            /// This method must be called before executing transactional operations
            /// such as savepoints or commit/rollback. If a transaction is already ongoing,
            /// the behavior depends on the SQL backend (may error or allow nesting).
            pub fn begin(&self) -> anyhow::Result<()> {
                self.place_ongoing_transaction(
                    RUNTIME
                        .block_on(self.pool.begin())
                        .map_err(|err| anyhow!("{err}"))?,
                );
                Ok(())
            }

            /// Savepoints allow partial rollbacks without aborting the entire transaction.
            /// The `savepoint` name must be a valid SQL identifier.
            ///
            /// # Errors
            /// Returns an error if no transaction is active or the name is invalid.
            pub fn savepoint(&self, savepoint: &str) -> anyhow::Result<()> {
                if !is_valid_ident(savepoint) {
                    bail!("Invalid savepoint format");
                }
                if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                    let val = RUNTIME
                        .block_on(
                            sqlx_oldapi::query(&format!("SAVEPOINT {savepoint}")).execute(&mut *tx),
                        )
                        .map_err(|err| anyhow!("{err}"));
                    self.place_ongoing_transaction(tx);
                    val?;
                    Ok(())
                } else {
                    Err(anyhow!("There's no ongoing transaction"))
                }
            }

            /// Rolls back to a previously declared savepoint.
            ///
            /// This undoes all changes made after the given savepoint but does not terminate the transaction.
            /// The `savepoint` name must be a valid SQL identifier.
            ///
            /// # Errors
            /// Returns an error if no transaction is active or the name is invalid.
            pub fn rollback_to_savepoint(&self, savepoint: &str) -> anyhow::Result<()> {
                if !is_valid_ident(savepoint) {
                    bail!("Invalid savepoint format");
                }
                if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                    let val = RUNTIME
                        .block_on(
                            sqlx_oldapi::query(&format!("ROLLBACK TO SAVEPOINT {savepoint}"))
                                .execute(&mut *tx),
                        )
                        .map_err(|err| anyhow!("{err}"));
                    self.place_ongoing_transaction(tx);
                    val?;
                    Ok(())
                } else {
                    Err(anyhow!("There's no ongoing transaction"))
                }
            }

            /// Releases a previously declared savepoint.
            ///
            /// After releasing, the savepoint can no longer be rolled back to.
            /// The `savepoint` name must be a valid SQL identifier.
            ///
            /// # Errors
            /// Returns an error if no transaction is active or the name is invalid.
            pub fn release_savepoint(&self, savepoint: &str) -> anyhow::Result<()> {
                if !is_valid_ident(savepoint) {
                    bail!("Invalid savepoint format");
                }
                if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                    let val = RUNTIME
                        .block_on(
                            sqlx_oldapi::query(&format!("RELEASE SAVEPOINT {savepoint}"))
                                .execute(&mut *tx),
                        )
                        .map_err(|err| anyhow!("{err}"));
                    self.place_ongoing_transaction(tx);
                    val?;
                    Ok(())
                } else {
                    Err(anyhow!("There's no ongoing transaction"))
                }
            }

            /// Commits the current ongoing transaction.
            ///
            /// This method retrieves the transaction from the stack and commits it.
            /// After commit, the transaction is removed from the stack.
            ///
            /// # Errors
            /// Returns an error if no transaction is active or the commit fails.
            pub fn commit(&self) -> anyhow::Result<()> {
                if let Some(tx) = self.retrieve_ongoing_transaction() {
                    RUNTIME
                        .block_on(tx.commit())
                        .map_err(|err| anyhow!("Failed to commit transaction: {err}"))?;
                    Ok(())
                } else {
                    Err(anyhow!("There's no ongoing transaction to commit"))
                }
            }

            /// Rolls back the current ongoing transaction.
            ///
            /// This method retrieves the transaction from the stack and rolls it back.
            /// After rollback, the transaction is removed from the stack.
            ///
            /// # Errors
            /// Returns an error if no transaction is active or the rollback fails.
            pub fn rollback(&self) -> anyhow::Result<()> {
                if let Some(tx) = self.retrieve_ongoing_transaction() {
                    RUNTIME
                        .block_on(tx.rollback())
                        .map_err(|err| anyhow!("Failed to rollback transaction: {err}"))?;
                    Ok(())
                } else {
                    Err(anyhow!("There's no ongoing transaction to rollback"))
                }
            }

            /// Retrieves the ongoing transaction from the transaction stack, if any.
            ///
            /// This is used internally to manage nested transactional logic.
            #[inline(always)]
            pub fn retrieve_ongoing_transaction(&self) -> Option<Transaction<'static, $database>> {
                self.tx_stack.write().expect("Poisoned tx_stack").pop()
            }

            /// Pushes a transaction onto the internal transaction stack.
            ///
            /// Used internally to persist ongoing transaction state across method calls.
            #[inline(always)]
            pub fn place_ongoing_transaction(&self, tx: Transaction<'static, $database>) {
                self.tx_stack.write().expect("Poisoned tx_stack").push(tx);
            }
        }
    };
    ( $( $t:tt )* ) => {
        compile_error!(
            "php_sqlx_impl_driver_inner! accepts 2 arguments: \
             ($struct, $database)"
        );
    };
}
