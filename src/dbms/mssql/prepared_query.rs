use crate::dbms::mssql::inner::MssqlDriverInner;
use crate::php_sqlx_impl_prepared_query;

php_sqlx_impl_prepared_query!(
    MssqlPreparedQuery,
    "Sqlx\\Driver\\MssqlPreparedQuery",
    MssqlDriverInner
);
