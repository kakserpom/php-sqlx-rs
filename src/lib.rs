//! # php-sqlx
//!
//! A high-performance PHP extension for database access, built with Rust and SQLx.
//!
//! ## Features
//!
//! - **AST-based SQL augmentation**: Conditional blocks, safe IN clauses, pagination
//! - **Multiple database support**: PostgreSQL, MySQL, MSSQL
//! - **Connection pooling**: Efficient connection management with configurable limits
//! - **Prepared queries**: Cached AST parsing for repeated queries
//! - **Query builder**: Fluent API for constructing SQL queries safely
//! - **Transaction support**: Both callback-based and imperative styles
//!
//! ## Architecture
//!
//! The crate is organized into several modules:
//!
//! - [`ast`]: SQL parsing and AST representation
//! - [`param_value`]: Parameter value types and conversion
//! - [`query_builder`]: Fluent query builder API
//! - [`options`]: Driver configuration options
//! - [`driver_factory`]: Factory for creating database drivers
//! - [`error`]: Typed error handling with PHP exception conversion
//! - Clause modules: [`select_clause`], [`by_clause`], [`paginate_clause`]

#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::must_use_candidate)]
#![cfg_attr(windows, feature(abi_vectorcall))]

// Use mimalloc allocator in release builds for better performance
#[cfg(not(debug_assertions))]
#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod ast;
pub mod by_clause;
pub mod conversion;
mod driver;
pub mod error;
mod inner_driver;
#[cfg(feature = "lazy-row")]
mod lazy_row;
pub mod options;
pub mod paginate_clause;
pub mod param_value;
mod prepared_query;
pub mod query_builder;
pub mod select_clause;

mod dbms;
pub mod driver_factory;
#[cfg(test)]
mod tests;
mod types;
pub mod utils;

use dbms::{mssql, mysql, postgres};
use ext_php_rs::prelude::*;
#[cfg(feature = "lazy-row")]
pub use lazy_row::{LazyRow, LazyRowJson};
use std::num::NonZeroU32;
use std::sync::LazyLock;
use tokio::runtime::Runtime;

/// Global Tokio runtime for executing async SQLx queries from synchronous PHP context.
///
/// This runtime is lazily initialized on first use and shared across all database
/// operations. It bridges the async SQLx API with PHP's synchronous execution model.
static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    Runtime::new().expect("Failed to create Tokio runtime")
});

/// Default number of shards in the AST LRU cache for concurrent access.
const DEFAULT_AST_CACHE_SHARD_COUNT: usize = 8;

/// Default maximum entries per shard in the AST LRU cache.
const DEFAULT_AST_CACHE_SHARD_SIZE: usize = 256;

/// Default maximum number of connections in the connection pool.
const DEFAULT_MAX_CONNECTIONS: NonZeroU32 = NonZeroU32::new(2).unwrap();

/// Default minimum number of idle connections to maintain in the pool.
const DEFAULT_MIN_CONNECTIONS: u32 = 0;

/// Default setting for returning results as associative arrays vs objects.
/// When false, results are returned as `stdClass` objects.
const DEFAULT_ASSOC_ARRAYS: bool = false;

/// Default setting for collapsible IN clause optimization.
/// When true, empty IN clauses become `FALSE` and empty NOT IN become `TRUE`.
const DEFAULT_COLLAPSIBLE_IN: bool = true;

/// Default setting for testing connections before acquiring from pool.
/// When true, validates connection health before use (adds latency).
const DEFAULT_TEST_BEFORE_ACQUIRE: bool = false;

#[cfg(feature = "mysql")]
pub use dbms::mysql::{MySqlDriver, MySqlPreparedQuery};

/// Builds the PHP module with all registered classes and functions.
///
/// This function is called by the PHP runtime when the extension is loaded.
/// It registers all database drivers, query builders, clause helpers, and
/// utility classes with the PHP module.
#[php_module]
pub fn module(mut module: ModuleBuilder) -> ModuleBuilder {
    module = module.name("sqlx").version(env!("CARGO_PKG_VERSION"));
    module = error::build(module);
    module = select_clause::build(module);
    module = by_clause::build(module);
    module = paginate_clause::build(module);
    module = query_builder::build(module);
    module = driver_factory::build(module);
    module = types::build(module);

    #[cfg(feature = "mysql")]
    {
        module = mysql::build(module);
    }

    #[cfg(feature = "postgres")]
    {
        module = postgres::build(module);
    }

    #[cfg(feature = "mssql")]
    {
        module = mssql::build(module);
    }

    #[cfg(feature = "lazy-row")]
    {
        module = lazy_row::build(module);
    }

    module
}
