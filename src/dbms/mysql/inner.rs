#![allow(clippy::needless_pass_by_value)]
use crate::ast::{IdentifierQuoteStyle, UpsertStyle};
use crate::php_sqlx_impl_driver_inner;

/// SQL query to set the application name as a session variable.
/// Can be queried via `SELECT @sqlx_application_name`.
/// Note: `MySQL` doesn't have a built-in equivalent to `PostgreSQL`'s `application_name`,
/// but this value can be retrieved in custom monitoring queries.
pub const SET_APPLICATION_NAME_QUERY: &str = "SET @sqlx_application_name = $name";

/// SQL query to describe table columns using `information_schema`.
pub const DESCRIBE_TABLE_QUERY: &str = r"
SELECT
    column_name AS name,
    column_type AS type,
    is_nullable = 'YES' AS nullable,
    column_default AS `default`,
    ordinal_position AS ordinal
FROM information_schema.columns
WHERE table_name = $table
  AND table_schema = COALESCE($schema!n, DATABASE())
ORDER BY ordinal_position
";

pub const SETTINGS: Settings = Settings {
    collapsible_in_enabled: true,
    escaping_double_single_quotes: true,
    comment_hash: true,
    column_backticks: true,
    placeholder_dollar_sign: false,
    placeholder_at_sign: false,
    max_placeholders: 65535,
    strict_placeholders: false,

    booleans_as_literals: false, // MySQL prefers 1/0
    strings_as_ntext: false,     // MySQL doesn't support N'...' (but MariaDB does)
    cast_json: Some("AS JSON"),  // optional, supported in MySQL 5.7+
    escape_backslash: true,      // MySQL interprets \ as escape
    upsert_style: UpsertStyle::OnDuplicateKey,
    identifier_quote_style: IdentifierQuoteStyle::Backtick,
};

php_sqlx_impl_driver_inner!(MySqlDriverInner, MySql);

impl MySqlDriverInner {
    /// Bulk-ingest fast path is not available for `MySQL`.
    ///
    /// `MySQL`'s `LOAD DATA LOCAL INFILE` is not exposed by the underlying driver,
    /// so there is no `COPY`-equivalent. Use `insertMany()` instead.
    #[allow(clippy::unused_self)]
    pub fn copy_in(
        &self,
        _table: &str,
        _rows: &ext_php_rs::types::Zval,
    ) -> crate::error::Result<u64> {
        Err(crate::error::Error::Other(
            "copyIn (bulk COPY) is only supported on PostgreSQL; use insertMany() on MySQL"
                .to_string(),
        ))
    }
}
