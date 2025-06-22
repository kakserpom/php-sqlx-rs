pub mod ast;
pub use crate::mysql::inner::MySqlDriverInner;
use crate::utils::ColumnArgument;
use dashmap::DashMap;
use ext_php_rs::prelude::ModuleBuilder;
use ext_php_rs::types::Zval;
use ext_php_rs::{php_class, php_impl};
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
mod conversion;
pub mod inner;
pub mod prepared_query;
use crate::options::DriverOptionsArg;
use crate::paramvalue::ParameterValue;
use crate::php_sqlx_impl_driver;
use anyhow::anyhow;
use ext_php_rs::prelude::*;
pub use prepared_query::*;

static PERSISTENT_DRIVER_REGISTRY: LazyLock<DashMap<String, Arc<MySqlDriverInner>>> =
    LazyLock::new(DashMap::new);

#[php_class]
#[php(name = "Sqlx\\MySqlDriver")]
#[php(rename = "none")]
#[derive(Clone)]
pub struct MySqlDriver {
    pub driver_inner: Arc<MySqlDriverInner>,
}

php_sqlx_impl_driver!(MySqlDriver, MySqlDriverInner, MySqlPreparedQuery);

pub fn build(module: ModuleBuilder) -> ModuleBuilder {
    module.class::<MySqlPreparedQuery>().class::<MySqlDriver>()
}
