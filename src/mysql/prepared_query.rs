use crate::mysql::inner::MySqlDriverInner;
use crate::php_sqlx_impl_prepared_query;
use ext_php_rs::prelude::*;
use std::sync::Arc;

/// A reusable prepared SQL query with parameter support. Created using `PgDriver::prepare()`, shares context with original driver.
#[php_class]
#[php(name = "Sqlx\\MySqlPreparedQuery")]
#[php(rename = "none")]
pub struct MySqlPreparedQuery {
    pub(crate) query: String,
    pub(crate) driver_inner: Arc<MySqlDriverInner>,
}

php_sqlx_impl_prepared_query!(MySqlPreparedQuery);
