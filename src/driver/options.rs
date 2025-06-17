use ext_php_rs::{php_class, php_impl};

#[php_class(name = "Sqlx\\DriverOptions")]
pub struct DriverOptions {}
#[php_impl]
impl DriverOptions {
    pub const OPT_URL: &'static str = "url";
    pub const OPT_AST_CACHE_SHARD_COUNT: &'static str = "ast_cache_shard_count";

    pub const OPT_AST_CACHE_SHARD_SIZE: &'static str = "ast_cache_shard_size";

    pub const OPT_PERSISTENT_NAME: &'static str = "persistent_name";
    pub const OPT_ASSOC_ARRAYS: &'static str = "assoc_arrays";

    pub const OPT_MAX_CONNECTIONS: &'static str = "max_connections";
}
