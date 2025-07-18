use crate::php_sqlx_impl_query_builder;
use crate::postgres::inner::PgDriverInner;
use crate::postgres::PgDriver;

php_sqlx_impl_query_builder!(PgQueryBuilder, "Sqlx\\PgQueryBuilder", PgDriver, PgDriverInner);
