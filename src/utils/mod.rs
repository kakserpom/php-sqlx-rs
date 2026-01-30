//! Utility functions and types for php-sqlx.
//!
//! This module provides various helper functions used throughout the crate:
//!
//! - [`ident`]: SQL identifier validation
//! - [`hashmap_fold`]: Helpers for folding query results into PHP hash maps
//! - [`indent_sql`]: SQL formatting with indentation
//! - [`strip_prefix`]: Case-insensitive prefix stripping for SQL parsing
//! - [`types`]: Common type definitions

pub mod hashmap_fold;
pub mod ident;
pub mod indent_sql;
pub mod strip_prefix;
pub mod types;
