#![allow(clippy::needless_pass_by_value)]
use crate::php_sqlx_impl_driver_inner;
pub const ESCAPING_DOUBLE_SINGLE_QUOTES: bool = true;
pub const COMMENT_HASH: bool = true;
pub const COLUMN_BACKTICKS: bool = true;
pub const PLACEHOLDER_DOLLAR_SIGN: bool = false;
pub const PLACEHOLDER_AT_SIGN: bool = false;
php_sqlx_impl_driver_inner!(MySqlDriverInner, MySql);
