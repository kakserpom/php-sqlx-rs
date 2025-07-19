#[cfg(test)]
mod tests;

use crate::php_sqlx_impl_driver;

php_sqlx_impl_driver!(
    MssqlDriver,
    "Sqlx\\MssqlDriver",
    MssqlDriverInner,
    MssqlPreparedQuery,
    MssqlReadQueryBuilder,
    MssqlWriteQueryBuilder
);
