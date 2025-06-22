#![allow(clippy::needless_pass_by_value)]
use crate::php_sqlx_impl_driver_inner;
use sqlx_oldapi::postgres::PgPoolOptions as PoolOptions;
const ESCAPING_DOUBLE_SINGLE_QUOTES: bool = false;
const COMMENT_HASH: bool = false;
const COLUMN_BACKTICKS: bool = false;
const DOLLAR_SIGN_PLACEHOLDERS: bool = true;

php_sqlx_impl_driver_inner!(PgDriverInner, Postgres);
