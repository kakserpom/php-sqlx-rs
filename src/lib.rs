#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::must_use_candidate)]
#![cfg_attr(windows, feature(abi_vectorcall))]

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod ast;
pub mod byclause;
pub mod conversion;
mod driver;
mod inner_driver;
#[cfg(feature = "lazy-row")]
mod lazy_row;
#[cfg(feature = "mssql")]
pub mod mssql;
#[cfg(feature = "mysql")]
pub mod mysql;
pub mod options;
pub mod paginateclause;
pub mod paramvalue;
#[cfg(feature = "postgres")]
pub mod postgres;
mod prepared_query;
pub mod query_builder;
pub mod selectclause;
#[cfg(test)]
mod tests;

pub mod utils;

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
const DEFAULT_MIN_CONNECTIONS: u32 = 0;
const DEFAULT_ASSOC_ARRAYS: bool = false;
const DEFAULT_COLLAPSIBLE_IN: bool = true;
const DEFAULT_TEST_BEFORE_ACQUIRE: bool = false;

use crate::options::DriverOptions;

#[cfg(feature = "mysql")]
pub use mysql::{MySqlDriver, MySqlPreparedQuery};

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    let module = selectclause::build(module);
    let module = byclause::build(module);
    let module = paginateclause::build(module);
    let module = query_builder::build(module);

    #[cfg(feature = "mysql")]
    let module = mysql::build(module);

    #[cfg(feature = "postgres")]
    let module = postgres::build(module);

    #[cfg(feature = "mssql")]
    let module = mssql::build(module);

    #[cfg(feature = "lazy-row")]
    let module = lazy_row::build(module);

    module.class::<DriverOptions>()
}
