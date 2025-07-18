#![allow(clippy::needless_pass_by_value)]
use crate::php_sqlx_impl_driver_inner;
pub const SETTINGS: Settings = Settings {
    collapsible_in_enabled: true,
    escaping_double_single_quotes: true,
    comment_hash: false,
    column_backticks: false,       // PostgreSQL uses double quotes
    placeholder_dollar_sign: true, // $1, $2...
    placeholder_at_sign: false,
    max_placeholders: 65535,

    booleans_as_literals: true, // PostgreSQL prefers TRUE/FALSE
    strings_as_ntext: false,    // N'' not used
    cast_json: Some("::jsonb"), // or ::json if jsonb not desired
    escape_backslash: false,    // unnecessary
};
php_sqlx_impl_driver_inner!(PgDriverInner, Postgres);
