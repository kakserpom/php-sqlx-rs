use crate::ast::Ast;
use crate::byclause::{ByClause, ByClauseColumnDefinition};
use crate::mssql::inner::SETTINGS;
use crate::paginateclause::PaginateClause;
use crate::paramvalue::{ParameterValue, ParamsMap};
use collapse::*;

fn into_ast(sql: &str) -> Ast {
    Ast::parse(sql, &SETTINGS).expect("failed to parse SQL statement")
}
#[test]
fn test_named_and_positional() {
    if let Ast::Root {
        branches,
        required_placeholders,
    } = into_ast("SELECT :param, ?, ? FROM table WHERE {{ x = $x }}")
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
    let ast = into_ast("SELECT * FROM users WHERE {{status = $status AND}} id = $id");
    let mut vals = ParamsMap::default();
    vals.insert("status".into(), "active".into());
    vals.insert("id".into(), "42".into());
    let (query, params) = ast.render(vals, &SETTINGS).expect("Rendering failed");
    collapsed_eq!(
        &query,
        "SELECT * FROM users WHERE status = @p1 AND id = @p2"
    );
    assert_eq!(params, vec!["active".into(), "42".into()]);
}

#[test]
fn test_render_optional_skip() {
    let sql = "SELECT * FROM users WHERE {{status = $status AND}} id = $id";
    let ast = into_ast(sql);
    let (query, params) = ast
        .render([("id", 100)], &SETTINGS)
        .expect("Rendering failed");
    collapsed_eq!(&query, "SELECT * FROM users WHERE id = @p1");
    assert_eq!(params, vec![100.into()]);
}

#[test]
fn test_render_var_types() {
    let sql =
        "SELECT * FROM table WHERE id = $id AND active = :flag AND scores IN (?) AND data = $data";
    let ast = into_ast(sql);
    println!("{ast:#?}");
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
        "SELECT * FROM table WHERE id = @p1 AND active = @p2 AND scores IN (@p3, @p4) AND data = @p5"
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
fn test_render_order_by_apply() {
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
fn test_parse_in_not_in_and_string() {
    let ast = into_ast(
        "SELECT * FROM users WHERE name = 'O''Reilly' AND status IN (:statuses) AND age NOT IN (:ages)",
    );
    let Ast::Root {
        branches,
        required_placeholders,
    } = ast
    else {
        panic!("Expected Root variant for Ast");
    };
    assert!(required_placeholders.is_empty());
    // 0: Sql up to AND before IN
    match &branches[0] {
        Ast::Raw(s) => assert!(s.ends_with("name = 'O''Reilly' AND ")),
        _ => panic!("Expected Sql at branch 0"),
    }
    // 1: InClause
    match &branches[1] {
        Ast::InClause { expr, placeholder } => {
            assert_eq!(expr, "status");
            assert_eq!(placeholder, "statuses");
        }
        _ => panic!("Expected InClause at branch 1"),
    }
    // 2: Sql between clauses
    match &branches[2] {
        Ast::Raw(s) => assert_eq!(s, " AND "),
        _ => panic!("Expected Sql at branch 2"),
    }
    // 3: NotInClause
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
    let ast = into_ast("SELECT * FROM users age IN (?, ?)");
    println!("AST = {ast:#?}");
    let Ast::Root {
        required_placeholders,
        ..
    } = ast
    else {
        panic!("Expected Root variant for Ast");
    };
    assert_eq!(required_placeholders.len(), 2);
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
        "SELECT * FROM users age ORDER BY id LIMIT @p1 OFFSET @p2"
    );
    assert_eq!(
        values,
        vec![ParameterValue::Int(5), ParameterValue::Int(35)]
    );
}
