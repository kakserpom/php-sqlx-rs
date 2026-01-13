use crate::ast::Ast;
use crate::by_clause::{ByClause, ByClauseColumnDefinition};
use crate::dbms::postgres::inner::SETTINGS;
use crate::paginate_clause::PaginateClause;
use crate::param_value::{ParameterValue, ParamsMap};
use collapse::*;

fn into_ast(sql: &str) -> Ast {
    Ast::parse(sql, &SETTINGS).expect("failed to parse SQL statement")
}

#[test]
fn test_named_and_positional() {
    let sql = "SELECT :param, ?, ? FROM table WHERE {{ x = $x }}";
    if let Ast::Root {
        branches,
        required_placeholders,
    } = into_ast(sql)
    {
        println!("{required_placeholders:#?}");
        let names: Vec<&str> = branches
            .iter()
            .filter_map(|b| {
                if let Ast::Placeholder { name, .. } = b {
                    Some(name.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert!(names.contains(&"param"));
        assert!(names.contains(&"1"));
        assert!(names.contains(&"2"));
    } else {
        panic!("Expected Nested");
    }
}

#[test]
fn test_render_basic() {
    let sql = "SELECT * FROM users WHERE {{status = $status AND}} id = $id";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("status".into(), "active".into());
    vals.insert("id".into(), "42".into());
    let (query, params) = ast.render(vals, &SETTINGS).expect("Rendering failed");
    collapsed_eq!(&query, "SELECT * FROM users WHERE status = $1 AND id = $2");
    assert_eq!(params, vec!["active".into(), "42".into()]);
}

#[test]
fn test_render_optional_skip() {
    let sql = "SELECT * FROM users WHERE {{status = $status AND}} id = $id";
    let ast = into_ast(sql);
    let (query, params) = ast
        .render([("id", 100)], &SETTINGS)
        .expect("Rendering failed");
    collapsed_eq!(&query, "SELECT * FROM users WHERE id = $1");
    assert_eq!(params, vec![100.into()]);
}

#[test]
fn test_render_var_types() {
    let sql =
        "SELECT * FROM table WHERE id = $id AND active = :flag AND scores IN (?) AND data = $data";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::new();
    vals.insert("id".into(), ParameterValue::Int(7));
    vals.insert("flag".into(), ParameterValue::Bool(true));
    vals.insert(
        "0".into(),
        ParameterValue::Array(vec![ParameterValue::Int(1), ParameterValue::Int(2)]),
    );
    vals.insert("data".into(), ParameterValue::String("xyz".into()));
    let (q, params) = ast.render(vals, &SETTINGS).expect("Rendering failed");
    collapsed_eq!(
        &q,
        "SELECT * FROM table WHERE id = $1 AND active = $2 AND scores IN ($3, $4) AND data = $5"
    );
    assert_eq!(
        params,
        vec![
            ParameterValue::Int(7),
            ParameterValue::Bool(true),
            ParameterValue::Int(1),
            ParameterValue::Int(2),
            ParameterValue::String("xyz".into()),
        ]
    );
}

#[test]
fn test_render_order_by_input() {
    let ob = ByClause::allowed([
        ("name", "users.name"),
        ("age", "users.age"),
        ("posts", "COUNT(posts.id)"),
    ])
    .unwrap();

    let rendered = ob.render(vec![
        ByClauseColumnDefinition::Short("name".into()),
        ByClauseColumnDefinition::Full(vec!["posts".into(), ByClause::_DESC.into()]),
    ]);

    let sql = "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id ORDER BY $order_by";
    let ast = into_ast(sql);
    let (query, params) = ast
        .render(
            [("order_by", ParameterValue::ByClauseRendered(rendered))],
            &SETTINGS,
        )
        .expect("Rendering failed");

    collapsed_eq!(
        &query,
        "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id ORDER BY users.name, COUNT(posts.id) DESC"
    );
    assert_eq!(params, vec![]);
}

#[test]
fn test_render_order_by_apply_empty() {
    use crate::by_clause::ByClause;

    let ob = ByClause::allowed([
        ("name", "users.name"),
        ("age", "users.age"),
        ("posts", "COUNT(posts.id)"),
    ])
    .unwrap();

    let rendered = ob.render(vec![]);

    let sql =
        "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id {{ ORDER BY $order_by }}";
    let ast = into_ast(sql);
    let (query, params) = ast
        .render(
            [("order_by", ParameterValue::ByClauseRendered(rendered))],
            &SETTINGS,
        )
        .expect("Rendering failed");

    collapsed_eq!(
        &query,
        "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id"
    );
    assert_eq!(params, vec![]);
}

#[test]
fn test_in_clause_parsing() {
    let sql = "SELECT * FROM users WHERE status IN :statuses AND age NOT IN (:ages)";
    let ast = into_ast(sql);
    println!("AST = {ast:#?}");
    let (q, p) = ast
        .render(
            [
                (
                    "statuses",
                    ParameterValue::Array(vec![
                        ParameterValue::Int(1),
                        ParameterValue::Int(2),
                        ParameterValue::Int(3),
                    ]),
                ),
                (
                    "ages",
                    ParameterValue::Array(vec![ParameterValue::Int(20), ParameterValue::Int(30)]),
                ),
            ],
            &SETTINGS,
        )
        .unwrap();

    collapsed_eq!(
        &q,
        "SELECT * FROM users WHERE status IN ($1, $2, $3) AND age NOT IN ($4, $5)"
    );
    assert_eq!(
        p,
        vec![
            ParameterValue::from(1),
            ParameterValue::from(2),
            ParameterValue::from(3),
            ParameterValue::from(20),
            ParameterValue::from(30),
        ]
    );
}

#[test]
fn test_parse_in_not_in_and_string() {
    let sql = "SELECT * FROM users WHERE name = 'O''Reilly' AND status IN (:statuses) AND age NOT IN (:ages)";
    let ast = into_ast(sql);
    println!("AST = {ast:#?}");
    let Ast::Root {
        branches,
        required_placeholders,
    } = ast
    else {
        panic!("AST root is not Root variant");
    };
    // Expect placeholders
    assert!(required_placeholders.is_empty());
    // Expect branch count
    //assert_eq!(branches.len(), 5);
    // Check sequence of branches
    match &branches[0] {
        Ast::Raw(s) => assert!(s.ends_with("name = 'O''Reilly' AND ")),
        _ => panic!("Expected Sql at branch 0"),
    }
    match &branches[1] {
        Ast::InClause {
            expr, placeholder, ..
        } => {
            assert_eq!(expr, "status");
            assert_eq!(placeholder, "statuses");
        }
        _ => panic!("Expected InClause at branch 1"),
    }
    match &branches[2] {
        Ast::Raw(s) => assert_eq!(s, " AND "),
        _ => panic!("Expected Sql at branch 2"),
    }
    match &branches[3] {
        Ast::NotInClause {
            expr, placeholder, ..
        } => {
            assert_eq!(expr, "age");
            assert_eq!(placeholder, "ages");
        }
        _ => panic!("Expected NotInClause at branch 3"),
    }
    assert_eq!(branches.len(), 4);
}

#[test]
fn test_parse_multi_in() {
    let sql = "SELECT * FROM users age IN (?, ?)";
    let ast = into_ast(sql);
    println!("AST = {ast:#?}");
    let Ast::Root {
        required_placeholders,
        ..
    } = ast
    else {
        panic!("Expected Root variant for MySqlAst");
    };
    assert_eq!(required_placeholders.len(), 2);
}

#[test]
fn test_parse_required_in() {
    let sql = "SELECT * FROM users age IN (?/*required*/)";
    let ast = into_ast(sql);
    println!("AST = {ast:#?}");
    let Ast::Root {
        required_placeholders,
        ..
    } = ast
    else {
        panic!("Expected Root variant for MySqlAst");
    };
    assert_eq!(required_placeholders.len(), 1);
}

#[test]
fn test_pagination() {
    let sql = "SELECT * FROM users age ORDER BY id PAGINATE :pagination";
    let ast = into_ast(sql);
    println!("AST = {ast:#?}");
    assert_eq!(
        ast,
        Ast::Root {
            branches: vec![
                Ast::Raw(String::from("SELECT * FROM users age ORDER BY id  "),),
                Ast::PaginateClause {
                    placeholder: String::from("pagination"),
                },
            ],
            required_placeholders: vec![String::from("pagination")],
        }
    );

    let mut paginate_clause = PaginateClause::new();

    paginate_clause.min_per_page = 1;
    paginate_clause.max_per_page = 10;
    paginate_clause.default_per_page = 5;

    let mut vals = ParamsMap::default();
    vals.insert(
        "pagination".into(),
        ParameterValue::PaginateClauseRendered(paginate_clause.input(Some(7), None)),
    );
    let (sql, values) = ast.render(vals, &SETTINGS).unwrap();
    println!("sql = {sql:#?}");
    collapsed_eq!(
        &sql,
        "SELECT * FROM users age ORDER BY id LIMIT $1 OFFSET $2"
    );
    assert_eq!(
        values,
        vec![ParameterValue::Int(5), ParameterValue::Int(35)]
    );
}

/// Test that null values are rejected for non-nullable placeholders
#[test]
fn test_null_placeholder_rejected_for_non_nullable() {
    let sql = "SELECT * FROM users WHERE name = :name";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("name".into(), ParameterValue::Null);
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_err(),
        "Null value should be rejected for non-nullable placeholder"
    );
}

/// Test that null values are allowed for explicitly nullable placeholders
#[test]
fn test_null_placeholder_accepted_for_nullable() {
    let sql = "SELECT * FROM users WHERE name = :name!n";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("name".into(), ParameterValue::Null);
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Null value should be valid for nullable placeholder"
    );
    let (query, params) = result.unwrap();
    collapsed_eq!(&query, "SELECT * FROM users WHERE name = $1");
    assert_eq!(params, vec![ParameterValue::Null]);
}

/// Test that absent placeholders still error
#[test]
fn test_absent_placeholder_errors() {
    let sql = "SELECT * FROM users WHERE name = :name";
    let ast = into_ast(sql);
    let vals = ParamsMap::default(); // empty - no "name" provided
    let result = ast.render(vals, &SETTINGS);
    assert!(result.is_err(), "Absent placeholder should error");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("name"),
        "Error should mention the missing placeholder name"
    );
}

/// Test that null values in conditional blocks cause the block to be skipped
#[test]
fn test_null_in_conditional_block_skips() {
    let sql = "SELECT * FROM users WHERE id = :id {{ AND name = :name }}";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("id".into(), ParameterValue::Int(1));
    vals.insert("name".into(), ParameterValue::Null);
    let (query, params) = ast.render(vals, &SETTINGS).unwrap();
    // Block should be skipped because name is null
    collapsed_eq!(&query, "SELECT * FROM users WHERE id = $1");
    assert_eq!(params, vec![ParameterValue::Int(1)]);
}

/// Test that absent placeholders in conditional blocks cause the block to be skipped
#[test]
fn test_absent_in_conditional_block_skips() {
    let sql = "SELECT * FROM users WHERE id = :id {{ AND name = :name }}";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("id".into(), ParameterValue::Int(1));
    // name is not provided at all
    let (query, params) = ast.render(vals, &SETTINGS).unwrap();
    // Block should be skipped because name is absent
    collapsed_eq!(&query, "SELECT * FROM users WHERE id = $1");
    assert_eq!(params, vec![ParameterValue::Int(1)]);
}

/// Test that empty arrays are allowed for required IN clause placeholders
#[test]
fn test_empty_array_in_clause_renders_false() {
    let sql = "SELECT * FROM users WHERE status IN :statuses";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("statuses".into(), ParameterValue::Array(vec![]));
    let result = ast.render(vals, &SETTINGS);
    assert!(result.is_ok(), "Empty array should be valid for IN clause");
    let (query, _) = result.unwrap();
    // Empty IN clause should render as FALSE
    assert!(
        query.contains("FALSE"),
        "Empty IN clause should render as FALSE"
    );
}

// ============================================================================
// Typed Placeholder Tests
// ============================================================================

use crate::ast::PlaceholderType;

/// Test parsing of typed positional placeholders (?i, ?s, ?d)
#[test]
fn test_parse_typed_positional_placeholders() {
    let sql = "SELECT * FROM users WHERE id = ?i AND name = ?s AND score = ?d";
    let ast = into_ast(sql);
    if let Ast::Root { branches, .. } = ast {
        let placeholders: Vec<_> = branches
            .iter()
            .filter_map(|b| {
                if let Ast::Placeholder {
                    name,
                    expected_type,
                    ..
                } = b
                {
                    Some((name.as_str(), *expected_type))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(placeholders.len(), 3);
        assert_eq!(placeholders[0], ("1", Some(PlaceholderType::Int)));
        assert_eq!(placeholders[1], ("2", Some(PlaceholderType::String)));
        assert_eq!(placeholders[2], ("3", Some(PlaceholderType::Decimal)));
    } else {
        panic!("Expected Root");
    }
}

/// Test parsing of typed named placeholders (:name!i, $name!s)
#[test]
fn test_parse_typed_named_placeholders() {
    let sql = "SELECT * FROM users WHERE id = :user_id!i AND name = $username!s AND age = :age!u";
    let ast = into_ast(sql);
    if let Ast::Root { branches, .. } = ast {
        let placeholders: Vec<_> = branches
            .iter()
            .filter_map(|b| {
                if let Ast::Placeholder {
                    name,
                    expected_type,
                    ..
                } = b
                {
                    Some((name.as_str(), *expected_type))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(placeholders.len(), 3);
        assert_eq!(placeholders[0], ("user_id", Some(PlaceholderType::Int)));
        assert_eq!(placeholders[1], ("username", Some(PlaceholderType::String)));
        assert_eq!(placeholders[2], ("age", Some(PlaceholderType::UnsignedInt)));
    } else {
        panic!("Expected Root");
    }
}

/// Test that untyped placeholders still work
#[test]
fn test_untyped_placeholders_still_work() {
    let sql = "SELECT * FROM users WHERE id = :id AND name = ?";
    let ast = into_ast(sql);
    if let Ast::Root { branches, .. } = ast {
        let placeholders: Vec<_> = branches
            .iter()
            .filter_map(|b| {
                if let Ast::Placeholder {
                    name,
                    expected_type,
                    ..
                } = b
                {
                    Some((name.as_str(), *expected_type))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(placeholders.len(), 2);
        assert_eq!(placeholders[0], ("id", None));
        assert_eq!(placeholders[1], ("1", None));
    } else {
        panic!("Expected Root");
    }
}

/// Test type validation with correct types
#[test]
fn test_typed_placeholder_valid_int() {
    let sql = "SELECT * FROM users WHERE id = :id!i";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("id".into(), ParameterValue::Int(42));
    let result = ast.render(vals, &SETTINGS);
    assert!(result.is_ok(), "Integer should be valid for !i placeholder");
    let (query, params) = result.unwrap();
    collapsed_eq!(&query, "SELECT * FROM users WHERE id = $1");
    assert_eq!(params, vec![ParameterValue::Int(42)]);
}

/// Test type validation with correct string type
#[test]
fn test_typed_placeholder_valid_string() {
    let sql = "SELECT * FROM users WHERE name = ?s";
    let ast = into_ast(sql);
    let result = ast.render([("0", ParameterValue::String("John".into()))], &SETTINGS);
    assert!(result.is_ok(), "String should be valid for ?s placeholder");
    let (query, params) = result.unwrap();
    collapsed_eq!(&query, "SELECT * FROM users WHERE name = $1");
    assert_eq!(params, vec![ParameterValue::String("John".into())]);
}

/// Test type validation with correct decimal type (float)
#[test]
fn test_typed_placeholder_valid_decimal_float() {
    let sql = "SELECT * FROM users WHERE score = :score!d";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("score".into(), ParameterValue::Float(3.5));
    let result = ast.render(vals, &SETTINGS);
    assert!(result.is_ok(), "Float should be valid for !d placeholder");
}

/// Test type validation with correct decimal type (int coercion)
#[test]
fn test_typed_placeholder_valid_decimal_int() {
    let sql = "SELECT * FROM users WHERE score = :score!d";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("score".into(), ParameterValue::Int(42));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Int should also be valid for !d placeholder"
    );
}

/// Test type validation with correct integer array type
#[test]
fn test_typed_placeholder_valid_int_array() {
    let sql = "SELECT * FROM users WHERE id IN :ids!ia";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert(
        "ids".into(),
        ParameterValue::Array(vec![ParameterValue::Int(1), ParameterValue::Int(2)]),
    );
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Int array should be valid for !ia placeholder"
    );
}

/// Test type validation with unsigned int constraint
#[test]
fn test_typed_placeholder_valid_unsigned_int() {
    let sql = "SELECT * FROM users WHERE age = :age!u";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("age".into(), ParameterValue::Int(25));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Positive int should be valid for !u placeholder"
    );
}

/// Test type validation with unsigned int constraint (zero is valid)
#[test]
fn test_typed_placeholder_valid_unsigned_int_zero() {
    let sql = "SELECT * FROM users WHERE age = :age!u";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("age".into(), ParameterValue::Int(0));
    let result = ast.render(vals, &SETTINGS);
    assert!(result.is_ok(), "Zero should be valid for !u placeholder");
}

/// Test that null is allowed for nullable typed placeholders (!ni, !ns)
#[test]
fn test_typed_placeholder_allows_null() {
    let sql = "SELECT * FROM users WHERE id = :id!ni AND name = :name!ns";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("id".into(), ParameterValue::Null);
    vals.insert("name".into(), ParameterValue::Null);
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Null should be valid for nullable typed placeholders"
    );
}

/// Test that null is NOT allowed for non-nullable typed placeholders (!i, !s)
#[test]
fn test_typed_placeholder_rejects_null() {
    let sql = "SELECT * FROM users WHERE id = :id!i";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("id".into(), ParameterValue::Null);
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_err(),
        "Null should NOT be valid for non-nullable typed placeholder"
    );
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Type mismatch"));
}

/// Test type mismatch error: string for integer placeholder
#[test]
fn test_typed_placeholder_mismatch_string_for_int() {
    let sql = "SELECT * FROM users WHERE id = :id!i";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("id".into(), ParameterValue::String("not-an-int".into()));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_err(),
        "String should not be valid for !i placeholder"
    );
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Type mismatch"));
    assert!(err.to_string().contains("id"));
    assert!(err.to_string().contains("integer"));
    assert!(err.to_string().contains("string"));
}

/// Test type mismatch error: int for string placeholder
#[test]
fn test_typed_placeholder_mismatch_int_for_string() {
    let sql = "SELECT * FROM users WHERE name = ?s";
    let ast = into_ast(sql);
    let result = ast.render([("0", ParameterValue::Int(42))], &SETTINGS);
    assert!(
        result.is_err(),
        "Int should not be valid for ?s placeholder"
    );
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Type mismatch"));
}

/// Test type mismatch error: negative int for unsigned placeholder
#[test]
fn test_typed_placeholder_mismatch_negative_for_unsigned() {
    let sql = "SELECT * FROM users WHERE age = :age!u";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("age".into(), ParameterValue::Int(-5));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_err(),
        "Negative int should not be valid for !u placeholder"
    );
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Type mismatch"));
}

/// Test type mismatch error: string for integer array placeholder
#[test]
fn test_typed_placeholder_mismatch_string_for_int_array() {
    let sql = "SELECT * FROM users WHERE id IN :ids!ia";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("ids".into(), ParameterValue::String("not-an-array".into()));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_err(),
        "String should not be valid for !ia placeholder"
    );
}

/// Test type mismatch error: string elements in integer array
#[test]
fn test_typed_placeholder_mismatch_string_elements_in_int_array() {
    let sql = "SELECT * FROM users WHERE id IN :ids!ia";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert(
        "ids".into(),
        ParameterValue::Array(vec![
            ParameterValue::Int(1),
            ParameterValue::String("not-an-int".into()),
        ]),
    );
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_err(),
        "Array with string elements should not be valid for !ia placeholder"
    );
}

// ============================================================================
// Unsigned Decimal (?ud, !ud) Tests
// ============================================================================

/// Test parsing of unsigned decimal placeholder (?ud)
#[test]
fn test_parse_unsigned_decimal_positional() {
    let sql = "SELECT * FROM products WHERE price = ?ud";
    let ast = into_ast(sql);
    if let Ast::Root { branches, .. } = ast {
        let placeholders: Vec<_> = branches
            .iter()
            .filter_map(|b| {
                if let Ast::Placeholder {
                    name,
                    expected_type,
                    ..
                } = b
                {
                    Some((name.as_str(), *expected_type))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(placeholders.len(), 1);
        assert_eq!(
            placeholders[0],
            ("1", Some(PlaceholderType::UnsignedDecimal))
        );
    } else {
        panic!("Expected Root");
    }
}

/// Test parsing of unsigned decimal placeholder (:name!ud)
#[test]
fn test_parse_unsigned_decimal_named() {
    let sql = "SELECT * FROM products WHERE price = :price!ud";
    let ast = into_ast(sql);
    if let Ast::Root { branches, .. } = ast {
        let placeholders: Vec<_> = branches
            .iter()
            .filter_map(|b| {
                if let Ast::Placeholder {
                    name,
                    expected_type,
                    ..
                } = b
                {
                    Some((name.as_str(), *expected_type))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(placeholders.len(), 1);
        assert_eq!(
            placeholders[0],
            ("price", Some(PlaceholderType::UnsignedDecimal))
        );
    } else {
        panic!("Expected Root");
    }
}

/// Test unsigned decimal accepts positive float
#[test]
fn test_typed_placeholder_valid_unsigned_decimal_float() {
    let sql = "SELECT * FROM products WHERE price = :price!ud";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("price".into(), ParameterValue::Float(19.99));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Positive float should be valid for !ud placeholder"
    );
}

/// Test unsigned decimal accepts zero float
#[test]
fn test_typed_placeholder_valid_unsigned_decimal_zero_float() {
    let sql = "SELECT * FROM products WHERE price = :price!ud";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("price".into(), ParameterValue::Float(0.0));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Zero float should be valid for !ud placeholder"
    );
}

/// Test unsigned decimal accepts positive int
#[test]
fn test_typed_placeholder_valid_unsigned_decimal_int() {
    let sql = "SELECT * FROM products WHERE price = :price!ud";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("price".into(), ParameterValue::Int(100));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Positive int should be valid for !ud placeholder"
    );
}

/// Test decimal accepts numeric string
#[test]
fn test_typed_placeholder_valid_decimal_string() {
    let sql = "SELECT * FROM products WHERE price = :price!d";
    let ast = into_ast(sql);

    // Positive numeric string
    let mut vals = ParamsMap::default();
    vals.insert("price".into(), ParameterValue::String("123.45".into()));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Positive numeric string should be valid for !d placeholder"
    );

    // Negative numeric string
    let mut vals = ParamsMap::default();
    vals.insert("price".into(), ParameterValue::String("-99.99".into()));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Negative numeric string should be valid for !d placeholder"
    );

    // Integer string
    let mut vals = ParamsMap::default();
    vals.insert("price".into(), ParameterValue::String("42".into()));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Integer string should be valid for !d placeholder"
    );

    // String with whitespace
    let mut vals = ParamsMap::default();
    vals.insert("price".into(), ParameterValue::String("  123.45  ".into()));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Numeric string with whitespace should be valid for !d placeholder"
    );
}

/// Test decimal rejects non-numeric string
#[test]
fn test_typed_placeholder_mismatch_non_numeric_string_for_decimal() {
    let sql = "SELECT * FROM products WHERE price = :price!d";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert(
        "price".into(),
        ParameterValue::String("not-a-number".into()),
    );
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_err(),
        "Non-numeric string should not be valid for !d placeholder"
    );
}

/// Test unsigned decimal accepts positive numeric string
#[test]
fn test_typed_placeholder_valid_unsigned_decimal_string() {
    let sql = "SELECT * FROM products WHERE price = :price!ud";
    let ast = into_ast(sql);

    // Positive numeric string
    let mut vals = ParamsMap::default();
    vals.insert("price".into(), ParameterValue::String("123.45".into()));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Positive numeric string should be valid for !ud placeholder"
    );

    // Zero string
    let mut vals = ParamsMap::default();
    vals.insert("price".into(), ParameterValue::String("0".into()));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Zero string should be valid for !ud placeholder"
    );
}

/// Test unsigned decimal rejects negative numeric string
#[test]
fn test_typed_placeholder_mismatch_negative_string_for_unsigned_decimal() {
    let sql = "SELECT * FROM products WHERE price = :price!ud";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("price".into(), ParameterValue::String("-5.00".into()));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_err(),
        "Negative numeric string should not be valid for !ud placeholder"
    );
}

/// Test decimal array accepts numeric strings
#[test]
fn test_typed_placeholder_valid_decimal_array_with_strings() {
    let sql = "SELECT * FROM products WHERE price IN :prices!da";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert(
        "prices".into(),
        ParameterValue::Array(vec![
            ParameterValue::Float(19.99),
            ParameterValue::String("25.50".into()),
            ParameterValue::Int(30),
        ]),
    );
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Decimal array with numeric strings should be valid for !da placeholder"
    );
}

/// Test unsigned decimal rejects negative float
#[test]
fn test_typed_placeholder_mismatch_negative_float_for_unsigned_decimal() {
    let sql = "SELECT * FROM products WHERE price = :price!ud";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("price".into(), ParameterValue::Float(-9.99));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_err(),
        "Negative float should not be valid for !ud placeholder"
    );
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Type mismatch"));
}

/// Test unsigned decimal rejects negative int
#[test]
fn test_typed_placeholder_mismatch_negative_int_for_unsigned_decimal() {
    let sql = "SELECT * FROM products WHERE price = :price!ud";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("price".into(), ParameterValue::Int(-100));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_err(),
        "Negative int should not be valid for !ud placeholder"
    );
}

// ============================================================================
// Typed Array Tests (?ia, ?sa, ?da, ?ua, ?uda)
// ============================================================================

/// Test parsing of typed array placeholders in IN clauses
#[test]
fn test_parse_typed_array_placeholders() {
    let sql = "SELECT * FROM t WHERE a IN :ids!ia AND b IN :names!sa AND c IN :scores!da";
    let ast = into_ast(sql);
    if let Ast::Root { branches, .. } = ast {
        let in_clauses: Vec<_> = branches
            .iter()
            .filter_map(|b| {
                if let Ast::InClause {
                    placeholder,
                    expected_type,
                    ..
                } = b
                {
                    Some((placeholder.as_str(), *expected_type))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(in_clauses.len(), 3);
        assert_eq!(in_clauses[0], ("ids", Some(PlaceholderType::IntArray)));
        assert_eq!(in_clauses[1], ("names", Some(PlaceholderType::StringArray)));
        assert_eq!(
            in_clauses[2],
            ("scores", Some(PlaceholderType::DecimalArray))
        );
    } else {
        panic!("Expected Root");
    }
}

/// Test parsing of unsigned typed array placeholders in IN clauses
#[test]
fn test_parse_unsigned_typed_array_placeholders() {
    let sql = "SELECT * FROM t WHERE a IN :ids!ua AND b IN :prices!uda";
    let ast = into_ast(sql);
    if let Ast::Root { branches, .. } = ast {
        let in_clauses: Vec<_> = branches
            .iter()
            .filter_map(|b| {
                if let Ast::InClause {
                    placeholder,
                    expected_type,
                    ..
                } = b
                {
                    Some((placeholder.as_str(), *expected_type))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(in_clauses.len(), 2);
        assert_eq!(
            in_clauses[0],
            ("ids", Some(PlaceholderType::UnsignedIntArray))
        );
        assert_eq!(
            in_clauses[1],
            ("prices", Some(PlaceholderType::UnsignedDecimalArray))
        );
    } else {
        panic!("Expected Root");
    }
}

/// Test string array validation
#[test]
fn test_typed_placeholder_valid_string_array() {
    let sql = "SELECT * FROM users WHERE name IN :names!sa";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert(
        "names".into(),
        ParameterValue::Array(vec![
            ParameterValue::String("Alice".into()),
            ParameterValue::String("Bob".into()),
        ]),
    );
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "String array should be valid for !sa placeholder"
    );
}

/// Test decimal array validation
#[test]
fn test_typed_placeholder_valid_decimal_array() {
    let sql = "SELECT * FROM products WHERE price IN :prices!da";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert(
        "prices".into(),
        ParameterValue::Array(vec![ParameterValue::Float(19.99), ParameterValue::Int(20)]),
    );
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Decimal array (mixed float/int) should be valid for !da placeholder"
    );
}

/// Test unsigned int array validation
#[test]
fn test_typed_placeholder_valid_unsigned_int_array() {
    let sql = "SELECT * FROM users WHERE age IN :ages!ua";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert(
        "ages".into(),
        ParameterValue::Array(vec![ParameterValue::Int(0), ParameterValue::Int(25)]),
    );
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Unsigned int array should be valid for !ua placeholder"
    );
}

/// Test unsigned int array rejects negative values
#[test]
fn test_typed_placeholder_mismatch_negative_in_unsigned_int_array() {
    let sql = "SELECT * FROM users WHERE age IN :ages!ua";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert(
        "ages".into(),
        ParameterValue::Array(vec![ParameterValue::Int(25), ParameterValue::Int(-1)]),
    );
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_err(),
        "Array with negative int should not be valid for !ua placeholder"
    );
}

/// Test unsigned decimal array validation
#[test]
fn test_typed_placeholder_valid_unsigned_decimal_array() {
    let sql = "SELECT * FROM products WHERE price IN :prices!uda";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert(
        "prices".into(),
        ParameterValue::Array(vec![
            ParameterValue::Float(0.0),
            ParameterValue::Float(19.99),
            ParameterValue::Int(100),
        ]),
    );
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Unsigned decimal array should be valid for !uda placeholder"
    );
}

/// Test unsigned decimal array rejects negative values
#[test]
fn test_typed_placeholder_mismatch_negative_in_unsigned_decimal_array() {
    let sql = "SELECT * FROM products WHERE price IN :prices!uda";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert(
        "prices".into(),
        ParameterValue::Array(vec![
            ParameterValue::Float(19.99),
            ParameterValue::Float(-0.01),
        ]),
    );
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_err(),
        "Array with negative float should not be valid for !uda placeholder"
    );
}

/// Test string array rejects integer elements
#[test]
fn test_typed_placeholder_mismatch_int_in_string_array() {
    let sql = "SELECT * FROM users WHERE name IN :names!sa";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert(
        "names".into(),
        ParameterValue::Array(vec![
            ParameterValue::String("Alice".into()),
            ParameterValue::Int(42),
        ]),
    );
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_err(),
        "Array with int should not be valid for !sa placeholder"
    );
}

/// Test null elements are allowed in typed arrays
#[test]
fn test_typed_array_allows_null_elements() {
    let sql = "SELECT * FROM users WHERE id IN :ids!ia";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert(
        "ids".into(),
        ParameterValue::Array(vec![
            ParameterValue::Int(1),
            ParameterValue::Null,
            ParameterValue::Int(3),
        ]),
    );
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Null elements should be allowed in typed arrays"
    );
}

// ============================================================================
// Nullable Prefix Tests (?n, ?ni, ?ns, !n, !ni, !ns, etc.)
// ============================================================================

/// Test parsing nullable mixed (?n)
#[test]
fn test_parse_nullable_mixed() {
    let sql = "SELECT * FROM users WHERE id = ?n";
    let ast = into_ast(sql);
    if let Ast::Root { branches, .. } = ast {
        let placeholders: Vec<_> = branches
            .iter()
            .filter_map(|b| {
                if let Ast::Placeholder {
                    name,
                    expected_type,
                    nullable,
                } = b
                {
                    Some((name.as_str(), *expected_type, *nullable))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(placeholders.len(), 1);
        assert_eq!(placeholders[0], ("1", None, true));
    } else {
        panic!("Expected Root");
    }
}

/// Test parsing nullable integer (?ni)
#[test]
fn test_parse_nullable_int() {
    let sql = "SELECT * FROM users WHERE id = ?ni AND age = :age!ni";
    let ast = into_ast(sql);
    if let Ast::Root { branches, .. } = ast {
        let placeholders: Vec<_> = branches
            .iter()
            .filter_map(|b| {
                if let Ast::Placeholder {
                    name,
                    expected_type,
                    nullable,
                } = b
                {
                    Some((name.as_str(), *expected_type, *nullable))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(placeholders.len(), 2);
        assert_eq!(placeholders[0], ("1", Some(PlaceholderType::Int), true));
        assert_eq!(placeholders[1], ("age", Some(PlaceholderType::Int), true));
    } else {
        panic!("Expected Root");
    }
}

/// Test parsing nullable unsigned decimal (?nud)
#[test]
fn test_parse_nullable_unsigned_decimal() {
    let sql = "SELECT * FROM products WHERE price = ?nud AND discount = $discount!nud";
    let ast = into_ast(sql);
    if let Ast::Root { branches, .. } = ast {
        let placeholders: Vec<_> = branches
            .iter()
            .filter_map(|b| {
                if let Ast::Placeholder {
                    name,
                    expected_type,
                    nullable,
                } = b
                {
                    Some((name.as_str(), *expected_type, *nullable))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(placeholders.len(), 2);
        assert_eq!(
            placeholders[0],
            ("1", Some(PlaceholderType::UnsignedDecimal), true)
        );
        assert_eq!(
            placeholders[1],
            ("discount", Some(PlaceholderType::UnsignedDecimal), true)
        );
    } else {
        panic!("Expected Root");
    }
}

/// Test parsing nullable unsigned decimal array (?nuda)
#[test]
fn test_parse_nullable_unsigned_decimal_array() {
    let sql = "SELECT * FROM products WHERE price IN :prices!nuda";
    let ast = into_ast(sql);
    if let Ast::Root { branches, .. } = ast {
        let in_clauses: Vec<_> = branches
            .iter()
            .filter_map(|b| {
                if let Ast::InClause {
                    placeholder,
                    expected_type,
                    nullable,
                    ..
                } = b
                {
                    Some((placeholder.as_str(), *expected_type, *nullable))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(in_clauses.len(), 1);
        assert_eq!(
            in_clauses[0],
            ("prices", Some(PlaceholderType::UnsignedDecimalArray), true)
        );
    } else {
        panic!("Expected Root");
    }
}

/// Test nullable int accepts null value
#[test]
fn test_nullable_int_accepts_null() {
    let sql = "SELECT * FROM users WHERE id = :id!ni";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("id".into(), ParameterValue::Null);
    let result = ast.render(vals, &SETTINGS);
    assert!(result.is_ok(), "Nullable int should accept null value");
}

/// Test nullable int accepts int value
#[test]
fn test_nullable_int_accepts_int() {
    let sql = "SELECT * FROM users WHERE id = :id!ni";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("id".into(), ParameterValue::Int(42));
    let result = ast.render(vals, &SETTINGS);
    assert!(result.is_ok(), "Nullable int should accept int value");
}

/// Test nullable int rejects string value
#[test]
fn test_nullable_int_rejects_string() {
    let sql = "SELECT * FROM users WHERE id = :id!ni";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("id".into(), ParameterValue::String("hello".into()));
    let result = ast.render(vals, &SETTINGS);
    assert!(result.is_err(), "Nullable int should reject string value");
}

/// Test nullable unsigned decimal accepts null value
#[test]
fn test_nullable_unsigned_decimal_accepts_null() {
    let sql = "SELECT * FROM products WHERE price = :price!nud";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("price".into(), ParameterValue::Null);
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Nullable unsigned decimal should accept null value"
    );
}

/// Test nullable unsigned decimal accepts positive float
#[test]
fn test_nullable_unsigned_decimal_accepts_positive_float() {
    let sql = "SELECT * FROM products WHERE price = :price!nud";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("price".into(), ParameterValue::Float(19.99));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_ok(),
        "Nullable unsigned decimal should accept positive float"
    );
}

/// Test nullable unsigned decimal rejects negative float
#[test]
fn test_nullable_unsigned_decimal_rejects_negative() {
    let sql = "SELECT * FROM products WHERE price = :price!nud";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("price".into(), ParameterValue::Float(-5.0));
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_err(),
        "Nullable unsigned decimal should reject negative value"
    );
}

/// Test nullable mixed accepts any value including null
#[test]
fn test_nullable_mixed_accepts_any() {
    let sql = "SELECT * FROM users WHERE data = :data!n";
    let ast = into_ast(sql);

    // Test null
    let mut vals = ParamsMap::default();
    vals.insert("data".into(), ParameterValue::Null);
    assert!(
        ast.render(vals, &SETTINGS).is_ok(),
        "Nullable mixed should accept null"
    );

    // Test int
    let mut vals = ParamsMap::default();
    vals.insert("data".into(), ParameterValue::Int(42));
    assert!(
        ast.render(vals, &SETTINGS).is_ok(),
        "Nullable mixed should accept int"
    );

    // Test string
    let mut vals = ParamsMap::default();
    vals.insert("data".into(), ParameterValue::String("test".into()));
    assert!(
        ast.render(vals, &SETTINGS).is_ok(),
        "Nullable mixed should accept string"
    );
}

/// Test that untyped placeholders reject null (non-nullable by default)
#[test]
fn test_untyped_placeholder_rejects_null() {
    let sql = "SELECT * FROM users WHERE data = :data";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("data".into(), ParameterValue::Null);
    let result = ast.render(vals, &SETTINGS);
    assert!(
        result.is_err(),
        "Untyped placeholder should reject null (non-nullable by default)"
    );
}

// ============================================================================
// Conditional Block + Nullable Tests
// ============================================================================

/// Test absent nullable placeholder in conditional block causes block to skip
#[test]
fn test_nullable_in_conditional_block_absent_skips() {
    let sql = "SELECT * FROM users WHERE 1=1 {{ AND status = :status!ni }}";
    let ast = into_ast(sql);
    let vals = ParamsMap::default(); // status not provided
    let (query, _) = ast
        .render(vals, &SETTINGS)
        .expect("Rendering should succeed");
    assert!(
        !query.contains("status"),
        "Block should be skipped when nullable placeholder is absent"
    );
}

/// Test null value for nullable placeholder in conditional block renders the block
#[test]
fn test_nullable_in_conditional_block_null_renders() {
    let sql = "SELECT * FROM users WHERE 1=1 {{ AND status = :status!ni }}";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("status".into(), ParameterValue::Null);
    let (query, _) = ast
        .render(vals, &SETTINGS)
        .expect("Rendering should succeed");
    assert!(
        query.contains("status"),
        "Block should render when nullable placeholder has null value"
    );
}

/// Test non-null value for nullable placeholder in conditional block renders the block
#[test]
fn test_nullable_in_conditional_block_value_renders() {
    let sql = "SELECT * FROM users WHERE 1=1 {{ AND status = :status!ni }}";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("status".into(), ParameterValue::Int(1));
    let (query, params) = ast
        .render(vals, &SETTINGS)
        .expect("Rendering should succeed");
    assert!(
        query.contains("status"),
        "Block should render when nullable placeholder has a value"
    );
    assert_eq!(params, vec![ParameterValue::Int(1)]);
}

/// Test non-nullable placeholder in conditional block: null skips the block
#[test]
fn test_non_nullable_in_conditional_block_null_skips() {
    let sql = "SELECT * FROM users WHERE 1=1 {{ AND status = :status!i }}";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("status".into(), ParameterValue::Null);
    let (query, _) = ast
        .render(vals, &SETTINGS)
        .expect("Rendering should succeed");
    assert!(
        !query.contains("status"),
        "Block should be skipped when non-nullable placeholder has null value"
    );
}

/// Test untyped placeholder in conditional block: null skips the block (backwards compatible)
#[test]
fn test_untyped_in_conditional_block_null_skips() {
    let sql = "SELECT * FROM users WHERE 1=1 {{ AND status = :status }}";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("status".into(), ParameterValue::Null);
    let (query, _) = ast
        .render(vals, &SETTINGS)
        .expect("Rendering should succeed");
    assert!(
        !query.contains("status"),
        "Block should be skipped when untyped placeholder has null value (backwards compat)"
    );
}

/// Test nullable mixed (?n) in conditional block with null value renders the block
#[test]
fn test_nullable_mixed_in_conditional_block_null_renders() {
    let sql = "SELECT * FROM users WHERE 1=1 {{ AND data = :data!n }}";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("data".into(), ParameterValue::Null);
    let (query, _) = ast
        .render(vals, &SETTINGS)
        .expect("Rendering should succeed");
    assert!(
        query.contains("data"),
        "Block should render when nullable mixed placeholder has null value"
    );
}

/// Test positional nullable placeholder in conditional block
#[test]
fn test_positional_nullable_in_conditional_block() {
    let sql = "SELECT * FROM users WHERE 1=1 {{ AND status = ?ni }}";
    let ast = into_ast(sql);

    // Test with null - should render
    let mut vals = ParamsMap::default();
    vals.insert("0".into(), ParameterValue::Null);
    let (query, _) = ast
        .render(vals, &SETTINGS)
        .expect("Rendering should succeed");
    assert!(
        query.contains("status"),
        "Block should render for positional nullable with null"
    );

    // Test with value - should render
    let mut vals = ParamsMap::default();
    vals.insert("0".into(), ParameterValue::Int(5));
    let (query, params) = ast
        .render(vals, &SETTINGS)
        .expect("Rendering should succeed");
    assert!(
        query.contains("status"),
        "Block should render for positional nullable with value"
    );
    assert_eq!(params, vec![ParameterValue::Int(5)]);
}
