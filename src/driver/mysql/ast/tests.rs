use super::*;
use crate::OrderFieldDefinition;

fn into_ast(sql: &str) -> MySqlAst {
    MySqlAst::parse(sql, true).expect("failed to parse SQL statement")
}
#[test]
fn test_named_and_positional() {
    if let MySqlAst::Root {
        branches,
        required_placeholders,
    } = into_ast("SELECT :param, ?, ? FROM table WHERE {{ x = $x }}")
    {
        println!("{:#?}", required_placeholders);
        let names: Vec<&str> = branches
            .iter()
            .filter_map(|b| {
                if let MySqlAst::Placeholder(n) = b {
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
    let (query, params) = ast.render(vals).expect("Rendering failed");
    assert_eq!(query, "SELECT * FROM users WHERE status = ? AND id = ?");
    assert_eq!(params, vec!["active".into(), "42".into()]);
}

#[test]
fn test_render_optional_skip() {
    let sql = "SELECT * FROM users WHERE {{status = $status AND}} id = $id";
    let ast = into_ast(sql);
    let (query, params) = ast.render([("id", 100)]).expect("Rendering failed");
    assert_eq!(query, "SELECT * FROM users WHERE id = ?");
    assert_eq!(params, vec![100.into()]);
}

#[test]
fn test_render_var_types() {
    let sql =
        "SELECT * FROM table WHERE id = $id AND active = :flag AND scores IN (?) AND data = $data";
    let ast = into_ast(sql);
    println!("{:#?}", ast);
    let mut vals = ParamsMap::new();
    vals.insert("id".into(), MySqlParameterValue::Int(7));
    vals.insert("flag".into(), MySqlParameterValue::Bool(true));
    vals.insert(
        "0".into(),
        MySqlParameterValue::Array(vec![
            MySqlParameterValue::Int(1),
            MySqlParameterValue::Int(2),
        ]),
    );
    vals.insert("data".into(), MySqlParameterValue::Str("xyz".into()));
    let (q, params) = ast.render(vals).expect("Rendering failed");
    assert_eq!(
        q,
        "SELECT * FROM table WHERE id = ? AND active = ? AND scores IN (?, ?) AND data = ?"
    );
    assert_eq!(
        params,
        vec![
            MySqlParameterValue::Int(7),
            MySqlParameterValue::Bool(true),
            MySqlParameterValue::Int(1),
            MySqlParameterValue::Int(2),
            MySqlParameterValue::Str("xyz".into()),
        ]
    );
}

#[test]
fn test_render_order_by_apply() {
    use crate::ByClause;

    let ob = ByClause::new([
        ("name", "users.name"),
        ("age", "users.age"),
        ("posts", "COUNT(posts.id)"),
    ])
    .unwrap();

    let rendered = ob.internal_apply(vec![
        OrderFieldDefinition::Short("name".into()),
        OrderFieldDefinition::Full(vec!["posts".into(), ByClause::_DESC.into()]),
    ]);

    let sql = "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id ORDER BY $order_by";
    let ast = into_ast(sql);
    let (query, params) = ast
        .render([("order_by", MySqlParameterValue::RenderedByClause(rendered))])
        .expect("Rendering failed");

    assert_eq!(
        query,
        "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id ORDER BY users.name, COUNT(posts.id) DESC"
    );
    assert_eq!(params, vec![]);
}

#[test]
fn test_render_order_by_apply_empty() {
    use crate::ByClause;

    let ob = ByClause::new([
        ("name", "users.name"),
        ("age", "users.age"),
        ("posts", "COUNT(posts.id)"),
    ])
    .unwrap();

    let rendered = ob.internal_apply(vec![]);

    let sql =
        "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id {{ ORDER BY $order_by }}";
    let ast = into_ast(sql);
    let (query, params) = ast
        .render([("order_by", MySqlParameterValue::RenderedByClause(rendered))])
        .expect("Rendering failed");

    assert_eq!(
        query,
        "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id"
    );
    assert_eq!(params, vec![]);
}

#[test]
fn test_parse_in_not_in_and_string() {
    let ast = into_ast(
        "SELECT * FROM users WHERE name = 'O''Reilly' AND status IN (:statuses) AND age NOT IN (:ages)",
    );
    let MySqlAst::Root {
        branches,
        required_placeholders,
    } = ast
    else {
        panic!("Expected Root variant for MySqlAst");
    };
    assert!(required_placeholders.is_empty());
    // 0: Sql up to AND before IN
    match &branches[0] {
        MySqlAst::Sql(s) => assert!(s.ends_with("name = 'O''Reilly' AND ")),
        _ => panic!("Expected Sql at branch 0"),
    }
    // 1: InClause
    match &branches[1] {
        MySqlAst::InClause { expr, placeholder } => {
            assert_eq!(expr, "status");
            assert_eq!(placeholder, "statuses");
        }
        _ => panic!("Expected InClause at branch 1"),
    }
    // 2: Sql between clauses
    match &branches[2] {
        MySqlAst::Sql(s) => assert_eq!(s, " AND "),
        _ => panic!("Expected Sql at branch 2"),
    }
    // 3: NotInClause
    match &branches[3] {
        MySqlAst::NotInClause { expr, placeholder } => {
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
    println!("AST = {:#?}", ast);
    let MySqlAst::Root {
        required_placeholders,
        ..
    } = ast
    else {
        panic!("Expected Root variant for MySqlAst");
    };
    assert_eq!(required_placeholders.len(), 2);
}
