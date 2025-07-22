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
    Null,
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<ParameterValue>),
    Object(BTreeMap<String, ParameterValue>),
    Json(Box<ParameterValue>),
    ByClauseRendered(ByClauseRendered),
    SelectClauseRendered(SelectClauseRendered),
    PaginateClauseRendered(PaginateClauseRendered),
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
