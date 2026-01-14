//! `MySQL` query result iterator.

crate::php_sqlx_impl_query_result!(
    MySqlQueryResult,
    "Sqlx\\MySqlQueryResult",
    MySql,
    MySqlDriverInner
);
