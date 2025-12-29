#![no_main]

use libfuzzer_sys::fuzz_target;
use php_sqlx::ast::{Ast, Settings};

/// PostgreSQL-specific settings for AST parsing
const POSTGRES_SETTINGS: Settings = Settings {
    collapsible_in_enabled: true,
    escaping_double_single_quotes: true,
    comment_hash: false,
    column_backticks: false,
    placeholder_dollar_sign: true,
    placeholder_at_sign: false,
    max_placeholders: 65535,
    booleans_as_literals: true,
    strings_as_ntext: false,
    cast_json: Some("::jsonb"),
    escape_backslash: false,
};

fuzz_target!(|data: &str| {
    // Parse should never panic, only return Ok or Err
    let _ = Ast::parse(data, &POSTGRES_SETTINGS);
});
