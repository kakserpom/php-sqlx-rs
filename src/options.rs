use crate::paramvalue::ParameterValue;
use crate::{
    DEFAULT_ASSOC_ARRAYS, DEFAULT_AST_CACHE_SHARD_COUNT, DEFAULT_AST_CACHE_SHARD_SIZE,
    DEFAULT_COLLAPSIBLE_IN, DEFAULT_MAX_CONNECTIONS,
};
use anyhow::anyhow;
use ext_php_rs::{ZvalConvert, php_class, php_impl};
use std::collections::HashMap;
use std::num::NonZeroU32;

pub struct DriverInnerOptions {
    pub(crate) url: Option<String>,
    pub(crate) ast_cache_shard_count: usize,
    pub(crate) ast_cache_shard_size: usize,
    pub(crate) persistent_name: Option<String>,
    pub(crate) associative_arrays: bool,
    pub(crate) max_connections: NonZeroU32,
    pub(crate) collapsible_in: bool,
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
            collapsible_in: DEFAULT_COLLAPSIBLE_IN,
        }
    }
}

#[php_class]
#[php(name = "Sqlx\\DriverOptions")]
#[php(rename = "none")]
pub struct DriverOptions {}
#[php_impl]
impl DriverOptions {
    pub const OPT_URL: &'static str = "url";
    pub const OPT_AST_CACHE_SHARD_COUNT: &'static str = "ast_cache_shard_count";
    pub const OPT_AST_CACHE_SHARD_SIZE: &'static str = "ast_cache_shard_size";
    pub const OPT_PERSISTENT_NAME: &'static str = "persistent_name";
    pub const OPT_ASSOC_ARRAYS: &'static str = "assoc_arrays";
    pub const OPT_MAX_CONNECTIONS: &'static str = "max_connections";
    pub const OPT_COLLAPSIBLE_IN: &'static str = "collapsible_in";
}

#[derive(ZvalConvert)]
pub enum DriverOptionsArg {
    Url(String),
    Options(HashMap<String, ParameterValue>),
}
impl DriverOptionsArg {
    pub fn parse(self) -> anyhow::Result<DriverInnerOptions> {
        Ok(match self {
            Self::Url(url) => DriverInnerOptions {
                url: Some(url),
                ..Default::default()
            },
            Self::Options(kv) => DriverInnerOptions {
                url: Some(
                    kv.get(DriverOptions::OPT_URL)
                        .ok_or_else(|| anyhow!("missing {}", DriverOptions::OPT_URL))
                        .and_then(|value| {
                            if let ParameterValue::Str(str) = value {
                                Ok(str.clone())
                            } else {
                                Err(anyhow!("{} must be a string", DriverOptions::OPT_URL))
                            }
                        })?,
                ),
                associative_arrays: kv.get(DriverOptions::OPT_ASSOC_ARRAYS).map_or(
                    Ok(DEFAULT_ASSOC_ARRAYS),
                    |value| {
                        if let ParameterValue::Bool(bool) = value {
                            Ok(*bool)
                        } else {
                            Err(anyhow!(
                                "{} must be a string",
                                DriverOptions::OPT_ASSOC_ARRAYS
                            ))
                        }
                    },
                )?,
                ast_cache_shard_count: kv.get(DriverOptions::OPT_AST_CACHE_SHARD_COUNT).map_or(
                    Ok(DEFAULT_AST_CACHE_SHARD_COUNT),
                    |value| {
                        if let ParameterValue::Int(n) = value {
                            Ok(usize::try_from(*n)?)
                        } else {
                            Err(anyhow!(
                                "{} must be an integer",
                                DriverOptions::OPT_AST_CACHE_SHARD_COUNT
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
                            Err(anyhow!(
                                "{} must be an integer",
                                DriverOptions::OPT_AST_CACHE_SHARD_SIZE
                            ))
                        }
                    },
                )?,
                persistent_name: match kv.get(DriverOptions::OPT_PERSISTENT_NAME) {
                    None => None,
                    Some(value) => {
                        if let ParameterValue::Str(str) = value {
                            Some(str.clone())
                        } else {
                            return Err(anyhow!(
                                "{} must be an integer",
                                DriverOptions::OPT_PERSISTENT_NAME
                            ));
                        }
                    }
                },
                max_connections: kv.get(DriverOptions::OPT_MAX_CONNECTIONS).map_or(
                    Ok(DEFAULT_MAX_CONNECTIONS),
                    |value| {
                        if let ParameterValue::Int(n) = value {
                            Ok(NonZeroU32::try_from(u32::try_from(*n)?)?)
                        } else {
                            Err(anyhow!(
                                "{} must be a positive integer",
                                DriverOptions::OPT_MAX_CONNECTIONS
                            ))
                        }
                    },
                )?,
                collapsible_in: kv.get(DriverOptions::OPT_COLLAPSIBLE_IN).map_or(
                    Ok(DEFAULT_COLLAPSIBLE_IN),
                    |value| {
                        if let ParameterValue::Bool(bool) = value {
                            Ok(*bool)
                        } else {
                            Err(anyhow!(
                                "{} must be a boolean",
                                DriverOptions::OPT_COLLAPSIBLE_IN
                            ))
                        }
                    },
                )?,
            },
        })
    }
}
