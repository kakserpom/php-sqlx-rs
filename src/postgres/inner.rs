#![allow(clippy::needless_pass_by_value)]
use crate::php_sqlx_impl_driver_inner;
use sqlx_oldapi::postgres::PgPoolOptions as PoolOptions;
pub const ESCAPING_DOUBLE_SINGLE_QUOTES: bool = false;
pub const COMMENT_HASH: bool = false;
pub const COLUMN_BACKTICKS: bool = false;
pub const PLACEHOLDER_DOLLAR_SIGN: bool = true;
pub const PLACEHOLDER_AT_SIGN: bool = false;
php_sqlx_impl_driver_inner!(PgDriverInner, Postgres);
