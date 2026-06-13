#![allow(clippy::needless_pass_by_value)]
use crate::ast::{IdentifierQuoteStyle, UpsertStyle};
use crate::php_sqlx_impl_driver_inner;

/// SQL query to set the application name in session context.
/// Readable via `SELECT SESSION_CONTEXT(N'application_name')`.
/// Requires SQL Server 2016 or later.
pub const SET_APPLICATION_NAME_QUERY: &str = "EXEC sp_set_session_context @key = N'application_name', @value = $name";

/// SQL query to describe table columns using `information_schema`.
pub const DESCRIBE_TABLE_QUERY: &str = r"
SELECT
    column_name AS name,
    data_type + COALESCE('(' + CAST(character_maximum_length AS VARCHAR) + ')', '') AS type,
    CASE WHEN is_nullable = 'YES' THEN 1 ELSE 0 END AS nullable,
    column_default AS [default],
    ordinal_position AS ordinal
FROM information_schema.columns
WHERE table_name = $table
  AND table_schema = COALESCE($schema!n, SCHEMA_NAME())
ORDER BY ordinal_position
";

pub const SETTINGS: Settings = Settings {
    collapsible_in_enabled: true,
    escaping_double_single_quotes: true,
    comment_hash: false,     // # not comment in T-SQL
    column_backticks: false, // uses brackets [column] or quotes
    placeholder_dollar_sign: false,
    placeholder_at_sign: true, // @p1, @p2, etc.
    max_placeholders: 2100,    // hardcoded MSSQL limit
    strict_placeholders: false,

    booleans_as_literals: false, // MSSQL doesn't have BOOLEAN: use 1/0
    strings_as_ntext: true,      // use N'string' for Unicode
    cast_json: Some("AS NVARCHAR(MAX)"), // or omit, depending on query
    escape_backslash: false,
    upsert_style: UpsertStyle::Unsupported,
    identifier_quote_style: IdentifierQuoteStyle::Bracket,
};

php_sqlx_impl_driver_inner!(MssqlDriverInner, Mssql);

impl MssqlDriverInner {
    /// Bulk-ingest fast path is not available for `MSSQL`.
    ///
    /// SQL Server has no `COPY`-equivalent exposed by the underlying driver.
    /// Use `insertMany()` instead.
    #[allow(clippy::unused_self)]
    pub fn copy_in(
        &self,
        _table: &str,
        _rows: &ext_php_rs::types::Zval,
    ) -> crate::error::Result<u64> {
        Err(crate::error::Error::Other(
            "copyIn (bulk COPY) is only supported on PostgreSQL; use insertMany() on MSSQL"
                .to_string(),
        ))
    }
}
