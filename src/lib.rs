#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::must_use_candidate)]
#![cfg_attr(windows, feature(abi_vectorcall))]
pub mod byclause;
pub mod conversion;
#[cfg(feature = "lazy-row")]
mod lazy_row;
#[cfg(feature = "mysql")]
pub mod mysql;
pub mod options;
pub mod paginateclause;
mod paramvalue;
#[cfg(feature = "postgres")]
pub mod postgres;
pub mod selectclause;
mod tests;
mod utils;
mod driver;
mod prepared_query;
mod inner_driver;
pub mod ast;

use ext_php_rs::prelude::*;
#[cfg(feature = "lazy-row")]
pub use lazy_row::{LazyRow, LazyRowJson};
use std::num::NonZeroU32;
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
pub use mysql::{MySqlDriver, MySqlPreparedQuery};

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    let module = selectclause::build(module);
    let module = byclause::build(module);
    let module = paginateclause::build(module);

    #[cfg(feature = "mysql")]
    let module = mysql::build(module);

    #[cfg(feature = "postgres")]
    let module = postgres::build(module);

    #[cfg(feature = "lazy-row")]
    let module = lazy_row::build(module);

    module.class::<DriverOptions>()
}
