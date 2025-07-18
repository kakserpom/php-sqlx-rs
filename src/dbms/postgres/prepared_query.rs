use crate::php_sqlx_impl_prepared_query;
use crate::dbms::postgres::inner::PgDriverInner;
php_sqlx_impl_prepared_query!(PgPreparedQuery, "Sqlx\\PgPreparedQuery", PgDriverInner);