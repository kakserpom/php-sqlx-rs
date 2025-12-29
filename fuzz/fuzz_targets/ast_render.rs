#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use php_sqlx::ast::{Ast, Settings};
use php_sqlx::param_value::{ParameterValue, ParamsMap};

/// Input for fuzzing both parsing and rendering
#[derive(Debug, Arbitrary)]
struct FuzzInput {
    sql: String,
    db_type: DbType,
    params: Vec<(String, ParamType)>,
}

#[derive(Debug, Arbitrary, Clone, Copy)]
enum DbType {
    Postgres,
    Mysql,
    Mssql,
}

#[derive(Debug, Arbitrary)]
enum ParamType {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    IntArray(Vec<i64>),
    StringArray(Vec<String>),
}

impl ParamType {
    fn into_parameter_value(self) -> ParameterValue {
        match self {
            Self::Null => ParameterValue::Null,
            Self::Bool(b) => ParameterValue::Bool(b),
            Self::Int(i) => ParameterValue::Int(i),
            Self::Float(f) => ParameterValue::Float(f),
            Self::String(s) => ParameterValue::String(s),
            Self::IntArray(arr) => {
                ParameterValue::Array(arr.into_iter().map(ParameterValue::Int).collect())
            }
            Self::StringArray(arr) => {
                ParameterValue::Array(arr.into_iter().map(ParameterValue::String).collect())
            }
        }
    }
}

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

fuzz_target!(|input: FuzzInput| {
    let settings = match input.db_type {
        DbType::Postgres => &POSTGRES_SETTINGS,
        DbType::Mysql => &MYSQL_SETTINGS,
        DbType::Mssql => &MSSQL_SETTINGS,
    };

    // Parse should never panic
    let ast = match Ast::parse(&input.sql, settings) {
        Ok(ast) => ast,
        Err(_) => return,
    };

    // Build params map
    let mut params = ParamsMap::new();
    for (name, value) in input.params {
        params.insert(name, value.into_parameter_value());
    }

    // Render should never panic
    let _ = ast.render(params, settings);
});
