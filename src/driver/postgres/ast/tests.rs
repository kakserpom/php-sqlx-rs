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
    use crate::OrderBy;

    let ob = OrderBy::new([
        ("name", "users.name"),
        ("age", "users.age"),
        ("posts", "COUNT(posts.id)"),
    ]);

    let rendered = ob.internal_apply(vec![
        OrderFieldDefinition::Short("name".into()),
        OrderFieldDefinition::Full(vec!["posts".into(), OrderBy::_DESC.into()]),
    ]);

    let sql = "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id ORDER BY $order_by";
    let ast = PgAst::parse(sql).unwrap();
    let (query, params) = ast
        .render([("order_by", PgParameterValue::RenderedOrderBy(rendered))])
        .expect("Rendering failed");

    assert_eq!(
        query,
        "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id ORDER BY users.name ASC, COUNT(posts.id) DESC"
    );
    assert_eq!(params, vec![]);
}

#[test]
fn test_render_order_by_apply_empty() {
    use crate::orderby::OrderBy;

    let ob = OrderBy::new([
        ("name", "users.name"),
        ("age", "users.age"),
        ("posts", "COUNT(posts.id)"),
    ]);

    let rendered = ob.internal_apply(vec![]);

    let sql =
        "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id {{ ORDER BY $order_by }}";
    let ast = PgAst::parse(sql).unwrap();
    let (query, params) = ast
        .render([("order_by", PgParameterValue::RenderedOrderBy(rendered))])
        .expect("Rendering failed");

    assert_eq!(
        query,
        "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id"
    );
    assert_eq!(params, vec![]);
}
