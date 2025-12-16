//! Parameter value types and conversion for php-sqlx.
//!
//! This module provides the [`ParameterValue`] enum that represents all possible
//! parameter types that can be bound to SQL queries. It handles conversion from
//! PHP values (via ext-php-rs) and rendering to SQL literals.
//!
//! # Supported Types
//!
//! - **Primitives**: null, string, integer, float, boolean
//! - **Arrays**: Homogeneous lists for IN clauses and batch operations
//! - **Objects**: Key-value maps for JSON columns
//! - **JSON**: Explicit JSON wrapper for proper serialization
//! - **Clauses**: Pre-rendered SELECT, ORDER BY, and pagination fragments
//!
//! # PHP to Rust Conversion
//!
//! PHP values are automatically converted via the `FromZval` trait implementation.
//! This allows seamless parameter binding from PHP code.

mod conversion;
mod json;
mod quote;
pub mod utils;
pub mod write;

use crate::by_clause::ByClauseRendered;
use crate::paginate_clause::PaginateClauseRendered;
use crate::select_clause::SelectClauseRendered;
use std::collections::BTreeMap;

/// A type alias representing the name of a placeholder in SQL templates.
pub type Placeholder = String;

/// A type alias for a parameter map used during query rendering and execution.
/// Keys are placeholders (e.g., `id`), and values are user-supplied input.
pub type ParamsMap = BTreeMap<Placeholder, ParameterValue>;

/// Represents a parameter value for use in SQL queries, supporting both primitive and complex structures.
///
/// Includes built-in types (string, int, float, bool), composite values (arrays, objects),
/// and pre-rendered clauses like `ORDER BY`, `SELECT`, and pagination fragments.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    /// SQL NULL value.
    Null,
    /// Text string value, escaped and quoted when rendered.
    String(String),
    /// 64-bit signed integer.
    Int(i64),
    /// 64-bit floating point number.
    Float(f64),
    /// Boolean value, rendered as TRUE/FALSE or 1/0 depending on database.
    Bool(bool),
    /// Array of values, typically expanded for IN clauses.
    Array(Vec<ParameterValue>),
    /// Key-value object, typically serialized as JSON.
    Object(BTreeMap<String, ParameterValue>),
    /// Explicit JSON wrapper that forces JSON serialization.
    Json(Box<ParameterValue>),
    /// Pre-rendered ORDER BY clause from `ByClause`.
    ByClauseRendered(ByClauseRendered),
    /// Pre-rendered SELECT clause from `SelectClause`.
    SelectClauseRendered(SelectClauseRendered),
    /// Pre-rendered pagination (LIMIT/OFFSET) from `PaginateClause`.
    PaginateClauseRendered(PaginateClauseRendered),
    /// Embedded query builder with its SQL and parameters.
    Builder((String, BTreeMap<String, ParameterValue>)),
}

impl ParameterValue {
    /// Checks whether the value is considered "empty".
    ///
    /// - For `ByClauseRendered`, `SelectClauseRendered`, and `Array`, returns true if empty.
    /// - For `Null`, always returns true.
    /// - Other variants return false.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::ByClauseRendered(x) => x.is_empty(),
            Self::SelectClauseRendered(x) => x.is_empty(),
            Self::Array(array) => array.is_empty(),
            Self::String(_)
            | Self::Int(_)
            | Self::Float(_)
            | Self::Bool(_)
            | Self::Object(_)
            | Self::PaginateClauseRendered(_)
            | Self::Builder(_) => false,
            Self::Null => true,
            Self::Json(inner) => inner.is_empty(),
        }
    }
}
