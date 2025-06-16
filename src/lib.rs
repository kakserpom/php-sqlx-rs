#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]
#![cfg_attr(windows, feature(abi_vectorcall))]

mod ast;
mod driver;
#[cfg(test)]
mod tests;
mod orderby;
mod prepared_query;

use crate::orderby::{OrderBy, OrderFieldDefinition, RenderedOrderBy};
use crate::prepared_query::PreparedQuery;
use crate::ast::Value;
use dashmap::DashMap;
pub use driver::*;
use ext_php_rs::{prelude::*, types::Zval};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use tokio::runtime::Runtime;

/// Global runtime for executing async `SQLx` queries from sync context.
static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());
static PERSISTENT_DRIVER_REGISTRY: LazyLock<DashMap<String, Arc<DriverInner>>> =
    LazyLock::new(DashMap::new);

const DEFAULT_AST_CACHE_SHARD_COUNT: usize = 8;
const DEFAULT_AST_CACHE_SHARD_SIZE: usize = 256;

#[derive(Debug, ZvalConvert)]
pub enum ColumnArgument<'a> {
    Index(usize),
    Name(&'a str),
}

/// Registers the PHP module.
#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
