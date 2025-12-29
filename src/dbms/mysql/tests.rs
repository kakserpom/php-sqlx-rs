use crate::ast::Ast;
use crate::dbms::mysql::inner::{DESCRIBE_TABLE_QUERY, SETTINGS};
use crate::param_value::ParamsMap;

fn into_ast(sql: &str) -> Ast {
    Ast::parse(sql, &SETTINGS).expect("failed to parse SQL statement")
}

#[test]
fn test_describe_table_query_parses() {
    // Ensure the DESCRIBE_TABLE_QUERY can be parsed
    let ast = into_ast(DESCRIBE_TABLE_QUERY);
    let Ast::Root {
        required_placeholders,
        ..
    } = &ast
    else {
        panic!("Expected Root variant");
    };
    // Should have table and schema as required placeholders
    assert!(
        required_placeholders.contains(&"table".to_string()),
        "Expected 'table' placeholder"
    );
    assert!(
        required_placeholders.contains(&"schema".to_string()),
        "Expected 'schema' placeholder"
    );
}

#[test]
fn test_describe_table_query_renders() {
    use crate::param_value::ParameterValue;

    let ast = into_ast(DESCRIBE_TABLE_QUERY);
    let mut params = ParamsMap::new();
    params.insert("table".into(), ParameterValue::String("users".into()));
    params.insert("schema".into(), ParameterValue::Null);

    let (sql, values) = ast.render(params, &SETTINGS).expect("Rendering failed");

    // MySQL uses ? placeholders
    assert!(sql.contains('?'), "Expected ? placeholder");
    assert_eq!(values.len(), 2);
    assert_eq!(values[0], ParameterValue::String("users".into()));
    assert_eq!(values[1], ParameterValue::Null);
}
