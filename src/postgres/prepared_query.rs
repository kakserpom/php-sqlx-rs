use crate::php_sqlx_impl_prepared_query;
use crate::postgres::inner::PgDriverInner;
use ext_php_rs::prelude::*;
use std::sync::Arc;

/// A reusable prepared SQL query with parameter support. Created using `PgDriver::prepare()`, shares context with original driver.
#[php_class]
#[php(name = "Sqlx\\PgPreparedQuery")]
#[php(rename = "none")]
pub struct PgPreparedQuery {
    pub(crate) query: String,
    pub(crate) driver_inner: Arc<PgDriverInner>,
}

php_sqlx_impl_prepared_query!(PgPreparedQuery);
