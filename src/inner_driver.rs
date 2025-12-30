//! Inner database driver implementation macro for php-sqlx.
//!
//! This module provides the [`php_sqlx_impl_driver_inner!`] macro that generates the
//! core database driver logic with connection pooling, query execution, and transaction
//! management. Each database type uses this macro to create its inner driver.
//!
//! The inner driver handles:
//! - Connection pool management with `SQLx`
//! - AST-based query rendering with LRU caching
//! - Transaction stack for nested transaction support
//! - All query execution methods (`query_row`, `query_all`, etc.)
//! - Automatic retry for transient failures
//!
//! This is separated from the outer driver (in `driver.rs`) to allow the PHP bindings
//! to wrap the inner driver with `Arc` for shared ownership across prepared queries
//! and query builders.

use crate::error::Error as SqlxError;
use std::time::Duration;

/// Retry policy configuration for transient database failures.
///
/// When enabled (`max_attempts` > 0), transient errors like connection drops,
/// pool exhaustion, and timeouts will be automatically retried with exponential
/// backoff.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (0 = disabled).
    pub max_attempts: u32,
    /// Initial backoff duration between retries.
    pub initial_backoff: Duration,
    /// Maximum backoff duration (caps exponential growth).
    pub max_backoff: Duration,
    /// Multiplier for exponential backoff (e.g., 2.0 doubles each time).
    pub multiplier: f64,
}

impl RetryPolicy {
    /// Returns the backoff duration for the given attempt, or None if no retry should occur.
    ///
    /// Returns `None` if:
    /// - `attempt >= max_attempts`
    /// - The error is not transient
    #[must_use]
    pub fn should_retry(&self, attempt: u32, error: &SqlxError) -> Option<Duration> {
        if attempt >= self.max_attempts || !error.is_transient() {
            return None;
        }
        // Use saturating conversion to avoid wrap-around for very high attempt counts
        let exponent = i32::try_from(attempt).unwrap_or(i32::MAX);
        let backoff = self.initial_backoff.mul_f64(self.multiplier.powi(exponent));
        Some(backoff.min(self.max_backoff))
    }

    /// Returns true if retry is disabled.
    #[must_use]
    #[inline]
    pub fn is_disabled(&self) -> bool {
        self.max_attempts == 0
    }
}

/// Generates the inner database driver implementation.
///
/// This macro creates the core driver struct with connection pooling, query execution,
/// and transaction management for a specific database backend.
///
/// # Arguments
///
/// - `$struct` - The Rust struct name for the inner driver (e.g., `PgInnerDriver`)
/// - `$database` - The `SQLx` database type (e.g., `Postgres`, `MySql`, `Mssql`)
///
/// # Generated Structure
///
/// The macro generates a struct with:
/// - `pool`: `SQLx` connection pool
/// - `ast_cache`: LRU cache for parsed AST queries
/// - `options`: Driver configuration
/// - `tx_stack`: Transaction stack for nested transaction support
/// - `settings`: AST rendering settings
#[macro_export]
macro_rules! php_sqlx_impl_driver_inner {
    ( $struct:ident, $database:ident ) => {
        use ext_php_rs::{convert::IntoZval, ffi::zend_array, types::Zval};
        use itertools::Itertools;
        use sqlx_oldapi::{
            Column, Row, Transaction,
            pool::{Pool, PoolConnection, PoolOptions},
            $database,
        };
        use std::collections::BTreeMap;
        use std::sync::RwLock;
        use std::sync::atomic::AtomicUsize;
        use threadsafe_lru::LruCache;
        use $crate::{
            RUNTIME,
            ast::{Ast, Settings},
            conversion::Conversion,
            error::Error as SqlxError,
            inner_driver::RetryPolicy,
            options::DriverInnerOptions,
            param_value::{ParameterValue, utils::bind_values},
            query_hook::{QueryHook, QueryTimer},
            utils::{
                hashmap_fold::{fold_into_zend_hashmap, fold_into_zend_hashmap_grouped},
                ident::is_valid_ident,
                types::ColumnArgument,
            },
        };
        /// Core database driver containing connection pool and query execution logic.
        ///
        /// This struct is typically wrapped in `Arc` and shared across the outer driver,
        /// prepared queries, and query builders.
        pub struct $struct {
            /// `SQLx` connection pool for efficient connection reuse (primary).
            pub pool: Pool<$database>,
            /// Read replica connection pools for automatic read/write splitting.
            pub replica_pools: Vec<Pool<$database>>,
            /// Weights for each replica pool (for weighted load balancing).
            replica_weights: Vec<u32>,
            /// Total weight of all replicas (sum of weights).
            replica_total_weight: u32,
            /// Counter for weighted round-robin replica selection.
            replica_counter: AtomicUsize,
            /// LRU cache for parsed SQL AST, reducing parse overhead for repeated queries.
            pub ast_cache: LruCache<String, Ast>,
            /// Driver configuration options.
            pub options: DriverInnerOptions,
            /// Stack of active transactions for nested transaction support.
            pub tx_stack: RwLock<Vec<Transaction<'static, $database>>>,
            /// Pinned connection for session-scoped operations (LAST_INSERT_ID, temp tables, etc.).
            pub pinned_conn: RwLock<Option<PoolConnection<$database>>>,
            /// AST rendering settings (placeholder style, collapsible IN, etc.).
            pub settings: Settings,
            /// Retry policy for transient failures.
            pub retry_policy: RetryPolicy,
            /// Query profiling hook for logging and monitoring.
            pub query_hook: QueryHook,
        }

        impl $struct {
            /// Creates a new inner driver with the given configuration.
            ///
            /// This establishes the connection pool and initializes the AST cache.
            /// The URL must be set in options or an error is returned.
            pub fn new(options: DriverInnerOptions) -> $crate::error::Result<Self> {
                let mut pool_options = PoolOptions::<$database>::new()
                    .max_connections(options.max_connections.into())
                    .min_connections(options.min_connections)
                    .max_lifetime(options.max_lifetime)
                    .idle_timeout(options.idle_timeout)
                    .test_before_acquire(options.test_before_acquire);
                if let Some(acquire_timeout) = options.acquire_timeout {
                    pool_options = pool_options.acquire_timeout(acquire_timeout);
                }
                let url = options.url.clone().ok_or(SqlxError::UrlRequired)?;
                let pool = RUNTIME
                    .block_on(pool_options.connect(url.as_str()))
                    .map_err(|e| SqlxError::connection_with_source("Failed to connect", e))?;

                // Create replica pools with weights
                let mut replica_pools = Vec::with_capacity(options.read_replicas.len());
                let mut replica_weights = Vec::with_capacity(options.read_replicas.len());
                for replica_config in &options.read_replicas {
                    let mut replica_pool_options = PoolOptions::<$database>::new()
                        .max_connections(options.max_connections.into())
                        .min_connections(options.min_connections)
                        .max_lifetime(options.max_lifetime)
                        .idle_timeout(options.idle_timeout)
                        .test_before_acquire(options.test_before_acquire);
                    if let Some(acquire_timeout) = options.acquire_timeout {
                        replica_pool_options =
                            replica_pool_options.acquire_timeout(acquire_timeout);
                    }
                    let replica_pool = RUNTIME
                        .block_on(replica_pool_options.connect(replica_config.url.as_str()))
                        .map_err(|e| {
                            SqlxError::connection_with_source(
                                format!("Failed to connect to replica: {}", replica_config.url),
                                e,
                            )
                        })?;
                    replica_pools.push(replica_pool);
                    replica_weights.push(replica_config.weight);
                }
                let replica_total_weight: u32 = replica_weights.iter().sum();

                let mut settings = SETTINGS.clone();
                settings.collapsible_in_enabled = options.collapsible_in_enabled;
                let retry_policy = RetryPolicy {
                    max_attempts: options.retry_max_attempts,
                    initial_backoff: options.retry_initial_backoff,
                    max_backoff: options.retry_max_backoff,
                    multiplier: options.retry_multiplier,
                };
                Ok(Self {
                    tx_stack: RwLock::new(Vec::new()),
                    pinned_conn: RwLock::new(None),
                    pool,
                    replica_pools,
                    replica_weights,
                    replica_total_weight,
                    replica_counter: AtomicUsize::new(0),
                    ast_cache: LruCache::new(
                        options.ast_cache_shard_count,
                        options.ast_cache_shard_size,
                    ),
                    settings,
                    retry_policy,
                    query_hook: QueryHook::new(),
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

            /// Returns true if there is an active transaction.
            ///
            /// This is used to disable retry logic inside transactions, as retrying
            /// partial transaction operations could lead to data inconsistency.
            #[inline]
            pub fn has_active_transaction(&self) -> bool {
                !self.tx_stack.read().expect("Poisoned tx_stack").is_empty()
            }

            /// Returns true if there is a pinned connection.
            ///
            /// A pinned connection ensures all queries run on the same connection,
            /// which is required for session-scoped operations like LAST_INSERT_ID(),
            /// temporary tables, and session variables.
            #[inline]
            pub fn has_pinned_connection(&self) -> bool {
                self.pinned_conn
                    .read()
                    .expect("Poisoned pinned_conn")
                    .is_some()
            }

            /// Returns true if queries should use a dedicated connection (transaction or pinned).
            #[inline]
            pub fn has_dedicated_connection(&self) -> bool {
                self.has_active_transaction() || self.has_pinned_connection()
            }

            /// Returns true if read replicas are configured.
            #[inline]
            pub fn has_read_replicas(&self) -> bool {
                !self.replica_pools.is_empty()
            }

            /// Returns a reference to a read replica pool using weighted selection.
            ///
            /// When weights are configured, replicas receive traffic proportional to their weight.
            /// For example, with weights [3, 1], the first replica gets ~75% of traffic.
            ///
            /// Returns the primary pool if:
            /// - No replicas are configured
            /// - There's an active transaction (all queries go to primary)
            #[inline]
            pub fn get_read_pool(&self) -> &Pool<$database> {
                if self.replica_pools.is_empty() || self.has_active_transaction() {
                    return &self.pool;
                }

                // Simple round-robin if all weights are equal (optimization)
                if self.replica_total_weight as usize == self.replica_pools.len() {
                    let index = self
                        .replica_counter
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    return &self.replica_pools[index % self.replica_pools.len()];
                }

                // Weighted selection (truncation is intentional for wrapping)
                let counter = self
                    .replica_counter
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                #[allow(clippy::cast_possible_truncation)]
                let slot = (counter as u32) % self.replica_total_weight;

                let mut cumulative = 0u32;
                for (i, &weight) in self.replica_weights.iter().enumerate() {
                    cumulative += weight;
                    if slot < cumulative {
                        return &self.replica_pools[i];
                    }
                }

                // Fallback (should never happen if weights are valid)
                &self.replica_pools[0]
            }

            /// Executes an operation with retry logic for transient failures.
            ///
            /// Retries are skipped if:
            /// - Retry is disabled (`max_attempts` = 0)
            /// - There is an active transaction (to prevent partial commits)
            /// - The error is not transient
            ///
            /// Uses exponential backoff between retries.
            fn with_retry<F, T>(&self, operation: F) -> $crate::error::Result<T>
            where
                F: Fn() -> $crate::error::Result<T>,
            {
                // Skip retry if disabled or in transaction
                if self.retry_policy.is_disabled() || self.has_active_transaction() {
                    return operation();
                }

                let mut attempt = 0u32;
                loop {
                    match operation() {
                        Ok(result) => return Ok(result),
                        Err(e) => {
                            if let Some(backoff) = self.retry_policy.should_retry(attempt, &e) {
                                attempt += 1;
                                std::thread::sleep(backoff);
                                continue;
                            }
                            return Err(e);
                        }
                    }
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
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<u64> {
                // Render inline query for logging only if hook is active
                let sql_inline = if self.query_hook.is_set() {
                    self.render_query_inline(query, parameters.clone()).ok()
                } else {
                    None
                };

                let (query, values) = self.render_query(query, parameters)?;

                // Start timing if hook is active
                let timer = QueryTimer::new(&self.query_hook, query.clone(), sql_inline);

                let result = self.with_retry(|| {
                    Ok(if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                        let val = RUNTIME.block_on(
                            bind_values(sqlx_oldapi::query(&query), &values)?.execute(&mut *tx),
                        );
                        self.place_ongoing_transaction(tx);
                        val
                    } else if let Some(mut conn) = self.retrieve_pinned_connection() {
                        let val = RUNTIME.block_on(
                            bind_values(sqlx_oldapi::query(&query), &values)?.execute(&mut *conn),
                        );
                        self.return_pinned_connection(conn);
                        val
                    } else {
                        RUNTIME.block_on(
                            bind_values(sqlx_oldapi::query(&query), &values)?.execute(&self.pool),
                        )
                    }
                    .map_err(|err| SqlxError::query_with_source(&query, err))?
                    .rows_affected())
                });

                // Call hook with timing info
                if let Some(t) = timer {
                    t.finish();
                }

                result
            }

            /// Renders the final SQL query and parameters using the AST cache.
            ///
            /// Looks up the query in the cache; if not found, parses it and caches the AST.
            /// Returns the rendered SQL string with positional placeholders and the parameter values.
            fn render_query(
                &self,
                query: &str,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<(String, Vec<ParameterValue>)> {
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
            ) -> $crate::error::Result<String> {
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
            pub fn parse_query(&self, query: &str) -> $crate::error::Result<Ast> {
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
            ) -> $crate::error::Result<Zval> {
                let (query, values) = self.render_query(query, parameters)?;
                let row = self.with_retry(|| {
                    if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                        let val = RUNTIME.block_on(
                            bind_values(sqlx_oldapi::query(&query), &values)?.fetch_one(&mut *tx),
                        );
                        self.place_ongoing_transaction(tx);
                        val
                    } else if let Some(mut conn) = self.retrieve_pinned_connection() {
                        let val = RUNTIME.block_on(
                            bind_values(sqlx_oldapi::query(&query), &values)?.fetch_one(&mut *conn),
                        );
                        self.return_pinned_connection(conn);
                        val
                    } else {
                        RUNTIME.block_on(
                            bind_values(sqlx_oldapi::query(&query), &values)?
                                .fetch_one(self.get_read_pool()),
                        )
                    }
                    .map_err(|err| SqlxError::query_with_source(&query, err))
                })?;
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
                            return Err(SqlxError::ColumnNotFound {
                                column: column_name.to_string(),
                            });
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
            ) -> $crate::error::Result<Vec<Zval>> {
                let (query, values) = self.render_query(query, parameters)?;
                let mut it = self
                    .with_retry(|| {
                        if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                            let val = RUNTIME.block_on(
                                bind_values(sqlx_oldapi::query(&query), &values)?
                                    .fetch_all(&mut *tx),
                            );
                            self.place_ongoing_transaction(tx);
                            val
                        } else if let Some(mut conn) = self.retrieve_pinned_connection() {
                            let val = RUNTIME.block_on(
                                bind_values(sqlx_oldapi::query(&query), &values)?
                                    .fetch_all(&mut *conn),
                            );
                            self.return_pinned_connection(conn);
                            val
                        } else {
                            RUNTIME.block_on(
                                bind_values(sqlx_oldapi::query(&query), &values)?
                                    .fetch_all(self.get_read_pool()),
                            )
                        }
                        .map_err(|err| SqlxError::query_with_source(&query, err))
                    })?
                    .into_iter()
                    .peekable();
                let Some(row) = it.peek() else {
                    return Ok(vec![]);
                };
                let column_idx: usize = match column {
                    Some(ColumnArgument::Index(i)) => {
                        if row.try_column(i).is_err() {
                            return Err(SqlxError::ColumnNotFound {
                                column: i.to_string(),
                            });
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
                            return Err(SqlxError::ColumnNotFound {
                                column: column_name.to_string(),
                            });
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
            ) -> $crate::error::Result<Zval> {
                let (query, values) = self.render_query(query, parameters)?;
                Ok(self
                    .with_retry(|| {
                        if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                            let val = RUNTIME.block_on(
                                bind_values(sqlx_oldapi::query(&query), &values)?
                                    .fetch_one(&mut *tx),
                            );
                            self.place_ongoing_transaction(tx);
                            val
                        } else if let Some(mut conn) = self.retrieve_pinned_connection() {
                            let val = RUNTIME.block_on(
                                bind_values(sqlx_oldapi::query(&query), &values)?
                                    .fetch_one(&mut *conn),
                            );
                            self.return_pinned_connection(conn);
                            val
                        } else {
                            RUNTIME.block_on(
                                bind_values(sqlx_oldapi::query(&query), &values)?
                                    .fetch_one(self.get_read_pool()),
                            )
                        }
                        .map(Some)
                        .or_else(|err: sqlx_oldapi::Error| match err {
                            sqlx_oldapi::Error::RowNotFound => Ok(None),
                            _ => Err(SqlxError::query_with_source(&query, err)),
                        })
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
                                    return Err(SqlxError::ColumnNotFound {
                                        column: column_name.to_string(),
                                    });
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
            ) -> $crate::error::Result<Zval> {
                let sql_inline = if self.query_hook.is_set() {
                    self.render_query_inline(query, parameters.clone()).ok()
                } else {
                    None
                };

                let (query, values) = self.render_query(query, parameters)?;
                let timer = QueryTimer::new(&self.query_hook, query.clone(), sql_inline);

                let result = self
                    .with_retry(|| {
                        if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                            let val = RUNTIME.block_on(
                                bind_values(sqlx_oldapi::query(&query), &values)?
                                    .fetch_one(&mut *tx),
                            );
                            self.place_ongoing_transaction(tx);
                            val
                        } else if let Some(mut conn) = self.retrieve_pinned_connection() {
                            let val = RUNTIME.block_on(
                                bind_values(sqlx_oldapi::query(&query), &values)?
                                    .fetch_one(&mut *conn),
                            );
                            self.return_pinned_connection(conn);
                            val
                        } else {
                            RUNTIME.block_on(
                                bind_values(sqlx_oldapi::query(&query), &values)?
                                    .fetch_one(self.get_read_pool()),
                            )
                        }
                        .map_err(|err| SqlxError::query_with_source(&query, err))
                    })?
                    .into_zval(associative_arrays.unwrap_or(self.options.associative_arrays));

                if let Some(t) = timer {
                    t.finish();
                }

                result
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
            ) -> $crate::error::Result<Zval> {
                let sql_inline = if self.query_hook.is_set() {
                    self.render_query_inline(query, parameters.clone()).ok()
                } else {
                    None
                };

                let (query, values) = self.render_query(query, parameters)?;
                let timer = QueryTimer::new(&self.query_hook, query.clone(), sql_inline);

                let result = self
                    .with_retry(|| {
                        if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                            let val = RUNTIME.block_on(
                                bind_values(sqlx_oldapi::query(&query), &values)?
                                    .fetch_one(&mut *tx),
                            );
                            self.place_ongoing_transaction(tx);
                            val
                        } else if let Some(mut conn) = self.retrieve_pinned_connection() {
                            let val = RUNTIME.block_on(
                                bind_values(sqlx_oldapi::query(&query), &values)?
                                    .fetch_one(&mut *conn),
                            );
                            self.return_pinned_connection(conn);
                            val
                        } else {
                            RUNTIME.block_on(
                                bind_values(sqlx_oldapi::query(&query), &values)?
                                    .fetch_one(self.get_read_pool()),
                            )
                        }
                        .map(Some)
                        .or_else(|err: sqlx_oldapi::Error| match err {
                            sqlx_oldapi::Error::RowNotFound => Ok(None),
                            _ => Err(SqlxError::query_with_source(&query, err)),
                        })
                    })?
                    .map(|x| {
                        x.into_zval(associative_arrays.unwrap_or(self.options.associative_arrays))
                    })
                    .transpose()?
                    .unwrap_or_else(Zval::null);

                if let Some(t) = timer {
                    t.finish();
                }

                Ok(result)
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
            ) -> $crate::error::Result<Vec<Zval>> {
                let sql_inline = if self.query_hook.is_set() {
                    self.render_query_inline(query, parameters.clone()).ok()
                } else {
                    None
                };

                let (query, values) = self.render_query(query, parameters)?;
                let timer = QueryTimer::new(&self.query_hook, query.clone(), sql_inline);

                let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);
                let result = self
                    .with_retry(|| {
                        if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                            let val = RUNTIME.block_on(
                                bind_values(sqlx_oldapi::query(&query), &values)?
                                    .fetch_all(&mut *tx),
                            );
                            self.place_ongoing_transaction(tx);
                            val
                        } else if let Some(mut conn) = self.retrieve_pinned_connection() {
                            let val = RUNTIME.block_on(
                                bind_values(sqlx_oldapi::query(&query), &values)?
                                    .fetch_all(&mut *conn),
                            );
                            self.return_pinned_connection(conn);
                            val
                        } else {
                            RUNTIME.block_on(
                                bind_values(sqlx_oldapi::query(&query), &values)?
                                    .fetch_all(self.get_read_pool()),
                            )
                        }
                        .map_err(|err| SqlxError::query_with_source(&query, err))
                    })?
                    .into_iter()
                    .map(|row| row.into_zval(assoc))
                    .try_collect();

                if let Some(t) = timer {
                    t.finish();
                }

                result
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
            ) -> $crate::error::Result<Vec<Zval>> {
                let (query, values) = self.render_query(query, parameters)?;
                Ok(vec![
                    query
                        .into_zval(false)
                        .map_err(|err| SqlxError::Conversion {
                            message: format!("{err:?}"),
                        })?,
                    values
                        .into_zval(false)
                        .map_err(|err| SqlxError::Conversion {
                            message: format!("{err:?}"),
                        })?,
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
            ) -> $crate::error::Result<String> {
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
            ) -> $crate::error::Result<Zval> {
                let (query, values) = self.render_query(query, parameters)?;
                let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);
                self.with_retry(|| {
                    if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                        let val = RUNTIME.block_on(
                            bind_values(sqlx_oldapi::query(&query), &values)?.fetch_all(&mut *tx),
                        );
                        self.place_ongoing_transaction(tx);
                        val
                    } else if let Some(mut conn) = self.retrieve_pinned_connection() {
                        let val = RUNTIME.block_on(
                            bind_values(sqlx_oldapi::query(&query), &values)?.fetch_all(&mut *conn),
                        );
                        self.return_pinned_connection(conn);
                        val
                    } else {
                        RUNTIME.block_on(
                            bind_values(sqlx_oldapi::query(&query), &values)?
                                .fetch_all(self.get_read_pool()),
                        )
                    }
                    .map_err(|err| SqlxError::query_with_source(&query, err))
                })?
                .into_iter()
                .map(|row| {
                    Ok((
                        row.column_value_into_array_key(row.column(0))?,
                        row.into_zval(assoc)?,
                    ))
                })
                .try_fold(zend_array::new(), fold_into_zend_hashmap)?
                .into_zval(false)
                .map_err(|err| SqlxError::Conversion {
                    message: format!("{err:?}"),
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
                parameters: Option<BTreeMap<String, ParameterValue>>,
                associative_arrays: Option<bool>,
            ) -> $crate::error::Result<Zval> {
                let (query, values) = self.render_query(query, parameters)?;
                let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);

                self.with_retry(|| {
                    if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                        let val = RUNTIME.block_on(
                            bind_values(sqlx_oldapi::query(&query), &values)?.fetch_all(&mut *tx),
                        );
                        self.place_ongoing_transaction(tx);
                        val
                    } else if let Some(mut conn) = self.retrieve_pinned_connection() {
                        let val = RUNTIME.block_on(
                            bind_values(sqlx_oldapi::query(&query), &values)?.fetch_all(&mut *conn),
                        );
                        self.return_pinned_connection(conn);
                        val
                    } else {
                        RUNTIME.block_on(
                            bind_values(sqlx_oldapi::query(&query), &values)?
                                .fetch_all(self.get_read_pool()),
                        )
                    }
                    .map_err(|err| SqlxError::query_with_source(&query, err))
                })?
                .into_iter()
                .map(|row| {
                    Ok((
                        row.column_value_into_array_key(row.column(0))?,
                        row.into_zval(assoc)?,
                    ))
                })
                .try_fold(zend_array::new(), fold_into_zend_hashmap_grouped)?
                .into_zval(false)
                .map_err(|err| SqlxError::Conversion {
                    message: format!("{err:?}"),
                })
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
            ) -> $crate::error::Result<Zval> {
                let (query, values) = self.render_query(query, parameters)?;
                let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);
                self.with_retry(|| {
                    if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                        let val = RUNTIME.block_on(
                            bind_values(sqlx_oldapi::query(&query), &values)?.fetch_all(&mut *tx),
                        );
                        self.place_ongoing_transaction(tx);
                        val
                    } else if let Some(mut conn) = self.retrieve_pinned_connection() {
                        let val = RUNTIME.block_on(
                            bind_values(sqlx_oldapi::query(&query), &values)?.fetch_all(&mut *conn),
                        );
                        self.return_pinned_connection(conn);
                        val
                    } else {
                        RUNTIME.block_on(
                            bind_values(sqlx_oldapi::query(&query), &values)?
                                .fetch_all(self.get_read_pool()),
                        )
                    }
                    .map_err(|err| SqlxError::query_with_source(&query, err))
                })?
                .into_iter()
                .map(|row| {
                    Ok((
                        row.column_value_into_array_key(row.column(0))?,
                        row.column_value_into_zval(row.column(1), assoc)?,
                    ))
                })
                .try_fold(zend_array::new(), fold_into_zend_hashmap_grouped)?
                .into_zval(false)
                .map_err(|err| SqlxError::Conversion {
                    message: format!("{err:?}"),
                })
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
            ) -> $crate::error::Result<Zval> {
                let (query, values) = self.render_query(query, parameters)?;
                let assoc = associative_arrays.unwrap_or(self.options.associative_arrays);
                self.with_retry(|| {
                    if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                        let val = RUNTIME.block_on(
                            bind_values(sqlx_oldapi::query(&query), &values)?.fetch_all(&mut *tx),
                        );
                        self.place_ongoing_transaction(tx);
                        val
                    } else if let Some(mut conn) = self.retrieve_pinned_connection() {
                        let val = RUNTIME.block_on(
                            bind_values(sqlx_oldapi::query(&query), &values)?.fetch_all(&mut *conn),
                        );
                        self.return_pinned_connection(conn);
                        val
                    } else {
                        RUNTIME.block_on(
                            bind_values(sqlx_oldapi::query(&query), &values)?
                                .fetch_all(self.get_read_pool()),
                        )
                    }
                    .map_err(|err| SqlxError::query_with_source(&query, err))
                })?
                .into_iter()
                .map(|row| {
                    Ok((
                        row.column_value_into_array_key(row.column(0))?,
                        row.column_value_into_zval(row.column(1), assoc)?,
                    ))
                })
                .try_fold(zend_array::new(), fold_into_zend_hashmap)?
                .into_zval(false)
                .map_err(|err| SqlxError::Conversion {
                    message: format!("{err:?}"),
                })
            }

            /// Describes table columns with their types and metadata.
            ///
            /// Returns information about each column in the specified table, including
            /// name, type, nullability, default value, and ordinal position.
            ///
            /// # Parameters
            /// - `table_name`: Name of the table to describe.
            /// - `schema`: Optional schema name. If `None`, uses the database default schema.
            ///
            /// # Returns
            /// An array of associative arrays, each containing:
            /// - `name`: Column name
            /// - `type`: Database-specific column type (e.g., "varchar(255)", "int")
            /// - `nullable`: Whether the column allows NULL values
            /// - `default`: Default value for the column, or NULL if none
            /// - `ordinal`: Column position (1-based)
            ///
            /// # Errors
            /// Returns an error if:
            /// - the table or schema name is invalid (contains invalid characters);
            /// - the query fails to execute;
            /// - the table does not exist.
            pub fn describe_table(
                &self,
                table_name: &str,
                schema: Option<&str>,
            ) -> $crate::error::Result<Vec<Zval>> {
                // Validate identifiers to prevent SQL injection
                if !is_valid_ident(table_name) {
                    return Err(SqlxError::InvalidIdentifier {
                        value: table_name.to_string(),
                    });
                }
                if let Some(s) = schema {
                    if !is_valid_ident(s) {
                        return Err(SqlxError::InvalidIdentifier {
                            value: s.to_string(),
                        });
                    }
                }

                let mut params = BTreeMap::new();
                params.insert(
                    "table".to_string(),
                    ParameterValue::String(table_name.to_string()),
                );
                params.insert(
                    "schema".to_string(),
                    schema.map_or(ParameterValue::Null, |s| {
                        ParameterValue::String(s.to_string())
                    }),
                );

                self.query_all(DESCRIBE_TABLE_QUERY, Some(params), Some(true))
            }

            /// Sets the application name for this connection.
            ///
            /// This helps identify the connection in database monitoring tools:
            /// - `PostgreSQL`: Visible in `pg_stat_activity.application_name`
            /// - `MySQL`: Stored in session variable `@sqlx_application_name`
            /// - `MSSQL`: Stored in session context, readable via `SESSION_CONTEXT(N'application_name')`
            ///
            /// # Parameters
            /// - `name`: The application name to set.
            ///
            /// # Errors
            /// Returns an error if the query fails to execute.
            pub fn set_application_name(&self, name: &str) -> $crate::error::Result<()> {
                let mut params = BTreeMap::new();
                params.insert("name".to_string(), ParameterValue::String(name.to_string()));
                self.execute(SET_APPLICATION_NAME_QUERY, Some(params))?;
                Ok(())
            }

            /// Sets client metadata for this connection.
            ///
            /// The metadata is formatted as a JSON object and appended to the application name,
            /// making it visible in database monitoring tools. This is useful for tracking
            /// request IDs, user IDs, or other debugging information.
            ///
            /// # Parameters
            /// - `info`: Key-value pairs of client metadata.
            ///
            /// # Example
            /// ```php
            /// $driver->setClientInfo(['request_id' => $requestId, 'user_id' => $userId]);
            /// // Sets application name to: "myapp {\"request_id\":\"abc123\",\"user_id\":42}"
            /// ```
            ///
            /// # Errors
            /// Returns an error if the query fails to execute.
            pub fn set_client_info(
                &self,
                base_name: &str,
                info: &BTreeMap<String, ParameterValue>,
            ) -> $crate::error::Result<()> {
                // Format info as key=value pairs
                let mut parts = Vec::new();
                for (k, v) in info {
                    let quoted = v.quote(&self.settings)?;
                    parts.push(format!("{k}={quoted}"));
                }
                let info_str = parts.join(",");

                let full_name = if info_str.is_empty() {
                    base_name.to_string()
                } else {
                    format!("{base_name} {{{info_str}}}")
                };

                self.set_application_name(&full_name)
            }

            /// Begins a new SQL transaction and places it into the transaction stack.
            ///
            /// This method must be called before executing transactional operations
            /// such as savepoints or commit/rollback. If a transaction is already ongoing,
            /// the behavior depends on the SQL backend (may error or allow nesting).
            pub fn begin(&self) -> $crate::error::Result<()> {
                self.place_ongoing_transaction(
                    RUNTIME
                        .block_on(self.pool.begin())
                        .map_err(|err| SqlxError::Other(err.to_string()))?,
                );
                Ok(())
            }

            /// Savepoints allow partial rollbacks without aborting the entire transaction.
            /// The `savepoint` name must be a valid SQL identifier.
            ///
            /// # Errors
            /// Returns an error if no transaction is active or the name is invalid.
            pub fn savepoint(&self, savepoint: &str) -> $crate::error::Result<()> {
                if !is_valid_ident(savepoint) {
                    return Err(SqlxError::InvalidSavepoint {
                        name: savepoint.to_string(),
                    });
                }
                if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                    let val = RUNTIME
                        .block_on(
                            sqlx_oldapi::query(&format!("SAVEPOINT {savepoint}")).execute(&mut *tx),
                        )
                        .map_err(|err| SqlxError::Other(err.to_string()));
                    self.place_ongoing_transaction(tx);
                    val?;
                    Ok(())
                } else {
                    Err(SqlxError::NoActiveTransaction)
                }
            }

            /// Rolls back to a previously declared savepoint.
            ///
            /// This undoes all changes made after the given savepoint but does not terminate the transaction.
            /// The `savepoint` name must be a valid SQL identifier.
            ///
            /// # Errors
            /// Returns an error if no transaction is active or the name is invalid.
            pub fn rollback_to_savepoint(&self, savepoint: &str) -> $crate::error::Result<()> {
                if !is_valid_ident(savepoint) {
                    return Err(SqlxError::InvalidSavepoint {
                        name: savepoint.to_string(),
                    });
                }
                if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                    let val = RUNTIME
                        .block_on(
                            sqlx_oldapi::query(&format!("ROLLBACK TO SAVEPOINT {savepoint}"))
                                .execute(&mut *tx),
                        )
                        .map_err(|err| SqlxError::Other(err.to_string()));
                    self.place_ongoing_transaction(tx);
                    val?;
                    Ok(())
                } else {
                    Err(SqlxError::NoActiveTransaction)
                }
            }

            /// Releases a previously declared savepoint.
            ///
            /// After releasing, the savepoint can no longer be rolled back to.
            /// The `savepoint` name must be a valid SQL identifier.
            ///
            /// # Errors
            /// Returns an error if no transaction is active or the name is invalid.
            pub fn release_savepoint(&self, savepoint: &str) -> $crate::error::Result<()> {
                if !is_valid_ident(savepoint) {
                    return Err(SqlxError::InvalidSavepoint {
                        name: savepoint.to_string(),
                    });
                }
                if let Some(mut tx) = self.retrieve_ongoing_transaction() {
                    let val = RUNTIME
                        .block_on(
                            sqlx_oldapi::query(&format!("RELEASE SAVEPOINT {savepoint}"))
                                .execute(&mut *tx),
                        )
                        .map_err(|err| SqlxError::Other(err.to_string()));
                    self.place_ongoing_transaction(tx);
                    val?;
                    Ok(())
                } else {
                    Err(SqlxError::NoActiveTransaction)
                }
            }

            /// Commits the current ongoing transaction.
            ///
            /// This method retrieves the transaction from the stack and commits it.
            /// After commit, the transaction is removed from the stack.
            ///
            /// # Errors
            /// Returns an error if no transaction is active or the commit fails.
            pub fn commit(&self) -> $crate::error::Result<()> {
                if let Some(tx) = self.retrieve_ongoing_transaction() {
                    RUNTIME
                        .block_on(tx.commit())
                        .map_err(SqlxError::commit_failed)?;
                    Ok(())
                } else {
                    Err(SqlxError::NoActiveTransaction)
                }
            }

            /// Rolls back the current ongoing transaction.
            ///
            /// This method retrieves the transaction from the stack and rolls it back.
            /// After rollback, the transaction is removed from the stack.
            ///
            /// # Errors
            /// Returns an error if no transaction is active or the rollback fails.
            pub fn rollback(&self) -> $crate::error::Result<()> {
                if let Some(tx) = self.retrieve_ongoing_transaction() {
                    RUNTIME
                        .block_on(tx.rollback())
                        .map_err(SqlxError::rollback_failed)?;
                    Ok(())
                } else {
                    Err(SqlxError::NoActiveTransaction)
                }
            }

            /// Pins a connection from the pool for exclusive use.
            ///
            /// All subsequent queries will use this connection until `unpin_connection()` is called.
            /// This is useful for session-scoped operations like:
            /// - `LAST_INSERT_ID()` in MySQL
            /// - Temporary tables
            /// - Session variables
            /// - Advisory locks
            ///
            /// Unlike transactions, pinned connections don't start a database transaction,
            /// so each query is auto-committed.
            ///
            /// # Errors
            /// Returns an error if a connection is already pinned or acquisition fails.
            pub fn pin_connection(&self) -> $crate::error::Result<()> {
                if self.has_pinned_connection() {
                    return Err(SqlxError::Other("Connection already pinned".to_string()));
                }
                let conn = RUNTIME.block_on(self.pool.acquire()).map_err(|err| {
                    SqlxError::connection_with_source("Failed to acquire connection", err)
                })?;
                *self.pinned_conn.write().expect("Poisoned pinned_conn") = Some(conn);
                Ok(())
            }

            /// Releases the pinned connection back to the pool.
            ///
            /// # Errors
            /// Returns an error if no connection is pinned.
            pub fn unpin_connection(&self) -> $crate::error::Result<()> {
                let conn = self
                    .pinned_conn
                    .write()
                    .expect("Poisoned pinned_conn")
                    .take();
                if conn.is_some() {
                    Ok(())
                } else {
                    Err(SqlxError::Other("No connection pinned".to_string()))
                }
            }

            /// Retrieves the pinned connection for use in a query.
            ///
            /// Returns the connection temporarily, caller must return it via `return_pinned_connection`.
            #[inline(always)]
            pub fn retrieve_pinned_connection(&self) -> Option<PoolConnection<$database>> {
                self.pinned_conn
                    .write()
                    .expect("Poisoned pinned_conn")
                    .take()
            }

            /// Returns a pinned connection after use.
            #[inline(always)]
            pub fn return_pinned_connection(&self, conn: PoolConnection<$database>) {
                *self.pinned_conn.write().expect("Poisoned pinned_conn") = Some(conn);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    fn test_policy() -> RetryPolicy {
        RetryPolicy {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(5),
            multiplier: 2.0,
        }
    }

    #[test]
    fn test_retry_policy_is_disabled() {
        let disabled = RetryPolicy {
            max_attempts: 0,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(5),
            multiplier: 2.0,
        };
        assert!(disabled.is_disabled());

        let enabled = test_policy();
        assert!(!enabled.is_disabled());
    }

    #[test]
    fn test_retry_policy_should_retry_transient_error() {
        let policy = test_policy();
        let transient_error = Error::PoolExhausted { timeout_ms: 1000 };

        // First attempt (0) should retry
        let backoff = policy.should_retry(0, &transient_error);
        assert!(backoff.is_some());
        assert_eq!(backoff.unwrap(), Duration::from_millis(100));

        // Second attempt (1) should retry with doubled backoff
        let backoff = policy.should_retry(1, &transient_error);
        assert!(backoff.is_some());
        assert_eq!(backoff.unwrap(), Duration::from_millis(200));

        // Third attempt (2) should retry with 4x backoff
        let backoff = policy.should_retry(2, &transient_error);
        assert!(backoff.is_some());
        assert_eq!(backoff.unwrap(), Duration::from_millis(400));

        // Fourth attempt (3) should NOT retry (exceeded max_attempts)
        let backoff = policy.should_retry(3, &transient_error);
        assert!(backoff.is_none());
    }

    #[test]
    fn test_retry_policy_should_not_retry_non_transient_error() {
        let policy = test_policy();
        let non_transient = Error::query("syntax error");

        // Should not retry non-transient errors even on first attempt
        let backoff = policy.should_retry(0, &non_transient);
        assert!(backoff.is_none());
    }

    #[test]
    fn test_retry_policy_backoff_capped_at_max() {
        let policy = RetryPolicy {
            max_attempts: 10,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(5),
            multiplier: 10.0,
        };
        let error = Error::connection("connection reset");

        // Attempt 0: 1s
        assert_eq!(policy.should_retry(0, &error), Some(Duration::from_secs(1)));

        // Attempt 1: 10s -> capped to 5s
        assert_eq!(policy.should_retry(1, &error), Some(Duration::from_secs(5)));

        // Attempt 2: 100s -> capped to 5s
        assert_eq!(policy.should_retry(2, &error), Some(Duration::from_secs(5)));
    }

    #[test]
    fn test_retry_policy_disabled_never_retries() {
        let disabled = RetryPolicy {
            max_attempts: 0,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(5),
            multiplier: 2.0,
        };
        let transient_error = Error::PoolExhausted { timeout_ms: 1000 };

        // Should not retry even transient errors when disabled
        assert!(disabled.should_retry(0, &transient_error).is_none());
    }

    #[test]
    fn test_retry_policy_all_transient_error_types() {
        let policy = test_policy();

        // PoolExhausted
        assert!(
            policy
                .should_retry(0, &Error::PoolExhausted { timeout_ms: 1000 })
                .is_some()
        );

        // Timeout
        assert!(
            policy
                .should_retry(
                    0,
                    &Error::Timeout {
                        operation: "acquire".to_string(),
                        timeout_ms: 5000
                    }
                )
                .is_some()
        );

        // Connection
        assert!(
            policy
                .should_retry(0, &Error::connection("connection reset"))
                .is_some()
        );
    }
}
