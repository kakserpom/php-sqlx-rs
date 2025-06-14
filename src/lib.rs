#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::too_many_lines)]
#![cfg_attr(windows, feature(abi_vectorcall))]

mod ast;

use crate::ast::{Ast, Value};
use anyhow::anyhow;
use dashmap::DashMap;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::{prelude::*, types::Zval};
use itertools::Itertools;
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{Column, Row};
use sqlx_core::database::Database;
use sqlx_core::query::Query;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use threadsafe_lru::LruCache;
use tokio::runtime::Runtime;

/// Global runtime for executing async `SQLx` queries from sync context.
static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

#[php_const]
const SQLX_OPTION_URL: &'static str = "url";
const SQLX_OPTION_AST_CACHE_SHARD_COUNT: &'static str = "ast_cache_shard_count";
const SQLX_OPTION_AST_CACHE_SHARD_SIZE: &'static str = "ast_cache_shard_size";
const SQLX_OPTION_PERSISTENT_NAME: &'static str = "persistent_name";

static PERSISTENT_DRIVER_REGISTRY: LazyLock<DashMap<String, Arc<DriverInner>>> =
    LazyLock::new(|| DashMap::new());

/// A database connection wrapper with query helpers and AST cache.
#[php_class(name = "Sqlx")]
pub struct Driver {
    pub inner: Arc<DriverInner>,
}
pub struct DriverInner {
    pub pool: sqlx::PgPool,
    pub ast_cache: LruCache<String, Ast>,
}

impl DriverInner {
    /// Execute a query (e.g., INSERT/UPDATE) and return the number of rows affected.
    pub fn execute(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<u64> {
        let (query, values) = self.render_query(query, parameters);
        Ok(RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).execute(&self.pool))?
            .rows_affected())
    }

    /// Render the final SQL query and parameters using the AST cache.
    fn render_query(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> (String, Vec<Value>) {
        let parameters = parameters.unwrap_or_default();
        if let Some(ast) = self.ast_cache.get(query) {
            ast.render(parameters)
        } else {
            let ast = Ast::parse(query).unwrap();
            let rendered = ast.render(parameters);
            self.ast_cache.insert(query.to_owned(), ast);
            rendered
        }
    }

    /// Execute a query and return a single row.
    pub fn query_one(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        let (query, values) = self.render_query(query, parameters);
        RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_one(&self.pool))?
            .into_zval()
    }

    /// Execute a query and return all matching rows.
    pub fn query_all(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Vec<Zval>> {
        let (query, values) = self.render_query(query, parameters);
        RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_all(&self.pool))?
            .into_iter()
            .map(PgRow::into_zval)
            .try_collect()
    }
}

/// Trait to convert a row into a PHP value.
trait RowToZval: Row {
    /// Convert the row into a PHP `Zval` associative array.
    fn into_zval(self) -> anyhow::Result<Zval>;
}

impl RowToZval for PgRow {
    fn into_zval(self) -> anyhow::Result<Zval> {
        self.columns()
            .iter()
            .map(|column| {
                let name = column.name();
                (
                    name.to_string(),
                    if let Ok(v) = self.try_get::<i64, _>(name) {
                        v.into_zval(false).ok()
                    } else if let Ok(v) = self.try_get::<f64, _>(name) {
                        v.into_zval(false).ok()
                    } else if let Ok(v) = self.try_get::<bool, _>(name) {
                        v.into_zval(false).ok()
                    } else if let Ok(v) = self.try_get::<String, _>(name) {
                        v.into_zval(false).ok()
                    } else {
                        None
                    }
                    .unwrap_or_else(|| {
                        let mut null = Zval::new();
                        null.set_null();
                        null
                    }),
                )
            })
            .collect::<HashMap<String, Zval>>()
            .into_zval(false)
            .map_err(|err| anyhow!("{:?}", err))
    }
}

/// Binds a list of `Value` arguments to an `SQLx` query.
fn bind_values<'a, D: Database>(
    query: Query<'a, D, <D>::Arguments<'a>>,
    values: &'a [Value],
) -> Query<'a, D, <D>::Arguments<'a>>
where
    f64: sqlx::Type<D>,
    f64: sqlx::Encode<'a, D>,
    i64: sqlx::Type<D>,
    i64: sqlx::Encode<'a, D>,
    bool: sqlx::Type<D>,
    bool: sqlx::Encode<'a, D>,
    String: sqlx::Type<D>,
    String: sqlx::Encode<'a, D>,
{
    fn walker<'a, D: Database>(
        q: Query<'a, D, <D>::Arguments<'a>>,
        value: &'a Value,
    ) -> Query<'a, D, <D>::Arguments<'a>>
    where
        f64: sqlx::Type<D>,
        f64: sqlx::Encode<'a, D>,
        i64: sqlx::Type<D>,
        i64: sqlx::Encode<'a, D>,
        bool: sqlx::Type<D>,
        bool: sqlx::Encode<'a, D>,
        String: sqlx::Type<D>,
        String: sqlx::Encode<'a, D>,
    {
        match value {
            Value::Str(s) => q.bind(s),
            Value::Int(s) => q.bind(s),
            Value::Bool(s) => q.bind(s),
            Value::Float(s) => q.bind(s),
            Value::Array(s) => s.iter().fold(q, walker),
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
}
impl Default for DriverOptions {
    fn default() -> Self {
        Self {
            url: None,
            ast_cache_shard_count: 8,
            ast_cache_shard_size: 128,
            persistent_name: None,
        }
    }
}

#[php_impl]
impl Driver {
    /// Create a new database connection using the given URL.
    ///
    /// # Parameters
    /// - `url`: SQLX connection string.
    pub fn __construct(options: DriverConstructorOptions) -> anyhow::Result<Driver> {
        let options = match options {
            DriverConstructorOptions::Url(url) => DriverOptions {
                url: Some(url),
                ..Default::default()
            },
            DriverConstructorOptions::Options(kv) => DriverOptions {
                url: Some(
                    kv.get(SQLX_OPTION_URL)
                        .ok_or_else(|| anyhow!("missing {SQLX_OPTION_URL}"))
                        .and_then(|value| {
                            if let Value::Str(s) = value {
                                Ok(s.clone())
                            } else {
                                Err(anyhow!("{SQLX_OPTION_URL} must be a string"))
                            }
                        })?,
                ),
                ast_cache_shard_count: kv.get(SQLX_OPTION_AST_CACHE_SHARD_COUNT).map_or(
                    Ok(8),
                    |value| {
                        if let Value::Int(n) = value {
                            Ok(*n as usize)
                        } else {
                            Err(anyhow!(
                                "{SQLX_OPTION_AST_CACHE_SHARD_COUNT} must be an integer"
                            ))
                        }
                    },
                )?,
                ast_cache_shard_size: kv.get(SQLX_OPTION_AST_CACHE_SHARD_SIZE).map_or(
                    Ok(8),
                    |value| {
                        if let Value::Int(n) = value {
                            Ok(*n as usize)
                        } else {
                            Err(anyhow!(
                                "{SQLX_OPTION_AST_CACHE_SHARD_SIZE} must be an integer"
                            ))
                        }
                    },
                )?,
                persistent_name: match kv.get(SQLX_OPTION_PERSISTENT_NAME) {
                    None => None,
                    Some(value) => {
                        if let Value::Str(str) = value {
                            Some(str.clone())
                        } else {
                            return Err(anyhow!(
                                "{SQLX_OPTION_PERSISTENT_NAME} must be an integer"
                            ));
                        }
                    }
                },
            },
        };

        if let Some(name) = options.persistent_name.as_ref() {
            if let Some(inner) = PERSISTENT_DRIVER_REGISTRY.get(name) {
                return Ok(Self {
                    inner: inner.clone(),
                });
            }
        }
        let inner = Arc::new(DriverInner {
            pool: crate::RUNTIME.block_on(
                PgPoolOptions::new().max_connections(5).connect(
                    options
                        .url
                        .ok_or_else(|| anyhow!("URL must be set"))?
                        .as_str(),
                ),
            )?,
            ast_cache: LruCache::new(options.ast_cache_shard_count, options.ast_cache_shard_size),
        });
        if let Some(name) = options.persistent_name {
            PERSISTENT_DRIVER_REGISTRY.insert(name, inner.clone());
        }
        Ok(Self { inner })
    }

    /// Execute a query and return a single row.
    pub fn query_one(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.inner.query_one(query, parameters)
    }

    /// Execute a query and return all matching rows.
    pub fn query_all(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Vec<Zval>> {
        self.inner.query_all(query, parameters)
    }

    /// Prepare a reusable query object with driver context.
    ///
    /// # Parameters
    /// - `query`: SQL string to prepare.
    #[must_use]
    pub fn prepare(&self, query: &str) -> PreparedQuery {
        PreparedQuery {
            driver_inner: self.inner.clone(),
            query: query.to_owned(),
        }
    }

    /// Execute a query (e.g., INSERT/UPDATE) and return the number of rows affected.
    pub fn execute(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<u64> {
        self.inner.execute(query, parameters)
    }

    /// Insert data into a table using a map of fields and values.
    ///
    /// # Parameters
    /// - `table`: Table name.
    /// - `fields`: Map of column names to values.
    ///
    /// # Returns
    /// Number of inserted rows.
    pub fn insert(&self, table: &str, fields: HashMap<String, Value>) -> anyhow::Result<u64> {
        self.execute(
            &format!(
                "INSERT INTO {table} SET {}",
                fields.keys().map(|k| format!("{k} = ${k}")).join(", ")
            ),
            Some(fields),
        )
    }
}

/// A prepared SQL query that holds the query string and a reference to the driver.
#[php_class]
pub struct PreparedQuery {
    query: String,
    driver_inner: Arc<DriverInner>,
}

#[php_impl]
impl PreparedQuery {
    /// Executes the prepared query with the given parameters.
    ///
    /// # Parameters
    /// - `parameters`: Optional map of parameter names to values.
    ///
    /// # Returns
    /// Number of rows affected.
    pub fn execute(&self, parameters: Option<HashMap<String, Value>>) -> anyhow::Result<u64> {
        self.driver_inner.execute(self.query.as_str(), parameters)
    }

    /// Executes a new query using the same driver, returning a single row.
    ///
    /// # Parameters
    /// - `query`: SQL query string.
    /// - `parameters`: Optional map of parameter names to values.
    ///
    /// # Returns
    /// A PHP value (usually an associative array).
    pub fn query_one(&self, parameters: Option<HashMap<String, Value>>) -> anyhow::Result<Zval> {
        self.driver_inner.query_one(&self.query, parameters)
    }

    /// Executes a new query using the same driver, returning all rows.
    ///
    /// # Parameters
    /// - `query`: SQL query string.
    /// - `parameters`: Optional map of parameter names to values.
    ///
    /// # Returns
    /// A list of PHP values.
    pub fn query_all(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Vec<Zval>> {
        self.driver_inner.query_all(&self.query, parameters)
    }
}

/// Registers the PHP module.
#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
