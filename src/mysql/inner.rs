#![allow(clippy::needless_pass_by_value)]
use crate::mysql::ast::Ast;
use crate::php_sqlx_impl_driver_inner;
use sqlx::mysql::MySqlPoolOptions as PoolOptions;
php_sqlx_impl_driver_inner!(MySqlDriverInner, MySql);
