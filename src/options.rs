//! Driver configuration options for php-sqlx.
//!
//! This module provides configuration types for database drivers, including
//! connection pooling settings, AST cache configuration, retry policy, and behavioral options.
//!
//! # PHP Usage
//!
//! Options can be passed either as a simple URL string or as an associative array:
//!
//! ```php
//! // Simple URL
//! $driver = DriverFactory::make('postgres://user:pass@localhost/db');
//!
//! // Full options array
//! $driver = DriverFactory::make([
//!     DriverOptions::OPT_URL => 'postgres://user:pass@localhost/db',
//!     DriverOptions::OPT_MAX_CONNECTIONS => 10,
//!     DriverOptions::OPT_ASSOC_ARRAYS => true,
//! ]);
//!
//! // With retry policy for transient failures
//! $driver = DriverFactory::make([
//!     DriverOptions::OPT_URL => 'postgres://user:pass@localhost/db',
//!     DriverOptions::OPT_RETRY_MAX_ATTEMPTS => 3,
//!     DriverOptions::OPT_RETRY_INITIAL_BACKOFF => '100ms',
//!     DriverOptions::OPT_RETRY_MAX_BACKOFF => '5s',
//!     DriverOptions::OPT_RETRY_MULTIPLIER => 2.0,
//! ]);
//! ```
//!
//! # Retry Policy
//!
//! The retry policy automatically retries transient failures (pool exhaustion,
//! connection drops, timeouts) with exponential backoff. Retry is disabled by
//! default (`max_attempts = 0`). Retries are skipped inside transactions to
//! prevent partial commits.

use crate::error::{Error as SqlxError, Result};
use crate::param_value::ParameterValue;
use crate::{
    DEFAULT_ASSOC_ARRAYS, DEFAULT_AST_CACHE_SHARD_COUNT, DEFAULT_AST_CACHE_SHARD_SIZE,
    DEFAULT_COLLAPSIBLE_IN, DEFAULT_MAX_CONNECTIONS, DEFAULT_MIN_CONNECTIONS,
    DEFAULT_RETRY_INITIAL_BACKOFF, DEFAULT_RETRY_MAX_ATTEMPTS, DEFAULT_RETRY_MAX_BACKOFF,
    DEFAULT_RETRY_MULTIPLIER, DEFAULT_TEST_BEFORE_ACQUIRE,
};
use ext_php_rs::{ZvalConvert, php_class, php_impl};
use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::time::Duration;

/// Internal configuration options for database drivers.
///
/// This struct holds all the parsed and validated configuration values
/// that control driver behavior. It is created by parsing [`DriverOptionsArg`].
#[allow(clippy::struct_excessive_bools)]
pub struct DriverInnerOptions {
    /// Database connection URL (e.g., `postgres://user:pass@host/db`).
    pub(crate) url: Option<String>,
    /// Number of shards in the AST LRU cache for concurrent access.
    pub(crate) ast_cache_shard_count: usize,
    /// Maximum entries per shard in the AST LRU cache.
    pub(crate) ast_cache_shard_size: usize,
    /// Optional name for persistent connection pooling across requests.
    pub(crate) persistent_name: Option<String>,
    /// Whether to return results as associative arrays (true) or objects (false).
    pub(crate) associative_arrays: bool,
    /// Maximum number of connections in the pool.
    pub(crate) max_connections: NonZeroU32,
    /// Minimum number of idle connections to maintain.
    pub(crate) min_connections: u32,
    /// Maximum lifetime of a connection before it's closed and replaced.
    pub(crate) max_lifetime: Option<Duration>,
    /// Timeout when acquiring a connection from the pool.
    pub(crate) acquire_timeout: Option<Duration>,
    /// How long a connection can remain idle before being closed.
    pub(crate) idle_timeout: Option<Duration>,
    /// Whether to validate connections before acquiring from pool.
    pub(crate) test_before_acquire: bool,
    /// Whether empty IN clauses collapse to FALSE (and NOT IN to TRUE).
    pub(crate) collapsible_in_enabled: bool,
    /// Whether the connection should be read-only (useful for replicas).
    pub(crate) readonly: bool,
    /// Maximum retry attempts for transient failures (0 = disabled).
    pub(crate) retry_max_attempts: u32,
    /// Initial backoff duration between retry attempts.
    pub(crate) retry_initial_backoff: Duration,
    /// Maximum backoff duration between retry attempts.
    pub(crate) retry_max_backoff: Duration,
    /// Backoff multiplier for exponential backoff between retries.
    pub(crate) retry_multiplier: f64,
}
impl Default for DriverInnerOptions {
    fn default() -> Self {
        Self {
            url: None,
            ast_cache_shard_count: DEFAULT_AST_CACHE_SHARD_COUNT,
            ast_cache_shard_size: DEFAULT_AST_CACHE_SHARD_SIZE,
            persistent_name: None,
            associative_arrays: DEFAULT_ASSOC_ARRAYS,
            max_connections: DEFAULT_MAX_CONNECTIONS,
            min_connections: DEFAULT_MIN_CONNECTIONS,
            max_lifetime: None,
            acquire_timeout: None,
            idle_timeout: None,
            test_before_acquire: DEFAULT_TEST_BEFORE_ACQUIRE,
            collapsible_in_enabled: DEFAULT_COLLAPSIBLE_IN,
            readonly: false,
            retry_max_attempts: DEFAULT_RETRY_MAX_ATTEMPTS,
            retry_initial_backoff: DEFAULT_RETRY_INITIAL_BACKOFF,
            retry_max_backoff: DEFAULT_RETRY_MAX_BACKOFF,
            retry_multiplier: DEFAULT_RETRY_MULTIPLIER,
        }
    }
}

#[php_class]
/// Represents the available options for `SQLx` drivers (`PgDriver`, `MySqlDriver`, `MssqlDriver`).
///
/// These constants are used as keys when constructing an options array passed to `DriverFactory::make(...)`.
#[php(name = "Sqlx\\DriverOptions")]
pub struct DriverOptions {}
#[php_impl]
impl DriverOptions {
    /// Required database URL, such as `postgres://user:pass@localhost/db`.
    pub const OPT_URL: &'static str = "url";

    /// Number of AST cache shards (advanced).
    pub const OPT_AST_CACHE_SHARD_COUNT: &'static str = "ast_cache_shard_count";

    /// Max entries per AST cache shard (advanced).
    pub const OPT_AST_CACHE_SHARD_SIZE: &'static str = "ast_cache_shard_size";

    /// Pool name to enable persistent connection reuse.
    pub const OPT_PERSISTENT_NAME: &'static str = "persistent_name";

    /// Return rows as associative arrays instead of objects (default: false).
    pub const OPT_ASSOC_ARRAYS: &'static str = "assoc_arrays";

    /// Maximum number of connections in the pool (default: 10).
    pub const OPT_MAX_CONNECTIONS: &'static str = "max_connections";

    /// Minimum number of connections in the pool (default: 0).
    pub const OPT_MIN_CONNECTIONS: &'static str = "min_connections";

    /// Enable automatic collapsing of `IN ()` clauses to `FALSE`/`TRUE`.
    pub const OPT_COLLAPSIBLE_IN: &'static str = "collapsible_in";

    /// Enable read-only mode (useful for replicas).
    pub const OPT_READONLY: &'static str = "readonly";

    /// Maximum lifetime of a pooled connection. Accepts string (`"30s"`, `"5 min"`) or integer (seconds).
    pub const OPT_MAX_LIFETIME: &'static str = "max_lifetime";

    /// Idle timeout for pooled connections. Accepts string or integer (seconds).
    pub const OPT_IDLE_TIMEOUT: &'static str = "idle_timeout";

    /// Timeout when acquiring a connection from the pool. Accepts string or integer (seconds).
    pub const OPT_ACQUIRE_TIMEOUT: &'static str = "acquire_timeout";

    /// Whether to validate connections before acquiring them from the pool.
    pub const OPT_TEST_BEFORE_ACQUIRE: &'static str = "test_before_acquire";

    /// Maximum retry attempts for transient failures (default: 0 = disabled).
    pub const OPT_RETRY_MAX_ATTEMPTS: &'static str = "retry_max_attempts";

    /// Initial backoff duration between retries. Accepts string (`"100ms"`, `"1s"`) or integer (seconds).
    pub const OPT_RETRY_INITIAL_BACKOFF: &'static str = "retry_initial_backoff";

    /// Maximum backoff duration between retries. Accepts string (`"5s"`, `"1 min"`) or integer (seconds).
    pub const OPT_RETRY_MAX_BACKOFF: &'static str = "retry_max_backoff";

    /// Backoff multiplier for exponential backoff (default: 2.0).
    pub const OPT_RETRY_MULTIPLIER: &'static str = "retry_multiplier";
}

/// Represents either a simple URL string or a full associative array of driver options.
///
/// This enum is automatically converted from PHP values by ext-php-rs, allowing
/// flexible driver construction syntax in PHP.
#[derive(ZvalConvert)]
pub enum DriverOptionsArg {
    /// A simple database connection URL string.
    Url(String),
    /// A full options array with configuration keys from [`DriverOptions`].
    Options(BTreeMap<String, ParameterValue>),
}
impl DriverOptionsArg {
    /// Converts the argument into a validated `DriverInnerOptions` instance.
    pub fn parse(self) -> Result<DriverInnerOptions> {
        Ok(match self {
            Self::Url(url) => DriverInnerOptions {
                url: Some(url),
                ..Default::default()
            },
            Self::Options(kv) => DriverInnerOptions {
                url: Some(
                    kv.get(DriverOptions::OPT_URL)
                        .ok_or(SqlxError::UrlRequired)
                        .and_then(|value| {
                            if let ParameterValue::String(str) = value {
                                Ok(str.clone())
                            } else {
                                Err(SqlxError::config("url", "must be a string"))
                            }
                        })?,
                ),
                associative_arrays: kv.get(DriverOptions::OPT_ASSOC_ARRAYS).map_or(
                    Ok(DEFAULT_ASSOC_ARRAYS),
                    |value| {
                        if let ParameterValue::Bool(bool) = value {
                            Ok(*bool)
                        } else {
                            Err(SqlxError::config("assoc_arrays", "must be a boolean"))
                        }
                    },
                )?,
                ast_cache_shard_count: kv.get(DriverOptions::OPT_AST_CACHE_SHARD_COUNT).map_or(
                    Ok(DEFAULT_AST_CACHE_SHARD_COUNT),
                    |value| {
                        if let ParameterValue::Int(n) = value {
                            Ok(usize::try_from(*n)?)
                        } else {
                            Err(SqlxError::config(
                                "ast_cache_shard_count",
                                "must be an integer",
                            ))
                        }
                    },
                )?,
                ast_cache_shard_size: kv.get(DriverOptions::OPT_AST_CACHE_SHARD_SIZE).map_or(
                    Ok(DEFAULT_AST_CACHE_SHARD_SIZE),
                    |value| {
                        if let ParameterValue::Int(n) = value {
                            Ok(usize::try_from(*n)?)
                        } else {
                            Err(SqlxError::config(
                                "ast_cache_shard_size",
                                "must be an integer",
                            ))
                        }
                    },
                )?,
                persistent_name: match kv.get(DriverOptions::OPT_PERSISTENT_NAME) {
                    None | Some(ParameterValue::Null) => None,
                    Some(value) => {
                        if let ParameterValue::String(str) = value {
                            Some(str.clone())
                        } else {
                            return Err(SqlxError::config("persistent_name", "must be a string"));
                        }
                    }
                },
                max_connections: kv.get(DriverOptions::OPT_MAX_CONNECTIONS).map_or(
                    Ok(DEFAULT_MAX_CONNECTIONS),
                    |value| {
                        if let ParameterValue::Int(n) = value {
                            Ok(NonZeroU32::try_from(u32::try_from(*n)?)?)
                        } else {
                            Err(SqlxError::config(
                                "max_connections",
                                "must be a positive integer",
                            ))
                        }
                    },
                )?,
                min_connections: kv.get(DriverOptions::OPT_MIN_CONNECTIONS).map_or(
                    Ok(DEFAULT_MIN_CONNECTIONS),
                    |value| {
                        if let ParameterValue::Int(n) = value {
                            Ok(u32::try_from(*n)?)
                        } else {
                            Err(SqlxError::config(
                                "min_connections",
                                "must be a non-negative integer",
                            ))
                        }
                    },
                )?,
                max_lifetime: match kv.get(DriverOptions::OPT_MAX_LIFETIME) {
                    None | Some(ParameterValue::Null) => None,
                    Some(ParameterValue::String(value)) => Some(
                        parse_duration::parse(value)
                            .map_err(|e| SqlxError::config("max_lifetime", e.to_string()))?,
                    ),
                    Some(ParameterValue::Int(value)) => {
                        Some(Duration::from_secs(u64::try_from(*value)?))
                    }
                    _ => {
                        return Err(SqlxError::config(
                            "max_lifetime",
                            "must be a string or a non-negative integer",
                        ));
                    }
                },
                idle_timeout: match kv.get(DriverOptions::OPT_IDLE_TIMEOUT) {
                    None | Some(ParameterValue::Null) => None,
                    Some(ParameterValue::String(value)) => Some(
                        parse_duration::parse(value)
                            .map_err(|e| SqlxError::config("idle_timeout", e.to_string()))?,
                    ),
                    Some(ParameterValue::Int(value)) => {
                        Some(Duration::from_secs(u64::try_from(*value)?))
                    }
                    _ => {
                        return Err(SqlxError::config(
                            "idle_timeout",
                            "must be a string or a non-negative integer",
                        ));
                    }
                },
                acquire_timeout: match kv.get(DriverOptions::OPT_ACQUIRE_TIMEOUT) {
                    None | Some(ParameterValue::Null) => None,
                    Some(ParameterValue::String(value)) => Some(
                        parse_duration::parse(value)
                            .map_err(|e| SqlxError::config("acquire_timeout", e.to_string()))?,
                    ),
                    Some(ParameterValue::Int(value)) => {
                        Some(Duration::from_secs(u64::try_from(*value)?))
                    }
                    _ => {
                        return Err(SqlxError::config(
                            "acquire_timeout",
                            "must be a string or a non-negative integer",
                        ));
                    }
                },
                test_before_acquire: kv.get(DriverOptions::OPT_TEST_BEFORE_ACQUIRE).map_or(
                    Ok(DEFAULT_TEST_BEFORE_ACQUIRE),
                    |value| {
                        if let ParameterValue::Bool(bool) = value {
                            Ok(*bool)
                        } else {
                            Err(SqlxError::config(
                                "test_before_acquire",
                                "must be a boolean",
                            ))
                        }
                    },
                )?,
                collapsible_in_enabled: kv.get(DriverOptions::OPT_COLLAPSIBLE_IN).map_or(
                    Ok(DEFAULT_COLLAPSIBLE_IN),
                    |value| {
                        if let ParameterValue::Bool(bool) = value {
                            Ok(*bool)
                        } else {
                            Err(SqlxError::config("collapsible_in", "must be a boolean"))
                        }
                    },
                )?,
                readonly: kv
                    .get(DriverOptions::OPT_READONLY)
                    .map_or(Ok(false), |value| {
                        if let ParameterValue::Bool(bool) = value {
                            Ok(*bool)
                        } else {
                            Err(SqlxError::config("readonly", "must be a boolean"))
                        }
                    })?,
                retry_max_attempts: kv.get(DriverOptions::OPT_RETRY_MAX_ATTEMPTS).map_or(
                    Ok(DEFAULT_RETRY_MAX_ATTEMPTS),
                    |value| {
                        if let ParameterValue::Int(n) = value {
                            Ok(u32::try_from(*n)?)
                        } else {
                            Err(SqlxError::config(
                                "retry_max_attempts",
                                "must be a non-negative integer",
                            ))
                        }
                    },
                )?,
                retry_initial_backoff: match kv.get(DriverOptions::OPT_RETRY_INITIAL_BACKOFF) {
                    None | Some(ParameterValue::Null) => DEFAULT_RETRY_INITIAL_BACKOFF,
                    Some(ParameterValue::String(value)) => parse_duration::parse(value)
                        .map_err(|e| SqlxError::config("retry_initial_backoff", e.to_string()))?,
                    Some(ParameterValue::Int(value)) => {
                        Duration::from_secs(u64::try_from(*value)?)
                    }
                    _ => {
                        return Err(SqlxError::config(
                            "retry_initial_backoff",
                            "must be a string or a non-negative integer",
                        ));
                    }
                },
                retry_max_backoff: match kv.get(DriverOptions::OPT_RETRY_MAX_BACKOFF) {
                    None | Some(ParameterValue::Null) => DEFAULT_RETRY_MAX_BACKOFF,
                    Some(ParameterValue::String(value)) => parse_duration::parse(value)
                        .map_err(|e| SqlxError::config("retry_max_backoff", e.to_string()))?,
                    Some(ParameterValue::Int(value)) => {
                        Duration::from_secs(u64::try_from(*value)?)
                    }
                    _ => {
                        return Err(SqlxError::config(
                            "retry_max_backoff",
                            "must be a string or a non-negative integer",
                        ));
                    }
                },
                retry_multiplier: kv.get(DriverOptions::OPT_RETRY_MULTIPLIER).map_or(
                    Ok(DEFAULT_RETRY_MULTIPLIER),
                    |value| match value {
                        ParameterValue::Float(f) => Ok(*f),
                        #[allow(clippy::cast_precision_loss)]
                        ParameterValue::Int(n) => Ok(*n as f64),
                        _ => Err(SqlxError::config(
                            "retry_multiplier",
                            "must be a number",
                        )),
                    },
                )?,
            },
        })
    }
}

#[test]
fn test_driver_options() {
    use crate::options::{DriverOptions, DriverOptionsArg};
    use std::collections::BTreeMap;
    use std::time::Duration;
    let driver_options = DriverOptionsArg::Options(BTreeMap::from_iter([
        (
            DriverOptions::OPT_URL.into(),
            "postgres://user:pass@host/database".into(),
        ),
        (DriverOptions::OPT_MAX_LIFETIME.into(), "1 hour".into()),
        (DriverOptions::OPT_IDLE_TIMEOUT.into(), "2 min".into()),
    ]))
    .parse()
    .unwrap();

    assert_eq!(driver_options.max_lifetime, Some(Duration::from_secs(3600)));
    assert_eq!(driver_options.idle_timeout, Some(Duration::from_secs(120)));
}
