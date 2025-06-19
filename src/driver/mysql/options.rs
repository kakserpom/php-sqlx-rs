use crate::driver::DriverOptions;
use crate::driver::mysql::MySqlParameterValue;
use crate::{DEFAULT_ASSOC_ARRAYS, DEFAULT_AST_CACHE_SHARD_COUNT, DEFAULT_AST_CACHE_SHARD_SIZE, DEFAULT_COLLAPSIBLE_IN, DEFAULT_MAX_CONNECTIONS};
use anyhow::anyhow;
use ext_php_rs::ZvalConvert;
use std::collections::HashMap;
use std::num::NonZeroU32;

pub struct MySqlDriverInnerOptions {
    pub(crate) url: Option<String>,
    pub(crate) ast_cache_shard_count: usize,
    pub(crate) ast_cache_shard_size: usize,
    pub(crate) persistent_name: Option<String>,
    pub(crate) associative_arrays: bool,
    pub(crate) max_connections: NonZeroU32,
    pub(crate) collapsible_in: bool,
}
impl Default for MySqlDriverInnerOptions {
    fn default() -> Self {
        Self {
            url: None,
            ast_cache_shard_count: DEFAULT_AST_CACHE_SHARD_COUNT,
            ast_cache_shard_size: DEFAULT_AST_CACHE_SHARD_SIZE,
            persistent_name: None,
            associative_arrays: DEFAULT_ASSOC_ARRAYS,
            collapsible_in: DEFAULT_COLLAPSIBLE_IN,
            max_connections: DEFAULT_MAX_CONNECTIONS,
        }
    }
}

#[derive(ZvalConvert)]
pub enum MySqlDriverOptions {
    Url(String),
    Options(HashMap<String, MySqlParameterValue>),
}
impl MySqlDriverOptions {
    pub fn parse(self) -> anyhow::Result<MySqlDriverInnerOptions> {
        Ok(match self {
            Self::Url(url) => MySqlDriverInnerOptions {
                url: Some(url),
                ..Default::default()
            },
            Self::Options(kv) => MySqlDriverInnerOptions {
                url: Some(
                    kv.get(DriverOptions::OPT_URL)
                        .ok_or_else(|| anyhow!("missing {}", DriverOptions::OPT_URL))
                        .and_then(|value| {
                            if let MySqlParameterValue::Str(str) = value {
                                Ok(str.clone())
                            } else {
                                Err(anyhow!("{} must be a string", DriverOptions::OPT_URL))
                            }
                        })?,
                ),
                associative_arrays: kv.get(DriverOptions::OPT_ASSOC_ARRAYS).map_or(
                    Ok(false),
                    |value| {
                        if let MySqlParameterValue::Bool(bool) = value {
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
                        if let MySqlParameterValue::Int(n) = value {
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
                        if let MySqlParameterValue::Int(n) = value {
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
                        if let MySqlParameterValue::Str(str) = value {
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
                        if let MySqlParameterValue::Int(n) = value {
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
                    Ok(false),
                    |value| {
                        if let MySqlParameterValue::Bool(bool) = value {
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
