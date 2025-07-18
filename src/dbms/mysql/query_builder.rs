use crate::dbms::mysql::inner::MySqlDriverInner;
use crate::{MySqlDriver, php_sqlx_impl_query_builder};

php_sqlx_impl_query_builder!(
    MySqlQueryBuilder,
    "Sqlx\\MySqlQueryBuilder",
    MySqlDriver,
    MySqlDriverInner
);
