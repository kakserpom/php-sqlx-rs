#[cfg(test)]
mod tests;

use crate::php_sqlx_impl_driver;

php_sqlx_impl_driver!(
    PgDriver,
    "Sqlx\\PgDriver",
    PgDriverInner,
    PgPreparedQuery,
    PgQueryBuilder
);
