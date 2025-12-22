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
                if let Ast::Placeholder(n) = b {
                    Some(n.as_str())
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
        Ast::InClause { expr, placeholder } => {
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
        Ast::NotInClause { expr, placeholder } => {
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

/// Test that null values are allowed for required placeholders (not treated as missing)
#[test]
fn test_null_placeholder_is_valid() {
    let sql = "SELECT * FROM users WHERE name = :name";
    let ast = into_ast(sql);
    let mut vals = ParamsMap::default();
    vals.insert("name".into(), ParameterValue::Null);
    let result = ast.render(vals, &SETTINGS);
    assert!(result.is_ok(), "Null value should be valid for required placeholder");
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
