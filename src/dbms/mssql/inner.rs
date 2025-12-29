#![allow(clippy::needless_pass_by_value)]
use crate::php_sqlx_impl_driver_inner;

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
  AND table_schema = COALESCE($schema, SCHEMA_NAME())
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

    booleans_as_literals: false, // MSSQL doesn't have BOOLEAN: use 1/0
    strings_as_ntext: true,      // use N'string' for Unicode
    cast_json: Some("AS NVARCHAR(MAX)"), // or omit, depending on query
    escape_backslash: false,
};

php_sqlx_impl_driver_inner!(MssqlDriverInner, Mssql);
