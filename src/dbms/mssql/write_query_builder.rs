use crate::dbms::mssql::MssqlDriver;
use crate::dbms::mssql::inner::MssqlDriverInner;
use crate::php_sqlx_impl_query_builder;

php_sqlx_impl_query_builder!(
    MssqlWriteQueryBuilder,
    "Sqlx\\MssqlWriteQueryBuilder",
    "Sqlx\\WriteQueryBuilderInterface",
    MssqlDriver,
    MssqlDriverInner
);
