#[cfg(test)]
mod tests;

use crate::RenderedOrderBy;
use anyhow::bail;
use ext_php_rs::ZvalConvert;
use itertools::Itertools;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Write};

#[derive(Debug, Clone)]
pub enum PgAst {
    Nested(Vec<PgAst>),
    /// Literal SQL text
    Sql(String),
    /// Placeholder like `$id`, `:param`, positional `?` replaced with ordinal number
    Placeholder(String),
    /// Optional segment with its own nested branches and collected placeholders
    ConditionalBlock {
        branches: Vec<PgAst>,
        required_placeholders: Vec<Placeholder>,
    },
    Root {
        branches: Vec<PgAst>,
        required_placeholders: Vec<Placeholder>,
    },
}

/// Represents a placeholder identifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placeholder(pub String);

/// Supported parameter types
#[derive(ZvalConvert, Debug, Clone, PartialEq)]
pub enum PgParameterValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<PgParameterValue>),
    Object(HashMap<String, PgParameterValue>),
    RenderedOrderBy(RenderedOrderBy),
}
impl PgParameterValue {
    pub fn is_empty(&self) -> bool {
        match self {
            PgParameterValue::RenderedOrderBy(rendered_order_by) => rendered_order_by.is_empty(),
            PgParameterValue::Array(array) => array.is_empty(),
            PgParameterValue::Str(_)
            | PgParameterValue::Int(_)
            | PgParameterValue::Float(_)
            | PgParameterValue::Bool(_)
            | PgParameterValue::Object(_) => false,
        }
    }
}
pub type ParamsMap = BTreeMap<String, PgParameterValue>;
impl From<&str> for PgParameterValue {
    fn from(s: &str) -> Self {
        PgParameterValue::Str(s.to_string())
    }
}
impl From<String> for PgParameterValue {
    fn from(s: String) -> Self {
        PgParameterValue::Str(s)
    }
}

impl From<i64> for PgParameterValue {
    fn from(s: i64) -> Self {
        PgParameterValue::Int(s)
    }
}

impl From<bool> for PgParameterValue {
    fn from(s: bool) -> Self {
        PgParameterValue::Bool(s)
    }
}

/// Parses an input SQL query containing optional blocks `{{ ... }}`, placeholders `$...`, `:param`, `?`,
/// but ignores them inside string literals and comments, with support for escaping via `\\`.
/// Returns an `AST::Nested` of top-level branches.
impl PgAst {
    pub fn parse(input: &str) -> Result<PgAst, String> {
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
            branches: &mut Vec<PgAst>,
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
                                branches.push(PgAst::Sql(buf.clone()));
                                buf.clear();
                            }
                            *pos += 2;
                            let mut inner_br = Vec::new();
                            let mut inner_ph = Vec::new();
                            inner(chars, pos, &mut inner_ph, &mut inner_br, positional_counter)?;
                            branches.push(PgAst::ConditionalBlock {
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
                                branches.push(PgAst::Sql(buf.clone()));
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
                            branches.push(PgAst::Placeholder(name));
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
                                        branches.push(PgAst::Sql(buf.clone()));
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
                                    branches.push(PgAst::Placeholder(name));
                                    continue;
                                }
                            }
                        }
                        // Positional placeholder
                        if c == '?' {
                            if !buf.is_empty() {
                                branches.push(PgAst::Sql(buf.clone()));
                                buf.clear();
                            }
                            *pos += 1;
                            *positional_counter += 1;
                            let idx = positional_counter.to_string();
                            placeholders_out.push(Placeholder(idx.clone()));
                            branches.push(PgAst::Placeholder(idx));
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
                branches.push(PgAst::Sql(buf));
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
        Ok(PgAst::Root {
            branches,
            required_placeholders: placeholders,
        })
    }

    /// Renders the AST into an SQL string with numbered placeholders like `$1`, `$2`, ...
    /// `values` can be any iterable of (key, value) pairs. Keys convertible to String; values convertible to Value.
    pub fn render<I, K, V>(&self, values: I) -> anyhow::Result<(String, Vec<PgParameterValue>)>
    where
        I: IntoIterator<Item = (K, V)> + Debug,
        K: Into<String>,
        V: Into<PgParameterValue>,
    {
        #[cfg(test)]
        {
            println!("AST = {:?}", self);
            println!("VALUES = {:?}", values);
        }
        fn walk(
            node: &PgAst,
            values: &ParamsMap,
            sql: &mut String,
            out_vals: &mut Vec<PgParameterValue>,
        ) {
            match node {
                PgAst::Root { branches, .. } | PgAst::Nested(branches) => {
                    for n in branches {
                        walk(n, values, sql, out_vals);
                    }
                }
                PgAst::Sql(s) => sql.push_str(s),
                PgAst::Placeholder(name) => {
                    #[cfg(test)]
                    {
                        println!("values = {values:?}");
                        println!("{name:?} ==> {:?}", values.get(name));
                    }
                    if let Some(val) = values.get(name) {
                        match val {
                            PgParameterValue::RenderedOrderBy(order_by) => {
                                for (i, item) in order_by.__inner.iter().enumerate() {
                                    if i > 0 {
                                        sql.push_str(", ");
                                    }
                                    sql.push_str(item);
                                }
                            }
                            PgParameterValue::Array(arr) => {
                                for (i, item) in arr.iter().enumerate() {
                                    if i > 0 {
                                        sql.push_str(", ");
                                    }
                                    out_vals.push(item.clone());
                                    write!(sql, "${}", out_vals.len()).unwrap();
                                }
                            }
                            _ => {
                                out_vals.push(val.clone());
                                write!(sql, "${}", out_vals.len()).unwrap();
                            }
                        }
                    }
                }
                PgAst::ConditionalBlock {
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
                            walk(b, values, sql, out_vals);
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

        if let PgAst::Root {
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
        walk(self, &values, &mut sql, &mut out_vals);
        let sql = sql.split_whitespace().join(" ");
        #[cfg(test)]
        println!("SQL = {sql}");
        Ok((sql, out_vals))
    }
}
