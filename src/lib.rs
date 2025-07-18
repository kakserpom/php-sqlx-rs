#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::must_use_candidate)]
#![cfg_attr(windows, feature(abi_vectorcall))]

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod ast;
pub mod by_clause;
pub mod conversion;
mod driver;
mod inner_driver;
#[cfg(feature = "lazy-row")]
mod lazy_row;
pub mod options;
pub mod paginate_clause;
pub mod paramvalue;
mod prepared_query;
pub mod query_builder;
pub mod select_clause;

pub mod driver_factory;
pub mod utils;
mod dbms;

use ext_php_rs::prelude::*;
#[cfg(feature = "lazy-row")]
pub use lazy_row::{LazyRow, LazyRowJson};
use std::num::NonZeroU32;
use std::sync::LazyLock;
use tokio::runtime::Runtime;
use dbms::{mssql, mysql, postgres};

/// Global runtime for executing async `SQLx` queries from sync context.
static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

const DEFAULT_AST_CACHE_SHARD_COUNT: usize = 8;
const DEFAULT_AST_CACHE_SHARD_SIZE: usize = 256;
const DEFAULT_MAX_CONNECTIONS: NonZeroU32 = NonZeroU32::new(2).unwrap();
const DEFAULT_MIN_CONNECTIONS: u32 = 0;
const DEFAULT_ASSOC_ARRAYS: bool = false;
const DEFAULT_COLLAPSIBLE_IN: bool = true;
const DEFAULT_TEST_BEFORE_ACQUIRE: bool = false;

#[cfg(feature = "mysql")]
pub use dbms::mysql::{MySqlDriver, MySqlPreparedQuery};

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    let module = select_clause::build(module);
    let module = by_clause::build(module);
    let module = paginate_clause::build(module);
    let module = query_builder::build(module);
    let module = driver_factory::build(module);

    #[cfg(feature = "mysql")]
    let module = mysql::build(module);

    #[cfg(feature = "postgres")]
    let module = postgres::build(module);

    #[cfg(feature = "mssql")]
    let module = mssql::build(module);

    #[cfg(feature = "lazy-row")]
    let module = lazy_row::build(module);

    module
}
