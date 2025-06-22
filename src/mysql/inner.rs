#![allow(clippy::needless_pass_by_value)]
use crate::php_sqlx_impl_driver_inner;
use sqlx::mysql::MySqlPoolOptions as PoolOptions;
const ESCAPING_DOUBLE_SINGLE_QUOTES: bool = true;
const COMMENT_HASH: bool = true;
const COLUMN_BACKTICKS: bool = true;
const DOLLAR_SIGN_PLACEHOLDERS: bool = false;
php_sqlx_impl_driver_inner!(MySqlDriverInner, MySql);
