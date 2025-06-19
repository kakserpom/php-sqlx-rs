#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::must_use_candidate)]
#![cfg_attr(windows, feature(abi_vectorcall))]
pub mod byclause;
pub mod conversion;
#[cfg(feature = "mysql")]
pub mod mysql;
pub mod options;
#[cfg(feature = "postgres")]
pub mod postgres;
pub mod selectclause;
mod tests;
mod utils;

use crate::byclause::{ByClause, ByClauseFieldDefinition, ByClauseRendered};
use crate::selectclause::{SelectClause, SelectClauseRendered};
use crate::utils::ColumnArgument;
use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::runtime::Runtime;

/// Global runtime for executing async `SQLx` queries from sync context.
static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

const DEFAULT_AST_CACHE_SHARD_COUNT: usize = 8;
const DEFAULT_AST_CACHE_SHARD_SIZE: usize = 256;
const DEFAULT_MAX_CONNECTIONS: NonZeroU32 = NonZeroU32::new(2).unwrap();
const DEFAULT_ASSOC_ARRAYS: bool = false;
const DEFAULT_COLLAPSIBLE_IN: bool = true;

use crate::options::DriverOptions;

#[cfg(feature = "mysql")]
pub use mysql::{MySqlDriver, MySqlDriverOptions, MySqlParameterValue, MySqlPreparedQuery};
#[cfg(feature = "postgres")]
pub use postgres::{PgDriver, PgDriverOptions, PgParameterValue, PgPreparedQuery};

use std::num::NonZeroU32;

/// Registers the PHP module.
#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
