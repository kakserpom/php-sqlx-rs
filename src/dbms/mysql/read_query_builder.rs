use crate::dbms::mysql::inner::MySqlDriverInner;
use crate::{MySqlDriver, php_sqlx_impl_query_builder};

php_sqlx_impl_query_builder!(
    MySqlReadQueryBuilder,
    "Sqlx\\MySqlReadQueryBuilder",
    "Sqlx\\ReadQueryBuilderInterface",
    MySqlDriver,
    MySqlDriverInner
);
