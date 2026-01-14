use crate::dbms::postgres::PgDriver;
use crate::dbms::postgres::inner::PgDriverInner;
use crate::php_sqlx_impl_query_builder;

php_sqlx_impl_query_builder!(
    PgWriteQueryBuilder,
    "Sqlx\\PgWriteQueryBuilder",
    "Sqlx\\WriteQueryBuilderInterface",
    PgDriver,
    PgDriverInner,
    PgQueryResult
);
