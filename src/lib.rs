#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::must_use_candidate)]
#![cfg_attr(windows, feature(abi_vectorcall))]
pub mod byclause;
pub mod driver;
pub mod selectclause;
mod tests;
mod utils;

use crate::byclause::{ByClause, OrderFieldDefinition, ByClauseRendered};
use crate::selectclause::{SelectClause, SelectClauseRendered};
use ext_php_rs::prelude::*;
use std::sync::LazyLock;
use tokio::runtime::Runtime;

/// Global runtime for executing async `SQLx` queries from sync context.
static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

const DEFAULT_AST_CACHE_SHARD_COUNT: usize = 8;
const DEFAULT_AST_CACHE_SHARD_SIZE: usize = 256;
const DEFAULT_MAX_CONNECTIONS: NonZeroU32 = NonZeroU32::new(2).unwrap();
const DEFAULT_ASSOC_ARRAYS: bool = false;
const DEFAULT_COLLAPSIBLE_IN: bool = true;

use ext_php_rs::types::Zval;

#[derive(Debug, ZvalConvert)]
pub enum ColumnArgument<'a> {
    Index(usize),
    Name(&'a str),
}
#[allow(clippy::wildcard_imports)]
use crate::driver::*;
use std::collections::HashMap;
use std::num::NonZeroU32;

/// Registers the PHP module.
#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
