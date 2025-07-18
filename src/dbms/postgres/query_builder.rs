use crate::php_sqlx_impl_query_builder;
use crate::dbms::postgres::inner::PgDriverInner;
use crate::dbms::postgres::PgDriver;

php_sqlx_impl_query_builder!(PgQueryBuilder, "Sqlx\\PgQueryBuilder", PgDriver, PgDriverInner);
