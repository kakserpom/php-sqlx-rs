#![allow(clippy::needless_pass_by_value)]
use crate::ast::{IdentifierQuoteStyle, UpsertStyle};
use crate::php_sqlx_impl_driver_inner;

/// SQL query to set the application name for connection identification.
/// Visible in `pg_stat_activity.application_name`.
/// Uses `set_config()` because `SET` doesn't support parameterized queries.
pub const SET_APPLICATION_NAME_QUERY: &str = "SELECT set_config('application_name', $name, false)";

/// SQL query to describe table columns using `information_schema`.
pub const DESCRIBE_TABLE_QUERY: &str = r"
SELECT
    column_name AS name,
    data_type || COALESCE('(' || character_maximum_length::text || ')', '') AS type,
    is_nullable = 'YES' AS nullable,
    column_default AS default,
    ordinal_position AS ordinal
FROM information_schema.columns
WHERE table_name = $table
  AND table_schema = COALESCE($schema!n, current_schema())
ORDER BY ordinal_position
";

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
    upsert_style: UpsertStyle::OnConflict,
    identifier_quote_style: IdentifierQuoteStyle::DoubleQuote,
};
php_sqlx_impl_driver_inner!(PgDriverInner, Postgres);
