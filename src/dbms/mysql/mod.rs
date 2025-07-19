#[cfg(test)]
mod tests;

use crate::php_sqlx_impl_driver;

php_sqlx_impl_driver!(
    MySqlDriver,
    "Sqlx\\MySqlDriver",
    MySqlDriverInner,
    MySqlPreparedQuery,
    MySqlReadQueryBuilder,
    MySqlWriteQueryBuilder,
);
