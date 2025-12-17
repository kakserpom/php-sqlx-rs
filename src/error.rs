//! Error types for php-sqlx.
//!
//! This module provides a structured error enum that converts to PHP exceptions
//! with appropriate error codes and messages.

use ext_php_rs::builders::ModuleBuilder;
use ext_php_rs::class::RegisteredClass;
use ext_php_rs::exception::PhpException;
use ext_php_rs::prelude::*;
use ext_php_rs::zend::ce;
use std::fmt;

/// Custom exception class for php-sqlx errors.
///
/// This exception class extends PHP's base Exception and is used for all
/// errors thrown by the php-sqlx extension. The error code can be used
/// to programmatically handle specific error types.
#[php_class]
#[php(name = "Sqlx\\SqlxException")]
#[php(extends(ce = ce::exception, stub = "\\Exception"))]
#[derive(Default)]
pub struct SqlxException;

#[php_impl]
impl SqlxException {
    /// General/unknown error
    pub const GENERAL: i32 = ErrorCode::General as i32;
    /// Database connection failed
    pub const CONNECTION: i32 = ErrorCode::Connection as i32;
    /// Query execution failed
    pub const QUERY: i32 = ErrorCode::Query as i32;
    /// Transaction-related error
    pub const TRANSACTION: i32 = ErrorCode::Transaction as i32;
    /// SQL parsing/AST error
    pub const PARSE: i32 = ErrorCode::Parse as i32;
    /// Missing or invalid parameter
    pub const PARAMETER: i32 = ErrorCode::Parameter as i32;
    /// Configuration/options error
    pub const CONFIGURATION: i32 = ErrorCode::Configuration as i32;
    /// Invalid identifier or input validation error
    pub const VALIDATION: i32 = ErrorCode::Validation as i32;
    /// Operation not permitted (e.g., write on readonly)
    pub const NOT_PERMITTED: i32 = ErrorCode::NotPermitted as i32;
    /// Timeout error
    pub const TIMEOUT: i32 = ErrorCode::Timeout as i32;
    /// Pool exhausted
    pub const POOL_EXHAUSTED: i32 = ErrorCode::PoolExhausted as i32;
}

/// Error codes for categorizing errors in PHP.
///
/// These codes can be used in PHP to programmatically handle specific error types.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// General/unknown error
    General = 0,
    /// Database connection failed
    Connection = 1,
    /// Query execution failed
    Query = 2,
    /// Transaction-related error
    Transaction = 3,
    /// SQL parsing/AST error
    Parse = 4,
    /// Missing or invalid parameter
    Parameter = 5,
    /// Configuration/options error
    Configuration = 6,
    /// Invalid identifier or input validation error
    Validation = 7,
    /// Operation not permitted (e.g., write on readonly)
    NotPermitted = 8,
    /// Timeout error
    Timeout = 9,
    /// Pool exhausted
    PoolExhausted = 10,
}

/// The main error type for php-sqlx operations.
///
/// This enum provides structured error handling with specific variants for
/// different failure modes. All variants convert to PHP exceptions via
/// `Into<PhpException>`.
#[derive(Debug)]
pub enum Error {
    /// Database connection failed.
    Connection {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Query execution failed.
    Query {
        message: String,
        query: Option<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// No active transaction to commit/rollback.
    NoActiveTransaction,

    /// Transaction commit failed.
    CommitFailed {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Transaction rollback failed.
    RollbackFailed {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Failed to begin transaction.
    BeginFailed {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// SQL parsing error.
    Parse { message: String, sql: Option<String> },

    /// Missing required placeholder in query.
    MissingPlaceholder { name: String },

    /// Invalid parameter type for placeholder.
    InvalidParameter { name: String, expected: String },

    /// Invalid configuration option.
    Configuration { option: String, message: String },

    /// Invalid SQL identifier.
    InvalidIdentifier { value: String },

    /// Write operation attempted on readonly connection.
    ReadonlyViolation,

    /// Connection pool exhausted.
    PoolExhausted { timeout_ms: u64 },

    /// Operation timed out.
    Timeout { operation: String, timeout_ms: u64 },

    /// URL not provided in options.
    UrlRequired,

    /// Column not found in result set.
    ColumnNotFound { column: String },

    /// Invalid savepoint name.
    InvalidSavepoint { name: String },

    /// Conversion error (e.g., to PHP Zval).
    Conversion { message: String },

    /// General error with message.
    Other(String),
}

impl Error {
    /// Returns the error code for this error.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        match self {
            Self::Connection { .. } => ErrorCode::Connection,
            Self::Query { .. } => ErrorCode::Query,
            Self::NoActiveTransaction
            | Self::CommitFailed { .. }
            | Self::RollbackFailed { .. }
            | Self::BeginFailed { .. } => ErrorCode::Transaction,
            Self::Parse { .. } => ErrorCode::Parse,
            Self::MissingPlaceholder { .. } | Self::InvalidParameter { .. } => ErrorCode::Parameter,
            Self::Configuration { .. } | Self::UrlRequired => ErrorCode::Configuration,
            Self::InvalidIdentifier { .. } | Self::InvalidSavepoint { .. } => ErrorCode::Validation,
            Self::ReadonlyViolation => ErrorCode::NotPermitted,
            Self::PoolExhausted { .. } => ErrorCode::PoolExhausted,
            Self::Timeout { .. } => ErrorCode::Timeout,
            Self::ColumnNotFound { .. } => ErrorCode::Query,
            Self::Conversion { .. } => ErrorCode::General,
            Self::Other(_) => ErrorCode::General,
        }
    }

    // Convenience constructors

    /// Creates a connection error.
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a connection error with source.
    pub fn connection_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Connection {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Creates a query error.
    pub fn query(message: impl Into<String>) -> Self {
        Self::Query {
            message: message.into(),
            query: None,
            source: None,
        }
    }

    /// Creates a query error with the SQL that failed.
    pub fn query_with_sql(message: impl Into<String>, sql: impl Into<String>) -> Self {
        Self::Query {
            message: message.into(),
            query: Some(sql.into()),
            source: None,
        }
    }

    /// Creates a query error with source and SQL.
    pub fn query_with_source(
        sql: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        let source_msg = source.to_string();
        Self::Query {
            message: source_msg,
            query: Some(sql.into()),
            source: Some(Box::new(source)),
        }
    }

    /// Creates a parse error.
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
            sql: None,
        }
    }

    /// Creates a parse error with the SQL that failed.
    pub fn parse_with_sql(message: impl Into<String>, sql: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
            sql: Some(sql.into()),
        }
    }

    /// Creates a configuration error.
    pub fn config(option: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Configuration {
            option: option.into(),
            message: message.into(),
        }
    }

    /// Creates a commit failed error with source.
    pub fn commit_failed(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::CommitFailed {
            message: source.to_string(),
            source: Some(Box::new(source)),
        }
    }

    /// Creates a rollback failed error with source.
    pub fn rollback_failed(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::RollbackFailed {
            message: source.to_string(),
            source: Some(Box::new(source)),
        }
    }

    /// Creates a begin failed error with source.
    pub fn begin_failed(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::BeginFailed {
            message: source.to_string(),
            source: Some(Box::new(source)),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection { message, .. } => write!(f, "Connection error: {message}"),
            Self::Query { message, query, .. } => {
                write!(f, "Query error: {message}")?;
                if let Some(sql) = query {
                    write!(f, "\nQuery: {sql}")?;
                }
                Ok(())
            }
            Self::NoActiveTransaction => write!(f, "No active transaction"),
            Self::CommitFailed { message, .. } => write!(f, "Failed to commit transaction: {message}"),
            Self::RollbackFailed { message, .. } => {
                write!(f, "Failed to rollback transaction: {message}")
            }
            Self::BeginFailed { message, .. } => write!(f, "Failed to begin transaction: {message}"),
            Self::Parse { message, sql } => {
                write!(f, "SQL parse error: {message}")?;
                if let Some(sql) = sql {
                    write!(f, "\nSQL: {sql}")?;
                }
                Ok(())
            }
            Self::MissingPlaceholder { name } => {
                write!(f, "Missing required placeholder: `{name}`")
            }
            Self::InvalidParameter { name, expected } => {
                write!(f, "Invalid parameter `{name}`: expected {expected}")
            }
            Self::Configuration { option, message } => {
                write!(f, "Configuration error for `{option}`: {message}")
            }
            Self::InvalidIdentifier { value } => write!(f, "Invalid identifier: `{value}`"),
            Self::ReadonlyViolation => write!(f, "Cannot write to a readonly connection"),
            Self::PoolExhausted { timeout_ms } => {
                write!(f, "Connection pool exhausted (timeout: {timeout_ms}ms)")
            }
            Self::Timeout {
                operation,
                timeout_ms,
            } => write!(f, "Operation `{operation}` timed out after {timeout_ms}ms"),
            Self::UrlRequired => write!(f, "Database URL is required"),
            Self::ColumnNotFound { column } => write!(f, "Column `{column}` not found"),
            Self::InvalidSavepoint { name } => write!(f, "Invalid savepoint name: `{name}`"),
            Self::Conversion { message } => write!(f, "Conversion error: {message}"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Connection { source, .. }
            | Self::Query { source, .. }
            | Self::CommitFailed { source, .. }
            | Self::RollbackFailed { source, .. }
            | Self::BeginFailed { source, .. } => {
                source.as_ref().map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
            }
            _ => None,
        }
    }
}

impl From<Error> for PhpException {
    fn from(err: Error) -> Self {
        let code = err.code() as i32;
        let message = err.to_string();

        PhpException::new(message, code, SqlxException::get_metadata().ce())
    }
}


impl From<std::num::TryFromIntError> for Error {
    fn from(err: std::num::TryFromIntError) -> Self {
        Self::Conversion {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Conversion {
            message: format!("JSON error: {err}"),
        }
    }
}

#[cfg(feature = "simd-json")]
impl From<simd_json::Error> for Error {
    fn from(err: simd_json::Error) -> Self {
        Self::Conversion {
            message: format!("JSON error: {err}"),
        }
    }
}

impl From<std::fmt::Error> for Error {
    fn from(err: std::fmt::Error) -> Self {
        Self::Conversion {
            message: format!("Format error: {err}"),
        }
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Self::Configuration {
            option: "url".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<sqlx_oldapi::Error> for Error {
    fn from(err: sqlx_oldapi::Error) -> Self {
        match &err {
            sqlx_oldapi::Error::RowNotFound => Self::Query {
                message: "Row not found".to_string(),
                query: None,
                source: Some(Box::new(err)),
            },
            sqlx_oldapi::Error::ColumnNotFound(name) => Self::ColumnNotFound {
                column: name.clone(),
            },
            sqlx_oldapi::Error::PoolTimedOut => Self::PoolExhausted { timeout_ms: 0 },
            _ => Self::Query {
                message: err.to_string(),
                query: None,
                source: Some(Box::new(err)),
            },
        }
    }
}

/// Result type alias using our Error.
pub type Result<T> = std::result::Result<T, Error>;

/// Registers the error module classes with the PHP module.
pub fn build(module: ModuleBuilder) -> ModuleBuilder {
    module.class::<SqlxException>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::MissingPlaceholder {
            name: "user_id".to_string(),
        };
        assert_eq!(err.to_string(), "Missing required placeholder: `user_id`");
    }

    #[test]
    fn test_error_code() {
        assert_eq!(Error::NoActiveTransaction.code(), ErrorCode::Transaction);
        assert_eq!(
            Error::MissingPlaceholder {
                name: "x".to_string()
            }
            .code(),
            ErrorCode::Parameter
        );
        assert_eq!(Error::ReadonlyViolation.code(), ErrorCode::NotPermitted);
    }

    #[test]
    fn test_query_error_with_sql() {
        let err = Error::query_with_sql("column not found", "SELECT foo FROM bar");
        assert!(err.to_string().contains("column not found"));
        assert!(err.to_string().contains("SELECT foo FROM bar"));
    }
}
