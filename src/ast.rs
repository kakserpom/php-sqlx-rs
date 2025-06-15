use crate::RenderedOrderBy;
use anyhow::bail;
use ext_php_rs::ZvalConvert;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Write};

#[derive(Debug, Clone)]
pub enum Ast {
    Nested(Vec<Ast>),
    /// Literal SQL text
    Sql(String),
    /// Placeholder like `$id`, `:param`, positional `?` replaced with ordinal number
    Placeholder(String),
    /// Optional segment with its own nested branches and collected placeholders
    ConditionalBlock {
        branches: Vec<Ast>,
        required_placeholders: Vec<Placeholder>,
    },
    Root {
        branches: Vec<Ast>,
        required_placeholders: Vec<Placeholder>,
    },
}

/// Represents a placeholder identifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placeholder(pub String);

/// Supported parameter types
#[derive(ZvalConvert, Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    RenderedOrderBy(RenderedOrderBy),
}
impl Value {
    pub fn is_empty(&self) -> bool {
        match self {
            Value::RenderedOrderBy(rendered_order_by) => rendered_order_by.is_empty(),
            Value::Array(array) => array.is_empty(),
            Value::Str(_) | Value::Int(_) | Value::Float(_) | Value::Bool(_) | Value::Object(_) => {
                false
            }
        }
    }
}
pub type ParamsMap = BTreeMap<String, Value>;
impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Str(s.to_string())
    }
}
impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Str(s)
    }
}

impl From<i64> for Value {
    fn from(s: i64) -> Self {
        Value::Int(s)
    }
}

impl From<bool> for Value {
    fn from(s: bool) -> Self {
        Value::Bool(s)
    }
}

/// Parses an input SQL query containing optional blocks `{{ ... }}`, placeholders `$...`, `:param`, `?`,
/// but ignores them inside string literals and comments, with support for escaping via `\\`.
/// Returns an `AST::Nested` of top-level branches.
impl Ast {
    pub fn parse(input: &str) -> Result<Ast, String> {
        #[derive(Debug, PartialEq)]
        enum Mode {
            Normal,
            InString(char),
            LineComment,
            BlockComment,
        }

        fn inner(
            chars: &[char],
            pos: &mut usize,
            placeholders_out: &mut Vec<Placeholder>,
            branches: &mut Vec<Ast>,
            positional_counter: &mut usize,
        ) -> Result<(), String> {
            let mut buf = String::new();
            let mut mode = Mode::Normal;
            while *pos < chars.len() {
                let c = chars[*pos];
                let next = chars.get(*pos + 1).copied();
                match mode {
                    Mode::Normal => {
                        // Optional block start
                        if c == '{' && next == Some('{') {
                            if !buf.is_empty() {
                                branches.push(Ast::Sql(buf.clone()));
                                buf.clear();
                            }
                            *pos += 2;
                            let mut inner_br = Vec::new();
                            let mut inner_ph = Vec::new();
                            inner(chars, pos, &mut inner_ph, &mut inner_br, positional_counter)?;
                            branches.push(Ast::ConditionalBlock {
                                branches: inner_br,
                                required_placeholders: inner_ph,
                            });
                            continue;
                        }
                        // Optional block end
                        if c == '}' && next == Some('}') {
                            *pos += 2;
                            break;
                        }
                        // Dollar placeholder
                        if c == '$' {
                            if next == Some('$') {
                                buf.push_str("$$");
                                *pos += 2;
                                continue;
                            }

                            if !buf.is_empty() {
                                branches.push(Ast::Sql(buf.clone()));
                                buf.clear();
                            }
                            *pos += 1;
                            let start = *pos;
                            while *pos < chars.len()
                                && (chars[*pos].is_alphanumeric() || chars[*pos] == '_')
                            {
                                *pos += 1;
                            }
                            let name: String = chars[start..*pos].iter().collect();
                            placeholders_out.push(Placeholder(name.clone()));
                            branches.push(Ast::Placeholder(name));
                            continue;
                        }
                        // Named placeholder
                        if c == ':' {
                            if next == Some(':') {
                                buf.push_str("::");
                                *pos += 2;
                                continue;
                            }
                            if let Some(nc) = next {
                                if nc.is_alphanumeric() || nc == '_' {
                                    if !buf.is_empty() {
                                        branches.push(Ast::Sql(buf.clone()));
                                        buf.clear();
                                    }
                                    *pos += 1;
                                    let start = *pos;
                                    while *pos < chars.len()
                                        && (chars[*pos].is_alphanumeric() || chars[*pos] == '_')
                                    {
                                        *pos += 1;
                                    }
                                    let name: String = chars[start..*pos].iter().collect();
                                    placeholders_out.push(Placeholder(name.clone()));
                                    branches.push(Ast::Placeholder(name));
                                    continue;
                                }
                            }
                        }
                        // Positional placeholder
                        if c == '?' {
                            if !buf.is_empty() {
                                branches.push(Ast::Sql(buf.clone()));
                                buf.clear();
                            }
                            *pos += 1;
                            *positional_counter += 1;
                            let idx = positional_counter.to_string();
                            placeholders_out.push(Placeholder(idx.clone()));
                            branches.push(Ast::Placeholder(idx));
                            continue;
                        }
                        // Enter string literal
                        if c == '\'' || c == '"' {
                            buf.push(c);
                            mode = Mode::InString(c);
                            *pos += 1;
                            continue;
                        }
                        // Enter line comment
                        if c == '-' && next == Some('-') {
                            buf.push('-');
                            buf.push('-');
                            mode = Mode::LineComment;
                            *pos += 2;
                            continue;
                        }
                        // Enter block comment
                        if c == '/' && next == Some('*') {
                            buf.push('/');
                            buf.push('*');
                            mode = Mode::BlockComment;
                            *pos += 2;
                            continue;
                        }
                        buf.push(c);
                        *pos += 1;
                    }
                    Mode::InString(q) => {
                        // Escape inside string
                        if c == '\\' && next.is_some() {
                            buf.push('\\');
                            *pos += 1;
                            buf.push(chars[*pos]);
                            *pos += 1;
                            continue;
                        }
                        buf.push(c);
                        *pos += 1;
                        if c == q {
                            if next == Some(q) {
                                buf.push(q);
                                *pos += 1;
                            } else {
                                mode = Mode::Normal;
                            }
                        }
                    }
                    Mode::LineComment => {
                        buf.push(c);
                        *pos += 1;
                        if c == '\n' {
                            mode = Mode::Normal;
                        }
                    }
                    Mode::BlockComment => {
                        if c == '*' && next == Some('/') {
                            buf.push('*');
                            buf.push('/');
                            *pos += 2;
                            mode = Mode::Normal;
                        } else {
                            buf.push(c);
                            *pos += 1;
                        }
                    }
                }
            }
            if !buf.is_empty() {
                branches.push(Ast::Sql(buf));
            }
            Ok(())
        }
        let chars: Vec<char> = input.chars().collect();
        let mut pos = 0;
        let mut pc = 0;
        let mut branches = Vec::new();
        let mut placeholders = Vec::new();
        inner(&chars, &mut pos, &mut placeholders, &mut branches, &mut pc)?;
        if pos < chars.len() {
            return Err("Unmatched optional block `{{ }}`".into());
        }
        Ok(Ast::Root {
            branches,
            required_placeholders: placeholders,
        })
    }

    /// Renders the AST into a SQL string with numbered placeholders like `$1`, `$2`, ...
    /// `values` can be any iterable of (key, value) pairs. Keys convertible to String; values convertible to Value.
    pub fn render<I, K, V>(&self, values: I) -> anyhow::Result<(String, Vec<Value>)>
    where
        I: IntoIterator<Item = (K, V)> + Debug,
        K: Into<String>,
        V: Into<Value>,
    {
        #[cfg(test)]
        {
            println!("AST = {:?}", self);
            println!("VALUES = {:?}", values);
        }
        fn walk(
            node: &Ast,
            values: &ParamsMap,
            sql: &mut String,
            out_vals: &mut Vec<Value>,
            counter: &mut usize,
        ) {
            match node {
                Ast::Root { branches, .. } | Ast::Nested(branches) => {
                    for n in branches {
                        walk(n, values, sql, out_vals, counter);
                    }
                }
                Ast::Sql(s) => sql.push_str(s),
                Ast::Placeholder(name) => {
                    #[cfg(test)]
                    {
                        println!("values = {values:?}");
                        println!("{name:?} ==> {:?}", values.get(name));
                    }
                    if let Some(val) = values.get(name) {
                        match val {
                            Value::RenderedOrderBy(order_by) => {
                                for (i, item) in order_by.__inner.iter().enumerate() {
                                    if i > 0 {
                                        sql.push_str(", ");
                                    }
                                    sql.push_str(item);
                                }
                            }
                            Value::Array(arr) => {
                                for (i, item) in arr.iter().enumerate() {
                                    *counter += 1;
                                    if i > 0 {
                                        sql.push_str(", ");
                                    }
                                    write!(sql, "${}", *counter).unwrap();
                                    out_vals.push(item.clone());
                                }
                            }
                            _ => {
                                *counter += 1;
                                write!(sql, "${}", *counter).unwrap();
                                out_vals.push(val.clone());
                            }
                        }
                    }
                }
                Ast::ConditionalBlock {
                    branches,
                    required_placeholders,
                } => {
                    if required_placeholders.iter().all(|ph| {
                        if let Some(value) = values.get(&ph.0) {
                            !value.is_empty()
                        } else {
                            false
                        }
                    }) {
                        for b in branches {
                            walk(b, values, sql, out_vals, counter);
                        }
                    }
                }
            }
        }

        let values: ParamsMap = values
            .into_iter()
            .map(|(k, v)| {
                let mut k = k.into();
                if let Ok(n) = k.parse::<u32>() {
                    k = n.saturating_add(1).to_string();
                }
                (k, v.into())
            })
            .collect();

        let mut sql = String::new();
        let mut out_vals = Vec::new();
        let mut counter = 0usize;

        if let Ast::Root {
            required_placeholders,
            ..
        } = self
        {
            if let Some(missing_placeholder) = required_placeholders.iter().find(|ph| {
                if let Some(value) = values.get(&ph.0) {
                    value.is_empty()
                } else {
                    true
                }
            }) {
                bail!("Missing required placeholder `{}`", missing_placeholder.0);
            }
        }
        walk(self, &values, &mut sql, &mut out_vals, &mut counter);
        let sql = sql.split_whitespace().collect::<Vec<_>>().join(" ");
        Ok((sql, out_vals))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OrderFieldDefinition;

    #[test]
    fn test_named_and_positional() {
        let sql = "SELECT :param, ?, ? FROM table WHERE {{ x = $x }}";
        if let Ast::Root {
            branches,
            required_placeholders,
        } = Ast::parse(sql).unwrap()
        {
            println!("{:#?}", required_placeholders);
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
        let ast = Ast::parse(sql).unwrap();
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
        let ast = Ast::parse(sql).unwrap();
        let (query, params) = ast.render([("id", 100)]).expect("Rendering failed");
        assert_eq!(query, "SELECT * FROM users WHERE id = $1");
        assert_eq!(params, vec![100.into()]);
    }

    #[test]
    fn test_render_var_types() {
        let sql = "SELECT * FROM table WHERE id = $id AND active = :flag AND scores IN (?) AND data = $data";
        let ast = Ast::parse(sql).unwrap();
        let mut vals = ParamsMap::new();
        vals.insert("id".into(), Value::Int(7));
        vals.insert("flag".into(), Value::Bool(true));
        vals.insert("0".into(), Value::Array(vec![Value::Int(1), Value::Int(2)]));
        vals.insert("data".into(), Value::Str("xyz".into()));
        let (q, params) = ast.render(vals).expect("Rendering failed");
        assert_eq!(
            q,
            "SELECT * FROM table WHERE id = $1 AND active = $2 AND scores IN ($3, $4) AND data = $5"
        );
        assert_eq!(
            params,
            vec![
                Value::Int(7),
                Value::Bool(true),
                Value::Int(1),
                Value::Int(2),
                Value::Str("xyz".into()),
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

        let rendered = ob._apply(vec![
            OrderFieldDefinition::Short("name".into()),
            OrderFieldDefinition::Full(vec!["posts".into(), OrderBy::_DESC.into()]),
        ]);

        let sql =
            "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id ORDER BY $order_by";
        let ast = Ast::parse(sql).unwrap();
        let (query, params) = ast
            .render([("order_by", Value::RenderedOrderBy(rendered))])
            .expect("Rendering failed");

        assert_eq!(
            query,
            "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id ORDER BY users.name ASC, COUNT(posts.id) DESC"
        );
        assert_eq!(params, vec![]);
    }

    #[test]
    fn test_render_order_by_apply_empty() {
        use crate::OrderBy;

        let ob = OrderBy::new([
            ("name", "users.name"),
            ("age", "users.age"),
            ("posts", "COUNT(posts.id)"),
        ]);

        let rendered = ob._apply(vec![]);

        let sql = "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id {{ ORDER BY $order_by }}";
        let ast = Ast::parse(sql).unwrap();
        let (query, params) = ast
            .render([("order_by", Value::RenderedOrderBy(rendered))])
            .expect("Rendering failed");

        assert_eq!(
            query,
            "SELECT * FROM users LEFT JOIN posts ON posts.user_id = users.id"
        );
        assert_eq!(params, vec![]);
    }
}
