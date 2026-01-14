//! MSSQL query result iterator.

crate::php_sqlx_impl_query_result!(
    MssqlQueryResult,
    "Sqlx\\MssqlQueryResult",
    Mssql,
    MssqlDriverInner
);
