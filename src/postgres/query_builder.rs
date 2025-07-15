use crate::php_sqlx_impl_query_builder;
use crate::postgres::inner::PgDriverInner;

php_sqlx_impl_query_builder!(PgQueryBuilder, "Sqlx\\PgQueryBuilder", PgDriverInner);
