use crate::mysql::inner::MySqlDriverInner;
use crate::php_sqlx_impl_prepared_query;

php_sqlx_impl_prepared_query!(
    MySqlPreparedQuery,
    "Sqlx\\MySqlPreparedQuery",
    MySqlDriverInner
);
