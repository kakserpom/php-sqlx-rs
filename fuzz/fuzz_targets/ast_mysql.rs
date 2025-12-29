#![no_main]

use libfuzzer_sys::fuzz_target;
use php_sqlx::ast::{Ast, Settings};

/// MySQL-specific settings for AST parsing
const MYSQL_SETTINGS: Settings = Settings {
    collapsible_in_enabled: true,
    escaping_double_single_quotes: true,
    comment_hash: true,
    column_backticks: true,
    placeholder_dollar_sign: false,
    placeholder_at_sign: false,
    max_placeholders: 65535,
    booleans_as_literals: false,
    strings_as_ntext: false,
    cast_json: None,
    escape_backslash: true,
};

fuzz_target!(|data: &str| {
    // Parse should never panic, only return Ok or Err
    let _ = Ast::parse(data, &MYSQL_SETTINGS);
});
