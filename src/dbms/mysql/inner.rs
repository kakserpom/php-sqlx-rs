#![allow(clippy::needless_pass_by_value)]

use crate::php_sqlx_impl_driver_inner;

pub const SETTINGS: Settings = Settings {
    collapsible_in_enabled: true,
    escaping_double_single_quotes: true,
    comment_hash: true,
    column_backticks: true,
    placeholder_dollar_sign: false,
    placeholder_at_sign: false,
    max_placeholders: 65535,

    booleans_as_literals: false, // MySQL prefers 1/0
    strings_as_ntext: false,     // MySQL doesn't support N'...' (but MariaDB does)
    cast_json: Some("AS JSON"),  // optional, supported in MySQL 5.7+
    escape_backslash: true,      // MySQL interprets \ as escape
};

php_sqlx_impl_driver_inner!(MySqlDriverInner, MySql);
