pub mod ast;
mod conversion;
pub mod prepared_query;

use crate::postgres::inner::PgDriverInner;
pub use crate::postgres::prepared_query::PgPreparedQuery;
use crate::utils::ColumnArgument;

use crate::options::DriverOptionsArg;
use crate::paramvalue::ParameterValue;
use crate::php_sqlx_impl_driver;
use anyhow::anyhow;
use dashmap::DashMap;
use ext_php_rs::builders::ModuleBuilder;
use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::{php_class, php_impl};
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
pub mod inner;

static PERSISTENT_DRIVER_REGISTRY: LazyLock<DashMap<String, Arc<PgDriverInner>>> =
    LazyLock::new(DashMap::new);

/// A Postgres driver using SQLx with query helpers and AST cache.
///
/// This class supports prepared queries, persistent connections, and augmented SQL.
#[php_class]
#[php(name = "Sqlx\\PgDriver")]
#[php(rename = "none")]
#[derive(Clone)]
pub struct PgDriver {
    pub driver_inner: Arc<PgDriverInner>,
}

php_sqlx_impl_driver!(PgDriver, PgDriverInner, PgPreparedQuery);

pub fn build(module: ModuleBuilder) -> ModuleBuilder {
    module.class::<PgDriver>().class::<PgPreparedQuery>()
}
