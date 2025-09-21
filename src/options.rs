use crate::param_value::ParameterValue;
use crate::{
    DEFAULT_ASSOC_ARRAYS, DEFAULT_AST_CACHE_SHARD_COUNT, DEFAULT_AST_CACHE_SHARD_SIZE,
    DEFAULT_COLLAPSIBLE_IN, DEFAULT_MAX_CONNECTIONS, DEFAULT_MIN_CONNECTIONS,
    DEFAULT_TEST_BEFORE_ACQUIRE,
};
use anyhow::{anyhow, bail};
use ext_php_rs::{ZvalConvert, php_class, php_impl};
use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::time::Duration;

#[allow(clippy::struct_excessive_bools)]
pub struct DriverInnerOptions {
    pub(crate) url: Option<String>,
    pub(crate) ast_cache_shard_count: usize,
    pub(crate) ast_cache_shard_size: usize,
    pub(crate) persistent_name: Option<String>,
    pub(crate) associative_arrays: bool,
    pub(crate) max_connections: NonZeroU32,
    pub(crate) min_connections: u32,
    pub(crate) max_lifetime: Option<Duration>,
    pub(crate) acquire_timeout: Option<Duration>,
    pub(crate) idle_timeout: Option<Duration>,
    pub(crate) test_before_acquire: bool,
    pub(crate) collapsible_in_enabled: bool,
    pub(crate) readonly: bool,
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
    pub const OPT_ACQUIRE_TIMEOUT: &'static str = "_timeout";

    /// Whether to validate connections before acquiring them from the pool.
    pub const OPT_TEST_BEFORE_ACQUIRE: &'static str = "test_before_acquire";
}

/// Represents either a simple URL string or a full associative array of driver options.
#[derive(ZvalConvert)]
pub enum DriverOptionsArg {
    Url(String),
    Options(BTreeMap<String, ParameterValue>),
}
impl DriverOptionsArg {
    /// Converts the argument into a validated `DriverInnerOptions` instance.
    pub fn parse(self) -> anyhow::Result<DriverInnerOptions> {
        Ok(match self {
            Self::Url(url) => DriverInnerOptions {
                url: Some(url),
                ..Default::default()
            },
            Self::Options(kv) => DriverInnerOptions {
                url: Some(
                    kv.get(DriverOptions::OPT_URL)
                        .ok_or_else(|| anyhow!("missing OPT_URL"))
                        .and_then(|value| {
                            if let ParameterValue::String(str) = value {
                                Ok(str.clone())
                            } else {
                                Err(anyhow!("OPT_URL must be a string"))
                            }
                        })?,
                ),
                associative_arrays: kv.get(DriverOptions::OPT_ASSOC_ARRAYS).map_or(
                    Ok(DEFAULT_ASSOC_ARRAYS),
                    |value| {
                        if let ParameterValue::Bool(bool) = value {
                            Ok(*bool)
                        } else {
                            Err(anyhow!("OPT_ASSOC_ARRAYS must be a string"))
                        }
                    },
                )?,
                ast_cache_shard_count: kv.get(DriverOptions::OPT_AST_CACHE_SHARD_COUNT).map_or(
                    Ok(DEFAULT_AST_CACHE_SHARD_COUNT),
                    |value| {
                        if let ParameterValue::Int(n) = value {
                            Ok(usize::try_from(*n)?)
                        } else {
                            Err(anyhow!("OPT_AST_CACHE_SHARD_COUNT must be an integer"))
                        }
                    },
                )?,
                ast_cache_shard_size: kv.get(DriverOptions::OPT_AST_CACHE_SHARD_SIZE).map_or(
                    Ok(DEFAULT_AST_CACHE_SHARD_SIZE),
                    |value| {
                        if let ParameterValue::Int(n) = value {
                            Ok(usize::try_from(*n)?)
                        } else {
                            Err(anyhow!("OPT_AST_CACHE_SHARD_SIZE must be an integer"))
                        }
                    },
                )?,
                persistent_name: match kv.get(DriverOptions::OPT_PERSISTENT_NAME) {
                    None | Some(ParameterValue::Null) => None,
                    Some(value) => {
                        if let ParameterValue::String(str) = value {
                            Some(str.clone())
                        } else {
                            bail!("OPT_PERSISTENT_NAME must be an integer");
                        }
                    }
                },
                max_connections: kv.get(DriverOptions::OPT_MAX_CONNECTIONS).map_or(
                    Ok(DEFAULT_MAX_CONNECTIONS),
                    |value| {
                        if let ParameterValue::Int(n) = value {
                            Ok(NonZeroU32::try_from(u32::try_from(*n)?)?)
                        } else {
                            Err(anyhow!("OPT_MAX_CONNECTIONS must be a positive integer"))
                        }
                    },
                )?,
                min_connections: kv.get(DriverOptions::OPT_MIN_CONNECTIONS).map_or(
                    Ok(DEFAULT_MIN_CONNECTIONS),
                    |value| {
                        if let ParameterValue::Int(n) = value {
                            Ok(u32::try_from(*n)?)
                        } else {
                            Err(anyhow!(
                                "OPT_MIN_CONNECTIONS must be a non-negative integer"
                            ))
                        }
                    },
                )?,
                max_lifetime: match kv.get(DriverOptions::OPT_MAX_LIFETIME) {
                    None | Some(ParameterValue::Null) => None,
                    Some(ParameterValue::String(value)) => Some(parse_duration::parse(value)?),
                    Some(ParameterValue::Int(value)) => {
                        Some(Duration::from_secs(u64::try_from(*value)?))
                    }
                    _ => bail!("OPT_MAX_LIFETIME must be a string or a non-negative integer"),
                },
                idle_timeout: match kv.get(DriverOptions::OPT_IDLE_TIMEOUT) {
                    None | Some(ParameterValue::Null) => None,
                    Some(ParameterValue::String(value)) => Some(parse_duration::parse(value)?),
                    Some(ParameterValue::Int(value)) => {
                        Some(Duration::from_secs(u64::try_from(*value)?))
                    }
                    _ => bail!("OPT_IDLE_TIMEOUT must be a string or a non-negative integer"),
                },
                acquire_timeout: match kv.get(DriverOptions::OPT_ACQUIRE_TIMEOUT) {
                    None | Some(ParameterValue::Null) => None,
                    Some(ParameterValue::String(value)) => Some(parse_duration::parse(value)?),
                    Some(ParameterValue::Int(value)) => {
                        Some(Duration::from_secs(u64::try_from(*value)?))
                    }
                    _ => bail!("OPT_ACQUIRE_TIMEOUT must be a string or a non-negative integer"),
                },
                test_before_acquire: kv.get(DriverOptions::OPT_TEST_BEFORE_ACQUIRE).map_or(
                    Ok(DEFAULT_TEST_BEFORE_ACQUIRE),
                    |value| {
                        if let ParameterValue::Bool(bool) = value {
                            Ok(*bool)
                        } else {
                            Err(anyhow!("OPT_TEST_BEFORE_ACQUIRE must be a boolean"))
                        }
                    },
                )?,
                collapsible_in_enabled: kv.get(DriverOptions::OPT_COLLAPSIBLE_IN).map_or(
                    Ok(DEFAULT_COLLAPSIBLE_IN),
                    |value| {
                        if let ParameterValue::Bool(bool) = value {
                            Ok(*bool)
                        } else {
                            Err(anyhow!("OPT_COLLAPSIBLE_IN must be a boolean"))
                        }
                    },
                )?,
                readonly: kv
                    .get(DriverOptions::OPT_READONLY)
                    .map_or(Ok(false), |value| {
                        if let ParameterValue::Bool(bool) = value {
                            Ok(*bool)
                        } else {
                            Err(anyhow!("OPT_READONLY must be a boolean"))
                        }
                    })?,
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
