//! PostgreSQL query result iterator.

crate::php_sqlx_impl_query_result!(
    PgQueryResult,
    "Sqlx\\PgQueryResult",
    Postgres,
    PgDriverInner
);
