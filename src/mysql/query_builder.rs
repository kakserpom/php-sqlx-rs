use crate::mysql::inner::MySqlDriverInner;
use crate::php_sqlx_impl_query_builder;

php_sqlx_impl_query_builder!(
    MySqlQueryBuilder,
    "Sqlx\\MySqlQueryBuilder",
    MySqlDriverInner
);
