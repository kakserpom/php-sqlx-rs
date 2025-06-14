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
use ext_php_rs::ffi::zend_object;
use ext_php_rs::{prelude::*, types::Zval};
use itertools::Itertools;
use sqlx::postgres::{PgPoolOptions, PgRow};
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
    /// ASCending order (A to Z)
    const _ASC: &'static str = "ASC";
    /// DESCending order (Z to A)
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
    /// ASCending order (A to Z)
    const ASC: &'static str = "ASC";
    /// DESCending order (Z to A)
    const DESC: &'static str = "DESC";

    /// Constructs an OrderBy helper with allowed sortable fields.
    ///
    /// # Parameters
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
        self.apply(order_by)
    }

    /// Applies ordering rules to a user-defined input.
    ///
    /// # Parameters
    /// - `order_by`: List of fields (as strings or [field, direction] arrays)
    ///
    /// # Returns
    /// A `RenderedOrderBy` object containing validated SQL ORDER BY clauses
    /// The returning value is to be used as a placeholder value
    ///
    /// # Errors
    /// This method does not return an error but silently ignores unknown fields.
    /// Use validation separately if strict input is required.
    #[must_use]
    pub fn apply(&self, order_by: Vec<OrderFieldDefinition>) -> RenderedOrderBy {
        self._apply(order_by)
    }
}
impl OrderBy {
    #[must_use]
    pub fn _apply(&self, order_by: Vec<OrderFieldDefinition>) -> RenderedOrderBy {
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
                    if let Some(x) = self.defined_fields.get(&field) {
                        Some(format!("{} {dir}", x.as_ref().unwrap_or(&field)))
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
    pub inner: Arc<DriverInner>,
}
pub struct DriverInner {
    pub pool: sqlx::PgPool,
    pub ast_cache: LruCache<String, Ast>,
    pub options: DriverOptions,
}

impl DriverInner {
    /// Executes an INSERT/UPDATE/DELETE query and returns affected row count.
    ///
    /// # Parameters
    /// - `query`: SQL query string
    /// - `parameters`: Optional map of named parameters
    ///
    /// # Returns
    /// Number of affected rows
    ///
    /// # Errors
    /// Returns an error if:
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

    /// Executes the prepared query and returns a single result.
    ///
    /// # Parameters
    /// - `parameters`: Optional map of named parameters
    ///
    /// # Returns
    /// Single row as array or object depending on config
    ///
    /// # Errors
    /// Returns an error if:
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

    /// Execute a query and return at most one row.
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

    /// Executes a SQL query and returns all results.
    ///
    /// # Parameters
    /// - `query`: SQL query string
    /// - `parameters`: Optional map of named parameters
    ///
    /// # Returns
    /// Array of rows as array or object depending on config
    ///
    /// # Errors
    /// Returns an error if:
    /// - SQL query is invalid or fails to execute;
    /// - parameter binding fails;
    /// - row decoding fails due to an unsupported or mismatched database type;
    /// - conversion to PHP Zval fails (e.g., due to memory or encoding issues).

    pub fn query_all(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
        associative_arrays: Option<bool>,
    ) -> anyhow::Result<Vec<Zval>> {
        let (query, values) = self.render_query(query, parameters)?;
        RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_all(&self.pool))?
            .into_iter()
            .map(|x| {
                PgRow::into_zval(
                    x,
                    associative_arrays.unwrap_or(self.options.associative_arrays),
                )
            })
            .try_collect()
    }

    /// Executes a SQL query and returns the rendered query and its parameters.
    ///
    /// This method does not execute the query but returns the SQL string with placeholders
    /// and the bound parameter values for debugging or logging purposes.
    ///
    /// # Parameters
    /// - `query`: SQL query string
    /// - `parameters`: Optional map of named parameters
    ///
    /// # Returns
    /// A list where the first element is the rendered SQL query (string), and the second is an array of bound values
    ///
    /// # Errors
    /// Returns an error if the query can't be parsed, rendered, or if parameters
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
}

/// Trait to convert a row into a PHP value.
trait RowToZval: Row {
    /// Convert the row into a PHP `Zval` associative array.
    fn into_zval(self, associative_arrays: bool) -> anyhow::Result<Zval>;
}

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

impl RowToZval for PgRow {
    fn into_zval(self, associative_arrays: bool) -> anyhow::Result<Zval> {
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

        let row = self.columns().iter().try_fold(
            HashMap::<String, Zval>::with_capacity(self.len()),
            |mut row, column| {
                let name = column.name();
                let column_type = column.type_info().name();
                let value =
                    match column_type {
                        "BOOL" => try_cast_into_zval::<bool>(&self, name)?,
                        "BYTEA" | "BINARY" => self
                            .try_get::<&[u8], _>(name)
                            .map_err(|err| anyhow!("{err:?}"))
                            .map(|x| x.iter().copied().collect::<Binary<_>>())?
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
                            .map(|x| json_into_zval(x, associative_arrays))?
                            .into_zval(false)
                            .map_err(|err| anyhow!("{err:?}"))?,
                        "_JSON" | "_JSONB" => self
                            .try_get::<Vec<serde_json::Value>, _>(name)
                            .map_err(|err| anyhow!("{err:?}"))
                            .map(|x| {
                                x.into_iter()
                                    .map(|x| json_into_zval(x, associative_arrays))
                                    .collect::<Vec<_>>()
                            })?
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
                        "INT4RANGE" | "NUMRANGE" | "TSRANGE" | "TSTZRANGE" | "DATERANGE"
                        | "INT8RANGE" => try_cast_into_zval::<String>(&self, name)?,
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
                        "_DATE" | "_TIME" | "_TIMESTAMP" | "_TIMESTAMPTZ" | "_INTERVAL"
                        | "_TIMETZ" => try_cast_into_zval::<Vec<String>>(&self, name)?,
                        "_INET" | "_CIDR" | "_MACADDR" | "_MACADDR8" => {
                            try_cast_into_zval::<Vec<String>>(&self, name)?
                        }
                        "_BIT" | "_VARBIT" => try_cast_into_zval::<Vec<String>>(&self, name)?,
                        "_POINT" | "_LSEG" | "_PATH" | "_BOX" | "_POLYGON" | "_LINE"
                        | "_CIRCLE" => try_cast_into_zval::<Vec<String>>(&self, name)?,
                        "_INT4RANGE" | "_NUMRANGE" | "_TSRANGE" | "_TSTZRANGE" | "_DATERANGE"
                        | "_INT8RANGE" => try_cast_into_zval::<Vec<String>>(&self, name)?,
                        "_RECORD" => try_cast_into_zval::<Vec<String>>(&self, name)?,
                        "_JSONPATH" => try_cast_into_zval::<Vec<String>>(&self, name)?,

                        _ => bail!("unsupported type: {column_type}"),
                    };
                row.insert(name.to_string(), value);
                Ok(row)
            },
        )?;
        if associative_arrays {
            row.into_zval(false).map_err(|err| anyhow!("{:?}", err))
        } else {
            Ok(row
                .into_iter()
                .try_fold(
                    zend_object::new_stdclass(),
                    |mut std_object, (key, value)| {
                        std_object
                            .set_property(&key, value)
                            .map(|()| std_object)
                            .map_err(|err| anyhow!("{:?}", err))
                    },
                )?
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
    /// # Parameters
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
            if let Some(inner) = PERSISTENT_DRIVER_REGISTRY.get(name) {
                return Ok(Self {
                    inner: inner.clone(),
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
        let inner = Arc::new(DriverInner {
            pool,
            ast_cache: LruCache::new(options.ast_cache_shard_count, options.ast_cache_shard_size),
            options,
        });
        if let Some(name) = persistent_name {
            PERSISTENT_DRIVER_REGISTRY.insert(name, inner.clone());
        }
        Ok(Self { inner })
    }

    /// Returns whether results are returned as associative arrays.
    ///
    /// If true, result rows are returned as PHP associative arrays (key-value pairs).
    /// If false, result rows are returned as PHP `stdClass` objects.
    #[getter]
    fn assoc_arrays(&self) -> bool {
        self.inner.options.associative_arrays
    }

    /// Executes a SQL query and returns a single result.
    ///
    /// # Parameters
    /// - `query`: SQL query string
    /// - `parameters`: Optional map of named parameters
    ///
    /// # Returns
    /// Single row as array or object depending on config
    ///
    /// # Errors
    /// Returns an error if:
    /// - SQL query is invalid or execution fails;
    /// - a parameter cannot be bound or has incorrect type;
    /// - the row contains unsupported database types;
    /// - conversion to PHP object fails.
    pub fn query_row(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.inner.query_row(query, parameters, None)
    }

    /// Executes a SQL query and returns one row as an associative array.
    ///
    /// # Parameters
    /// - `query`: SQL query string
    /// - `parameters`: Optional map of named parameters
    ///
    /// # Errors
    /// Returns an error if:
    /// - SQL query is invalid or execution fails;
    /// - a parameter cannot be bound or has incorrect type;
    /// - the row contains unsupported database types;
    /// - conversion to PHP object fails.
    pub fn query_row_assoc(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.inner.query_row(query, parameters, Some(true))
    }

    /// Executes a SQL query and returns one row as an object.
    ///
    /// # Parameters
    /// - `query`: SQL query string
    /// - `parameters`: Optional map of named parameters
    ///
    /// # Errors
    /// Returns an error if:
    /// - SQL query is invalid or execution fails;
    /// - a parameter cannot be bound or has incorrect type;
    /// - the row contains unsupported database types;
    /// - conversion to PHP object fails.
    pub fn query_row_obj(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.inner.query_row(query, parameters, Some(false))
    }

    /// Executes a SQL query and returns a single result.
    ///
    /// # Parameters
    /// - `query`: SQL query string
    /// - `parameters`: Optional map of named parameters
    ///
    /// # Returns
    /// Single row as array or object depending on config
    ///
    /// # Errors
    /// Returns an error if the query fails for reasons other than no matching rows.
    /// For example, syntax errors, type mismatches, or database connection issues.
    pub fn query_maybe_row(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.inner.query_maybe_row(query, parameters, None)
    }

    /// Executes a SQL query and returns one row as an associative array.
    ///
    /// # Parameters
    /// - `query`: SQL query string
    /// - `parameters`: Optional map of named parameters
    pub fn query_row_maybe_assoc(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.inner.query_maybe_row(query, parameters, Some(true))
    }

    /// Executes a SQL query and returns one row as an object.
    ///
    /// # Parameters
    /// - `query`: SQL query string
    /// - `parameters`: Optional map of named parameters
    pub fn query_maybe_row_obj(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.inner.query_maybe_row(query, parameters, Some(false))
    }

    /// Executes a SQL query and returns all results.
    ///
    /// # Parameters
    /// - `query`: SQL query string
    /// - `parameters`: Optional map of named parameters
    ///
    /// # Returns
    /// Array of rows as array or object depending on config
    ///
    /// # Errors
    /// Returns an error if:
    /// - SQL query is invalid or fails to execute;
    /// - parameter binding fails;
    /// - row decoding fails due to an unsupported or mismatched database type;
    /// - conversion to PHP Zval fails (e.g., due to memory or encoding issues).
    pub fn query_all(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Vec<Zval>> {
        self.inner.query_all(query, parameters, None)
    }

    /// Executes a SQL query and returns all rows as associative arrays.
    ///
    /// # Parameters
    /// - `query`: SQL query string
    /// - `parameters`: Optional map of named parameters
    ///
    /// # Errors
    /// Returns an error if:
    /// - SQL query is invalid or fails to execute;
    /// - parameter binding fails;
    /// - row decoding fails due to an unsupported or mismatched database type;
    /// - conversion to PHP Zval fails (e.g., due to memory or encoding issues).
    pub fn query_all_assoc(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Vec<Zval>> {
        self.inner.query_all(query, parameters, Some(true))
    }

    /// Executes a SQL query and returns all rows as objects.
    ///
    /// # Parameters
    /// - `query`: SQL query string
    /// - `parameters`: Optional map of named parameters
    ///
    /// # Errors
    /// Returns an error if:
    /// - SQL query is invalid or fails to execute;
    /// - parameter binding fails;
    /// - row decoding fails due to an unsupported or mismatched database type;
    /// - conversion to PHP Zval fails (e.g., due to memory or encoding issues).
    pub fn query_all_obj(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Vec<Zval>> {
        self.inner.query_all(query, parameters, Some(false))
    }

    /// Creates a prepared query object with the given SQL string.
    ///
    /// # Parameters
    /// - `query`: SQL query string to prepare
    ///
    /// # Returns
    /// Prepared query object
    #[must_use]
    pub fn prepare(&self, query: &str) -> PreparedQuery {
        PreparedQuery {
            driver_inner: self.inner.clone(),
            query: query.to_owned(),
        }
    }

    /// Executes an INSERT/UPDATE/DELETE query and returns affected row count.
    ///
    /// # Parameters
    /// - `query`: SQL query string
    /// - `parameters`: Optional map of named parameters
    ///
    /// # Returns
    /// Number of affected rows
    ///
    /// # Errors
    /// Returns an error if:
    /// - the SQL query is invalid or fails to execute (e.g., due to syntax error, constraint violation, or connection issue);
    /// - parameters contain unsupported types or fail to bind correctly;
    /// - the runtime fails to execute the query (e.g., task panic or timeout).
    pub fn execute(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<u64> {
        self.inner.execute(query, parameters)
    }

    /// Inserts a row into the given table using a map of fields.
    ///
    /// # Parameters
    /// - `table`: Table name
    /// - `row`: Map of column names to values
    ///
    /// # Returns
    /// Number of inserted rows
    ///
    /// # Errors
    /// Returns an error if:
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

    /// Executes a SQL query and returns the rendered query and its parameters.
    ///
    /// This method does not execute the query but returns the SQL string with placeholders
    /// and the bound parameter values for debugging or logging purposes.
    ///
    /// # Parameters
    /// - `query`: SQL query string
    /// - `parameters`: Optional map of named parameters
    ///
    /// # Returns
    /// A list where the first element is the rendered SQL query (string), and the second is an array of bound values
    ///
    /// # Errors
    /// Returns an error if the query can't be parsed, rendered, or if parameters
    /// cannot be converted from PHP values.
    pub fn dry(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Vec<Zval>> {
        self.inner.dry(query, parameters)
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
    /// Executes the prepared query with optional parameters.
    ///
    /// # Parameters
    /// - `parameters`: Optional map of named parameters
    ///
    /// # Returns
    /// Number of affected rows
    ///
    /// # Errors
    /// Returns an error if:
    /// - the SQL query is invalid or fails to execute (e.g., due to syntax error, constraint violation, or connection issue);
    /// - parameters contain unsupported types or fail to bind correctly;
    /// - the runtime fails to execute the query (e.g., task panic or timeout).
    pub fn execute(&self, parameters: Option<HashMap<String, Value>>) -> anyhow::Result<u64> {
        self.driver_inner.execute(self.query.as_str(), parameters)
    }

    /// Executes the prepared query and returns a single result.
    ///
    /// # Parameters
    /// - `parameters`: Optional map of named parameters
    ///
    /// # Returns
    /// Single row as array or object depending on config
    ///
    /// # Errors
    /// Returns an error if:
    /// - SQL query is invalid or execution fails;
    /// - a parameter cannot be bound or has incorrect type;
    /// - the row contains unsupported database types;
    /// - conversion to PHP object fails.
    pub fn query_row(&self, parameters: Option<HashMap<String, Value>>) -> anyhow::Result<Zval> {
        self.driver_inner.query_row(&self.query, parameters, None)
    }

    /// Executes the prepared query and returns one row as an associative array.
    ///
    /// # Parameters
    /// - `parameters`: Optional map of named parameters
    pub fn query_row_assoc(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_row(&self.query, parameters, Some(true))
    }

    /// Executes the prepared query and returns one row as an object.
    ///
    /// # Parameters
    /// - `parameters`: Optional map of named parameters
    pub fn query_row_obj(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self.driver_inner
            .query_row(&self.query, parameters, Some(false))
    }

    /// Executes the prepared query and returns all rows.
    ///
    /// # Parameters
    /// - `parameters`: Optional map of named parameters
    ///
    /// # Returns
    /// Array of rows as array or object depending on config
    ///
    /// # Errors
    /// Returns an error if:
    /// - SQL query is invalid or fails to execute;
    /// - parameter binding fails;
    /// - row decoding fails due to an unsupported or mismatched database type;
    /// - conversion to PHP Zval fails (e.g., due to memory or encoding issues).
    pub fn query_all(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Vec<Zval>> {
        self.driver_inner.query_all(&self.query, parameters, None)
    }

    /// Executes the prepared query and returns all rows as associative arrays.
    ///
    /// # Parameters
    /// - `parameters`: Optional map of named parameters
    ///
    /// # Errors
    /// Returns an error if:
    /// - SQL query is invalid or fails to execute;
    /// - parameter binding fails;
    /// - row decoding fails due to an unsupported or mismatched database type;
    /// - conversion to PHP Zval fails (e.g., due to memory or encoding issues).
    pub fn query_all_assoc(
        &self,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Vec<Zval>> {
        self.driver_inner
            .query_all(&self.query, parameters, Some(true))
    }

    /// Executes the prepared query and returns all rows as objects.
    ///
    /// # Parameters
    /// - `parameters`: Optional map of named parameters
    ///
    /// # Errors
    /// Returns an error if:
    /// - SQL query is invalid or fails to execute;
    /// - parameter binding fails;
    /// - row decoding fails due to an unsupported or mismatched database type;
    /// - conversion to PHP Zval fails (e.g., due to memory or encoding issues).
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
