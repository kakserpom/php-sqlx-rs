#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::too_many_lines)]
#![cfg_attr(windows, feature(abi_vectorcall))]

mod ast;

use crate::ast::{Ast, Value};
use anyhow::{anyhow, bail};
use dashmap::DashMap;
use ext_php_rs::binary::Binary;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::{prelude::*, types::Zval};
use itertools::Itertools;
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{Column, Row};
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

fn json_into_zval(value: serde_json::Value) -> anyhow::Result<Zval> {
    match value {
        serde_json::Value::String(str) => str
            .into_zval(false)
            .map_err(|err| anyhow!("String: {err:?}")),
        serde_json::Value::Number(number) => number
            .to_string()
            .into_zval(false)
            .map_err(|err| anyhow!("Number: {err:?}")),
        serde_json::Value::Bool(bool) => bool
            .into_zval(false)
            .map_err(|err| anyhow!("Bool: {err:?}")),
        serde_json::Value::Null => {
            let mut null = Zval::new();
            null.set_null();
            Ok(null)
        }
        serde_json::Value::Array(array) => Ok(array
            .into_iter()
            .map(json_into_zval)
            .collect::<anyhow::Result<Vec<Zval>>>()?
            .into_zval(false)
            .map_err(|err| anyhow!("Bool: {err:?}"))?),
        serde_json::Value::Object(object) => Ok(object
            .into_iter()
            .map(|(key, value)| Ok((key, json_into_zval(value)?)))
            .collect::<anyhow::Result<HashMap<String, Zval>>>()?
            .into_zval(false)
            .map_err(|err| anyhow!("Bool: {err:?}"))?),
    }
}

impl RowToZval for PgRow {
    fn into_zval(self) -> anyhow::Result<Zval> {
        fn try_cast_into_zval<'r, T>(row: &'r PgRow, name: &str) -> anyhow::Result<Zval>
        where
            T: Decode<'r, <PgRow as Row>::Database> + Type<<PgRow as Row>::Database>,
            T: IntoZval,
        {
            Ok(row
                .try_get::<'r, T, _>(name)
                .map_err(|err| anyhow!("{err:?}"))?
                .into_zval(false)
                .map_err(|err| anyhow!("{err:?}"))?)
        }

        let mut row: HashMap<String, Zval> = HashMap::with_capacity(self.len());
        for column in self.columns() {
            let name = column.name();
            let value = match column.type_info().name().to_uppercase().as_str() {
                "BOOL" => try_cast_into_zval::<bool>(&self, name)?,
                "BYTEA" | "BINARY" => self
                    .try_get::<&[u8], _>(name)
                    .map_err(|err| anyhow!("{err:?}"))
                    .map(|x| Binary::from_iter(x.iter().copied()))?
                    .into_zval(false)
                    .map_err(|err| anyhow!("{err:?}"))?,
                "CHAR" | "NAME" | "TEXT" | "BPCHAR" | "VARCHAR" => {
                    try_cast_into_zval::<String>(&self, name)?
                }
                "INT2" => try_cast_into_zval::<i16>(&self, name)?,
                "INT4" | "INT" => try_cast_into_zval::<i32>(&self, name)?,
                "INT8" => try_cast_into_zval::<i64>(&self, name)?,
                "OID" => try_cast_into_zval::<i32>(&self, name)?,
                "FLOAT4" => try_cast_into_zval::<f32>(&self, name)?,
                "FLOAT8" | "F64" => try_cast_into_zval::<f64>(&self, name)?,
                "NUMERIC" | "MONEY" => try_cast_into_zval::<String>(&self, name)?,
                "UUID" => try_cast_into_zval::<String>(&self, name)?,
                "JSON" | "JSONB" => self
                    .try_get::<serde_json::Value, _>(name)
                    .map_err(|err| anyhow!("{err:?}"))
                    .map(json_into_zval)?
                    .into_zval(false)
                    .map_err(|err| anyhow!("{err:?}"))?,
                "DATE" | "TIME" | "TIMESTAMP" | "TIMESTAMPTZ" | "INTERVAL" | "TIMETZ" => {
                    try_cast_into_zval::<String>(&self, name)?
                }
                "INET" | "CIDR" | "MACADDR" | "MACADDR8" => {
                    try_cast_into_zval::<String>(&self, name)?
                }
                "BIT" | "VARBIT" => try_cast_into_zval::<String>(&self, name)?,
                "POINT" | "LSEG" | "PATH" | "BOX" | "POLYGON" | "LINE" | "CIRCLE" => {
                    try_cast_into_zval::<String>(&self, name)?
                }
                "INT4RANGE" | "NUMRANGE" | "TSRANGE" | "TSTZRANGE" | "DATERANGE" | "INT8RANGE" => {
                    try_cast_into_zval::<String>(&self, name)?
                }
                "RECORD" => try_cast_into_zval::<String>(&self, name)?,
                "JSONPATH" => try_cast_into_zval::<String>(&self, name)?,

                // массивы
                "_BOOL" => try_cast_into_zval::<Vec<bool>>(&self, name)?,
                "_BYTEA" => try_cast_into_zval::<Vec<Vec<u8>>>(&self, name)?,
                "_CHAR" | "_NAME" | "_TEXT" | "_BPCHAR" | "_VARCHAR" => {
                    try_cast_into_zval::<Vec<String>>(&self, name)?
                }
                "_INT2" => try_cast_into_zval::<Vec<i16>>(&self, name)?,
                "_INT4" => try_cast_into_zval::<Vec<i32>>(&self, name)?,
                "_INT8" => try_cast_into_zval::<Vec<i64>>(&self, name)?,
                "_OID" => try_cast_into_zval::<Vec<i32>>(&self, name)?,
                "_FLOAT4" => try_cast_into_zval::<Vec<f32>>(&self, name)?,
                "_FLOAT8" => try_cast_into_zval::<Vec<f64>>(&self, name)?,
                "_NUMERIC" | "_MONEY" => try_cast_into_zval::<Vec<String>>(&self, name)?,
                "_UUID" => try_cast_into_zval::<Vec<String>>(&self, name)?,
                "_JSON" | "_JSONB" => try_cast_into_zval::<Vec<String>>(&self, name)?,
                "_DATE" | "_TIME" | "_TIMESTAMP" | "_TIMESTAMPTZ" | "_INTERVAL" | "_TIMETZ" => {
                    try_cast_into_zval::<Vec<String>>(&self, name)?
                }
                "_INET" | "_CIDR" | "_MACADDR" | "_MACADDR8" => {
                    try_cast_into_zval::<Vec<String>>(&self, name)?
                }
                "_BIT" | "_VARBIT" => try_cast_into_zval::<Vec<String>>(&self, name)?,
                "_POINT" | "_LSEG" | "_PATH" | "_BOX" | "_POLYGON" | "_LINE" | "_CIRCLE" => {
                    try_cast_into_zval::<Vec<String>>(&self, name)?
                }
                "_INT4RANGE" | "_NUMRANGE" | "_TSRANGE" | "_TSTZRANGE" | "_DATERANGE"
                | "_INT8RANGE" => try_cast_into_zval::<Vec<String>>(&self, name)?,
                "_RECORD" => try_cast_into_zval::<Vec<String>>(&self, name)?,
                "_JSONPATH" => try_cast_into_zval::<Vec<String>>(&self, name)?,

                _ => bail!("unsupported type: {}", column.type_info().name()),
            };

            row.insert(name.to_string(), value);
        }
        row.into_zval(false).map_err(|err| anyhow!("{:?}", err))
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
