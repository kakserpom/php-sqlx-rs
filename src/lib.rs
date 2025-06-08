#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::too_many_lines)]
#![cfg_attr(windows, feature(abi_vectorcall))]
mod ast;

use crate::ast::{Ast, Value};
use ext_php_rs::convert::FromZvalMut;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::flags::DataType;
use ext_php_rs::{prelude::*, types::Zval};
use itertools::Itertools;
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{Column, Postgres, Row};
use sqlx_core::database::Database;
use sqlx_core::query::Query;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::LazyLock;
use threadsafe_lru::LruCache;
use tokio::runtime::Runtime;
static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum QueryParamKey {
    String(String),
    //Int(i64),
}

impl FromZvalMut<'_> for QueryParamKey {
    const TYPE: DataType = DataType::String;

    fn from_zval_mut(zval: &'_ mut Zval) -> Option<Self> {
        if zval.is_string() {
            Some(QueryParamKey::String(zval.string()?))
        } else {
            None //Some(QueryParamKey::Int(zval.long()?.into()))
        }
    }
}

#[php_class]
pub struct Sqlx {
    inner: sqlx::PgPool,
    ast_cache: LruCache<String, Ast>,
}
trait RowToZval: Row {
    fn into_zval(self) -> Zval;
}
impl RowToZval for PgRow {
    fn into_zval(self) -> Zval {
        HashMap::from_iter(self.columns().into_iter().map(|column| {
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
        }))
        .into_zval(false)
        .unwrap()
    }
}

fn bind_values<'a>(
    query: Query<'a, Postgres, <Postgres as Database>::Arguments<'_>>,
    values: &'a [Value],
) -> Query<'a, Postgres, <Postgres as Database>::Arguments<'a>> {
    fn walker<'a>(
        q: Query<'a, Postgres, <Postgres as Database>::Arguments<'a>>,
        value: &'a Value,
    ) -> Query<'a, Postgres, <Postgres as Database>::Arguments<'a>> {
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

impl Sqlx {
    fn render_query(
        &self,
        query: &str,
        params: Option<HashMap<String, Value>>,
    ) -> (String, Vec<Value>) {
        let params = params.unwrap_or_default();
        if let Some(ast) = self.ast_cache.get(query) {
            ast.render(params)
        } else {
            let ast = Ast::parse(query).unwrap();
            let rendered = ast.render(params);
            self.ast_cache.insert(query.to_owned(), ast);
            rendered
        }
    }
}

#[php_impl]
impl Sqlx {
    pub fn __construct() -> Sqlx {
        Self {
            ast_cache: LruCache::new(4, 2),
            inner: RUNTIME
                .block_on(
                    PgPoolOptions::new()
                        .max_connections(5)
                        .connect("postgres://localhost/postgres"),
                )
                .unwrap(),
        }
    }

    pub fn query_one(&mut self, query: &str, params: Option<HashMap<String, Value>>) -> Zval {
        let (query, values) = self.render_query(query, params);

        RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_one(&self.inner))
            .unwrap()
            .into_zval()
    }

    pub fn query_all(&mut self, query: &str, params: Option<HashMap<String, Value>>) -> Vec<Zval> {
        let (query, values) = self.render_query(query, params);

        println!("query: {query:?}");
        println!("values: {values:?}");
        RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_all(&self.inner))
            .unwrap()
            .into_iter()
            .map(PgRow::into_zval)
            .collect()
    }

    pub fn execute(&mut self, query: &str, params: Option<HashMap<String, Value>>) -> u64 {
        let (query, values) = self.render_query(query, params);
        RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).execute(&self.inner))
            .unwrap()
            .rows_affected()
    }

    pub fn insert(&mut self, table: &str, fields: HashMap<String, Value>) -> u64 {
        let query = format!(
            "INSERT INTO {table} SET {}",
            fields.keys().map(|k| format!("{k} = ${k}")).join(", ")
        );
        self.execute(&query, Some(fields))
    }
}
// Required to register the extension with PHP.
#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
