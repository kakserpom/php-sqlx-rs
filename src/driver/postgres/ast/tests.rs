use super::*;
use crate::OrderFieldDefinition;

#[test]
fn test_named_and_positional() {
    let sql = "SELECT :param, ?, ? FROM table WHERE {{ x = $x }}";
    if let PgAst::Root {
        branches,
        required_placeholders,
    } = PgAst::parse(sql).unwrap()
    {
        println!("{:#?}", required_placeholders);
        let names: Vec<&str> = branches
            .iter()
            .filter_map(|b| {
                if let PgAst::Placeholder(n) = b {
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
    let ast = PgAst::parse(sql).unwrap();
    let mut vals = ParamsMap::default();
    vals.insert("status".into(), "active".into());
    vals.insert("id".into(), "42".into());
    let (query, params) = ast.render(vals).expect("Rendering failed");
    assert_eq!(query, "SELECT * FROM users WHERE status = $1 AND id = $2");
    assert_eq!(params, vec!["active".into(), "42".into()]);
}

#[test]
fn test_render_optional_skip() {
    let sql = "SELECT * FROM users WHERE {{status = $status AND}} id = $id";
    let ast = PgAst::parse(sql).unwrap();
    let (query, params) = ast.render([("id", 100)]).expect("Rendering failed");
    assert_eq!(query, "SELECT * FROM users WHERE id = $1");
    assert_eq!(params, vec![100.into()]);
}

#[test]
fn test_render_var_types() {
    let sql =
        "SELECT * FROM table WHERE id = $id AND active = :flag AND scores IN (?) AND data = $data";
    let ast = PgAst::parse(sql).unwrap();
    let mut vals = ParamsMap::new();
    vals.insert("id".into(), PgParameterValue::Int(7));
    vals.insert("flag".into(), PgParameterValue::Bool(true));
    vals.insert(
        "0".into(),
        PgParameterValue::Array(vec![PgParameterValue::Int(1), PgParameterValue::Int(2)]),
    );
    vals.insert("data".into(), PgParameterValue::Str("xyz".into()));
    let (q, params) = ast.render(vals).expect("Rendering failed");
    assert_eq!(
        q,
        "SELECT * FROM table WHERE id = $1 AND active = $2 AND scores IN ($3, $4) AND data = $5"
    );
    assert_eq!(
        params,
        vec![
            PgParameterValue::Int(7),
            PgParameterValue::Bool(true),
            PgParameterValue::Int(1),
            PgParameterValue::Int(2),
            PgParameterValue::Str("xyz".into()),
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
    let ast = PgAst::parse(sql).unwrap();
    let (query, params) = ast
        .render([("order_by", PgParameterValue::RenderedByClause(rendered))])
        .expect("Rendering failed");

    assert_eq!(
        query,
        "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id ORDER BY users.name, COUNT(posts.id) DESC"
    );
    assert_eq!(params, vec![]);
}

#[test]
fn test_render_order_by_apply_empty() {
    use crate::byclause::ByClause;

    let ob = ByClause::new([
        ("name", "users.name"),
        ("age", "users.age"),
        ("posts", "COUNT(posts.id)"),
    ])
    .unwrap();

    let rendered = ob.internal_apply(vec![]);

    let sql =
        "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id {{ ORDER BY $order_by }}";
    let ast = PgAst::parse(sql).unwrap();
    let (query, params) = ast
        .render([("order_by", PgParameterValue::RenderedByClause(rendered))])
        .expect("Rendering failed");

    assert_eq!(
        query,
        "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id"
    );
    assert_eq!(params, vec![]);
}

#[test]
fn test_in_clause_parsing() {
    let sql = "SELECT * FROM users WHERE status IN :statuses AND age NOT IN (:ages)";
    let ast = PgAst::parse(sql).unwrap();
    println!("AST = {:#?}", ast);
    let (q, p) = ast
        .render([
            (
                "statuses",
                PgParameterValue::Array(vec![
                    PgParameterValue::Int(1),
                    PgParameterValue::Int(2),
                    PgParameterValue::Int(3),
                ]),
            ),
            (
                "ages",
                PgParameterValue::Array(vec![PgParameterValue::Int(20), PgParameterValue::Int(30)]),
            ),
        ])
        .unwrap();

    assert_eq!(
        q,
        "SELECT * FROM users WHERE status IN ($1, $2, $3) AND age NOT IN ($4, $5)"
    );
    assert_eq!(
        p,
        vec![
            PgParameterValue::from(1),
            PgParameterValue::from(2),
            PgParameterValue::from(3),
            PgParameterValue::from(20),
            PgParameterValue::from(30),
        ]
    );
}

#[test]
fn test_parse_in_not_in_and_string() {
    let sql =
        "SELECT * FROM users WHERE name = 'O''Reilly' AND status IN (:statuses) AND age NOT IN (:ages)";
    let ast = PgAst::parse(sql).expect("Failed to parse");
    println!("AST = {:#?}", ast);
    if let PgAst::Root {
        branches,
        required_placeholders,
    } = ast
    {
        // Expect placeholders
        assert!(required_placeholders.is_empty());
        // Expect branch count
        //assert_eq!(branches.len(), 5);
        // Check sequence of branches
        match &branches[0] {
            PgAst::Sql(s) => assert!(s.ends_with("name = 'O''Reilly' AND ")),
            _ => panic!("Expected Sql at branch 0"),
        }
        match &branches[1] {
            PgAst::InClause { expr, placeholder } => {
                assert_eq!(expr, "status");
                assert_eq!(placeholder, "statuses");
            }
            _ => panic!("Expected InClause at branch 1"),
        }
        match &branches[2] {
            PgAst::Sql(s) => assert_eq!(s, " AND "),
            _ => panic!("Expected Sql at branch 2"),
        }
        match &branches[3] {
            PgAst::NotInClause { expr, placeholder } => {
                assert_eq!(expr, "age");
                assert_eq!(placeholder, "ages");
            }
            _ => panic!("Expected NotInClause at branch 3"),
        }
        assert_eq!(branches.len(), 4);
    } else {
        panic!("AST root is not Root variant");
    }
}
