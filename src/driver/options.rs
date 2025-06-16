use std::collections::HashMap;
use ext_php_rs::ZvalConvert;
use crate::{DEFAULT_AST_CACHE_SHARD_COUNT, DEFAULT_AST_CACHE_SHARD_SIZE};
use crate::ast::Value;

pub struct DriverOptions {
    pub(crate) url: Option<String>,
    pub(crate) ast_cache_shard_count: usize,
    pub(crate) ast_cache_shard_size: usize,
    pub(crate) persistent_name: Option<String>,
    pub(crate) associative_arrays: bool,
}
impl Default for DriverOptions {
    fn default() -> Self {
        Self {
            url: None,
            ast_cache_shard_count: DEFAULT_AST_CACHE_SHARD_COUNT,
            ast_cache_shard_size: DEFAULT_AST_CACHE_SHARD_SIZE,
            persistent_name: None,
            associative_arrays: false,
        }
    }
}

#[derive(ZvalConvert)]
pub enum DriverConstructorOptions {
    Url(String),
    Options(HashMap<String, Value>),
}
