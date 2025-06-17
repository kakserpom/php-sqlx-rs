#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]
#![cfg_attr(windows, feature(abi_vectorcall))]
pub mod driver;
pub mod orderby;
mod tests;

use crate::orderby::{OrderBy, OrderFieldDefinition, RenderedOrderBy};
use ext_php_rs::prelude::*;
use std::sync::LazyLock;
use tokio::runtime::Runtime;

/// Global runtime for executing async `SQLx` queries from sync context.
static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

const DEFAULT_AST_CACHE_SHARD_COUNT: usize = 8;
const DEFAULT_AST_CACHE_SHARD_SIZE: usize = 256;
use ext_php_rs::types::Zval;

pub trait ZvalNull {
    fn null() -> Zval;
}
impl ZvalNull for Zval {
    fn null() -> Zval {
        let mut zval = Zval::new();
        zval.set_null();
        zval
    }
}

#[derive(Debug, ZvalConvert)]
pub enum ColumnArgument<'a> {
    Index(usize),
    Name(&'a str),
}
#[allow(clippy::wildcard_imports)]
use crate::driver::*;
use std::collections::HashMap;

/// Registers the PHP module.
#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
