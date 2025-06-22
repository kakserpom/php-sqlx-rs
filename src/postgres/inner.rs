#![allow(clippy::needless_pass_by_value)]
use sqlx::postgres::PgPoolOptions as PoolOptions;
use crate::postgres::ast::Ast;
use crate::php_sqlx_impl_driver_inner;
php_sqlx_impl_driver_inner!(PgDriverInner, Postgres);
