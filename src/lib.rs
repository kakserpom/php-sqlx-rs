#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::too_many_lines)]
#![cfg_attr(windows, feature(abi_vectorcall))]
mod ast;
use crate::ast::{Ast, Value};
use anyhow::anyhow;
use ext_php_rs::boxed::ZBox;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::types::ZendClassObject;
use ext_php_rs::{prelude::*, types::Zval};
use itertools::Itertools;
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{Column, Row};
use sqlx_core::database::Database;
use sqlx_core::query::Query;
use std::collections::HashMap;
use std::sync::LazyLock;
use threadsafe_lru::LruCache;
use tokio::runtime::Runtime;
static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());
#[php_class]
pub struct PreparedQuery {
    query: String,
}
#[php_impl]
impl PreparedQuery {
    pub fn execute(
        self_: &ZendClassObject<Self>,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<u64> {
        self_
            .std
            .get_property::<&Sqlx>("driver")
            .unwrap()
            .execute(self_.query.as_str(), parameters)
    }

    pub fn query_one(
        self_: &ZendClassObject<Self>,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        self_
            .std
            .get_property::<&Sqlx>("driver")
            .unwrap()
            .query_one(query, parameters)
    }

    pub fn query_all(
        self_: &ZendClassObject<Self>,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Vec<Zval>> {
        self_
            .std
            .get_property::<&Sqlx>("driver")
            .unwrap()
            .query_all(query, parameters)
    }
}

#[php_class]
pub struct Sqlx {
    inner: sqlx::PgPool,
    ast_cache: LruCache<String, Ast>,
}
trait RowToZval: Row {
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

impl Sqlx {
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
}

#[php_impl]
impl Sqlx {
    pub fn __construct(url: &str) -> anyhow::Result<Sqlx> {
        Ok(Self {
            ast_cache: LruCache::new(4, 2),
            inner: RUNTIME.block_on(PgPoolOptions::new().max_connections(5).connect(url))?,
        })
    }

    pub fn query_one(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Zval> {
        let (query, values) = self.render_query(query, parameters);

        RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_one(&self.inner))?
            .into_zval()
    }

    pub fn query_all(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<Vec<Zval>> {
        let (query, values) = self.render_query(query, parameters);
        RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).fetch_all(&self.inner))?
            .into_iter()
            .map(PgRow::into_zval)
            .try_collect()
    }

    pub fn prepare(
        &self,
        self_: &mut ZendClassObject<Self>,
        query: &str,
    ) -> anyhow::Result<ZBox<ZendClassObject<PreparedQuery>>> {
        let mut prepared_query = ZendClassObject::<PreparedQuery>::new(PreparedQuery {
            query: query.to_owned(),
        });
        prepared_query
            .std
            .set_property("driver", self_)
            .map_err(|err| anyhow!("{:?}", err))?;
        Ok(prepared_query)
    }

    pub fn execute(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<u64> {
        let (query, values) = self.render_query(query, parameters);
        Ok(RUNTIME
            .block_on(bind_values(sqlx::query(&query), &values).execute(&self.inner))?
            .rows_affected())
    }

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

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
