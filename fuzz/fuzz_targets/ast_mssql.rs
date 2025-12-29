#![no_main]

use libfuzzer_sys::fuzz_target;
use php_sqlx::ast::{Ast, Settings};

/// MSSQL-specific settings for AST parsing
const MSSQL_SETTINGS: Settings = Settings {
    collapsible_in_enabled: true,
    escaping_double_single_quotes: true,
    comment_hash: false,
    column_backticks: false,
    placeholder_dollar_sign: false,
    placeholder_at_sign: true,
    max_placeholders: 2100,
    booleans_as_literals: false,
    strings_as_ntext: true,
    cast_json: None,
    escape_backslash: false,
};

fuzz_target!(|data: &str| {
    // Parse should never panic, only return Ok or Err
    let _ = Ast::parse(data, &MSSQL_SETTINGS);
});
