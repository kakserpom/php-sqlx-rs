#[cfg(test)]
mod tests;

use crate::ByClauseRendered;
use crate::byclause::ByClauseRenderedField;
use crate::selectclause::{SelectClauseRendered, SelectClauseRenderedField};
use anyhow::bail;
use ext_php_rs::ZvalConvert;
use itertools::Itertools;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Write};
use trim_in_place::TrimInPlace;

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
    InClause {
        expr: String,
        placeholder: String,
    },
    NotInClause {
        expr: String,
        placeholder: String,
    },
}

/// Represents a placeholder identifier
//#[derive(Debug, Clone, PartialEq, Eq)]
//pub struct Placeholder(pub String);
pub type Placeholder = String;

/// Supported parameter types
#[derive(ZvalConvert, Debug, Clone, PartialEq)]
pub enum PgParameterValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<PgParameterValue>),
    Object(HashMap<String, PgParameterValue>),
    RenderedByClause(ByClauseRendered),
    RenderedSelectClause(SelectClauseRendered),
}
impl PgParameterValue {
    #[must_use]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        match self {
            PgParameterValue::RenderedByClause(x) => x.is_empty(),
            PgParameterValue::RenderedSelectClause(x) => x.is_empty(),
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
impl PgAst {
    /// Parses an input SQL query containing optional blocks `{{ ... }}`, placeholders `$...`, `:param`, `?`,
    /// but ignores them inside string literals and comments.
    /// Returns an `AST::Nested` of top-level branches.
    pub fn parse(input: &str) -> Result<PgAst, String> {
        fn inner<'s>(
            mut rest: &'s str,
            placeholders_out: &mut Vec<String>,
            branches: &mut Vec<PgAst>,
            positional_counter: &mut usize,
        ) -> Result<&'s str, String> {
            let mut buf = String::new();

            while !rest.is_empty() {
                // Handle SQL string literal '...' with '' escape
                if rest.starts_with('\'') {
                    let mut idx = 1;
                    let mut iter = rest.char_indices().skip(1).peekable();
                    while let Some((i, c)) = iter.next() {
                        if c == '\'' {
                            // escaped ''? skip second '
                            if let Some((_, '\'')) = iter.peek() {
                                iter.next();
                                continue;
                            }
                            idx = i + c.len_utf8();
                            break;
                        }
                    }
                    let literal = &rest[..idx];
                    buf.push_str(literal);
                    rest = &rest[idx..];
                    continue;
                }
                // Handle line comment -- until newline
                if let Some(r) = rest.strip_prefix("--") {
                    // include '--' and content up to newline
                    let end = r.find('\n').map(|i| i + 1).unwrap_or(r.len());
                    let comment = &rest[..2 + end];
                    buf.push_str(comment);
                    rest = &rest[2 + end..];
                    continue;
                }
                // Handle block comment /* ... */
                if let Some(r) = rest.strip_prefix("/*") {
                    if let Some(close) = r.find("*/") {
                        let comment = &rest[..2 + close + 2];
                        buf.push_str(comment);
                        rest = &rest[2 + close + 2..];
                        continue;
                    } else {
                        return Err("Unterminated block comment".into());
                    }
                }

                // Conditional block start
                if let Some(r) = rest.strip_prefix("{{") {
                    if !buf.is_empty() {
                        branches.push(PgAst::Sql(std::mem::take(&mut buf)));
                    }
                    let mut inner_branches = Vec::new();
                    let mut inner_ph = Vec::new();
                    rest = inner(r, &mut inner_ph, &mut inner_branches, positional_counter)?;
                    branches.push(PgAst::ConditionalBlock {
                        branches: inner_branches,
                        required_placeholders: inner_ph,
                    });
                    continue;
                }

                // Conditional block end
                if let Some(r) = rest.strip_prefix("}}") {
                    if !buf.is_empty() {
                        branches.push(PgAst::Sql(std::mem::take(&mut buf)));
                    }
                    return Ok(r);
                }

                // NOT IN support (with or without parentheses)
                if let Some(suffix) = rest.strip_prefix("NOT IN") {
                    let after = suffix.trim_start();
                    let offset = rest.len() - suffix.len();
                    let mut name_opt = None;
                    let mut consumed = 0;
                    // parentheses form
                    if after.starts_with('(') {
                        if let Some(cl) = after[1..].find(')') {
                            let inside = &after[1..1 + cl].trim();
                            let name = if let Some(id) = inside.strip_prefix(':') {
                                id.to_string()
                            } else if let Some(id) = inside.strip_prefix('$') {
                                id.to_string()
                            } else if *inside == "?" {
                                *positional_counter += 1;
                                positional_counter.to_string()
                            } else {
                                return Err("Invalid placeholder inside NOT IN".into());
                            };
                            consumed = offset + 1 + cl + 2;
                            name_opt = Some(name);
                        }
                    } else {
                        // non-parentheses form
                        if let Some(sfx) = after.strip_prefix(':') {
                            let ident: String = sfx
                                .chars()
                                .take_while(|c| c.is_alphanumeric() || *c == '_')
                                .collect();
                            consumed = offset + 2 + ident.len();
                            name_opt = Some(ident);
                        } else if let Some(sfx) = after.strip_prefix('$') {
                            let ident: String = sfx
                                .chars()
                                .take_while(|c| c.is_alphanumeric() || *c == '_')
                                .collect();
                            consumed = offset + 2 + ident.len();
                            name_opt = Some(ident);
                        } else if after.starts_with('?') {
                            *positional_counter += 1;
                            let num = positional_counter.to_string();
                            consumed = offset + 2;
                            name_opt = Some(num);
                        }
                    }
                    if let Some(name) = name_opt {
                        let trimmed = buf.trim_end();
                        let (pre, expr) = match trimmed.rsplit_once(char::is_whitespace) {
                            Some((a, b)) => (format!("{} ", a), b),
                            None => (String::new(), trimmed),
                        };
                        if !pre.is_empty() {
                            branches.push(PgAst::Sql(pre));
                        }
                        branches.push(PgAst::NotInClause {
                            expr: expr.to_string(),
                            placeholder: name,
                        });
                        buf.clear();
                        rest = &rest[consumed..];
                        continue;
                    }
                }

                // --- IN ... ---
                if let Some(rest_after_in) = rest.strip_prefix("IN") {
                    let rest_after_in = rest_after_in.trim_start();
                    let original_len = rest.len();

                    let (name_opt, consumed_len) =
                        if let Some(sfx) = rest_after_in.strip_prefix(':') {
                            let ident: String = sfx
                                .chars()
                                .take_while(|c| c.is_alphanumeric() || *c == '_')
                                .collect();
                            let consumed_len = original_len - rest_after_in.len() + 1 + ident.len();
                            (
                                Some(ident),
                                consumed_len,
                            )
                        } else if let Some(sfx) = rest_after_in.strip_prefix('$') {
                            let ident: String = sfx
                                .chars()
                                .take_while(|c| c.is_alphanumeric() || *c == '_')
                                .collect();
                            (
                                Some(ident.clone()),
                                original_len - rest_after_in.len() + 1 + ident.len(),
                            )
                        } else if rest_after_in.starts_with('?') {
                            *positional_counter += 1;
                            let num = positional_counter.to_string();
                            (Some(num.clone()), original_len - rest_after_in.len() + 1)
                        } else if rest_after_in.starts_with('(') {
                            if let Some(close_idx) = rest_after_in[1..].find(')') {
                                let inside = &rest_after_in[1..1 + close_idx].trim();
                                let name = if let Some(id) = inside.strip_prefix(':') {
                                    id.to_string()
                                } else if let Some(id) = inside.strip_prefix('$') {
                                    id.to_string()
                                } else if *inside == "?" {
                                    *positional_counter += 1;
                                    positional_counter.to_string()
                                } else {
                                    return Err("Invalid placeholder inside IN (...)".into());
                                };
                                (
                                    Some(name.clone()),
                                    original_len - rest_after_in.len() + 1 + close_idx + 1,
                                )
                            } else {
                                (None, 0)
                            }
                        } else {
                            (None, 0)
                        };

                    if let Some(name) = name_opt {
                        let trimmed = buf.trim_end();
                        let (pre, expr) = match trimmed.rsplit_once(char::is_whitespace) {
                            Some((a, b)) => (format!("{} ", a), b),
                            None => ("".to_string(), trimmed),
                        };
                        if !pre.is_empty() {
                            branches.push(PgAst::Sql(pre));
                        }
                        branches.push(PgAst::InClause {
                            expr: expr.to_string(),
                            placeholder: name,
                        });
                        buf.clear();
                        rest = &rest[consumed_len..];
                        continue;
                    }
                }

                // --- :named placeholder ---
                if let Some(after) = rest.strip_prefix(":") {
                    if let Some((name, rem)) = after
                        .char_indices()
                        .take_while(|(_, c)| c.is_alphanumeric() || *c == '_')
                        .map(|(i, _)| i)
                        .last()
                        .map(|i| after.split_at(i + 1))
                    {
                        if !buf.is_empty() {
                            branches.push(PgAst::Sql(std::mem::take(&mut buf)));
                        }
                        branches.push(PgAst::Placeholder(name.to_string()));
                        placeholders_out.push(name.to_string());
                        rest = rem;
                        continue;
                    }
                }

                // --- $named placeholder ---
                if let Some(after) = rest.strip_prefix("$") {
                    if let Some((name, rem)) = after
                        .char_indices()
                        .take_while(|(_, c)| c.is_alphanumeric() || *c == '_')
                        .map(|(i, _)| i)
                        .last()
                        .map(|i| after.split_at(i + 1))
                    {
                        if !buf.is_empty() {
                            branches.push(PgAst::Sql(std::mem::take(&mut buf)));
                        }
                        branches.push(PgAst::Placeholder(name.to_string()));
                        placeholders_out.push(name.to_string());
                        rest = rem;
                        continue;
                    }
                }

                // --- ? positional placeholder ---
                if let Some(r) = rest.strip_prefix("?") {
                    if !buf.is_empty() {
                        branches.push(PgAst::Sql(std::mem::take(&mut buf)));
                    }
                    *positional_counter += 1;
                    let name = positional_counter.to_string();
                    branches.push(PgAst::Placeholder(name.clone()));
                    placeholders_out.push(name);
                    rest = r;
                    continue;
                }

                // --- :: type cast ---
                if let Some(r) = rest.strip_prefix("::") {
                    buf.push_str("::");
                    rest = r;
                    continue;
                }

                // Default: consume one character
                let ch = rest.chars().next().unwrap();
                let ch_len = ch.len_utf8();
                buf.push_str(&rest[..ch_len]);
                rest = &rest[ch_len..];
            }

            if !buf.is_empty() {
                branches.push(PgAst::Sql(buf));
            }
            Ok(rest)
        }

        let mut branches = Vec::new();
        let mut placeholders = Vec::new();
        let mut counter = 0;
        let rest = inner(input, &mut placeholders, &mut branches, &mut counter)?;
        if !rest.trim().is_empty() {
            return Err("Unmatched `{{` or extra trailing content".into());
        }

        Ok(PgAst::Root {
            branches,
            required_placeholders: placeholders,
        })
    }
}

impl PgAst {
    /// Parses an input SQL query containing optional blocks `{{ ... }}`, placeholders `$...`, `:param`, `?`,
    /// but ignores them inside string literals and comments, with support for escaping via `\\`.
    /// Returns an `AST::Nested` of top-level branches.

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
                            PgParameterValue::RenderedSelectClause(fields) => {
                                for (
                                    i,
                                    SelectClauseRenderedField {
                                        field,
                                        expression: expr,
                                    },
                                ) in fields.__inner.iter().enumerate()
                                {
                                    if i > 0 {
                                        sql.push_str(", ");
                                    }
                                    if let Some(expr) = expr {
                                        sql.push_str(&format!("{expr} AS \"{field}\""));
                                    } else {
                                        sql.push_str(&format!("\"{field}\""));
                                    }
                                }
                            }
                            PgParameterValue::RenderedByClause(order_by) => {
                                for (
                                    i,
                                    ByClauseRenderedField {
                                        expression_or_identifier,
                                        is_expression,
                                        descending_order,
                                    },
                                ) in order_by.__inner.iter().enumerate()
                                {
                                    if i > 0 {
                                        sql.push_str(", ");
                                    }
                                    if *is_expression {
                                        sql.push_str(expression_or_identifier);
                                    } else {
                                        sql.push_str(&format!("\"{expression_or_identifier}\""));
                                    }
                                    if *descending_order {
                                        sql.push_str(" DESC");
                                    }
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
                        if let Some(value) = values.get(ph) {
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
                PgAst::InClause { expr, placeholder } => match values.get(placeholder) {
                    Some(PgParameterValue::Array(arr)) if !arr.is_empty() => {
                        sql.push_str(expr);
                        sql.push_str(" IN (");
                        for (i, item) in arr.iter().enumerate() {
                            if i > 0 {
                                sql.push_str(", ");
                            }
                            out_vals.push(item.clone());
                            write!(sql, "${}", out_vals.len()).unwrap();
                        }
                        sql.push(')');
                    }
                    _ => {
                        write!(sql, "FALSE /* {expr} IN :{placeholder} */").unwrap();
                    }
                },
                PgAst::NotInClause { expr, placeholder } => match values.get(placeholder) {
                    Some(PgParameterValue::Array(arr)) if !arr.is_empty() => {
                        sql.reserve(expr.len() + 9 + arr.len() * 2 + (arr.len() - 1) * 2);
                        sql.push_str(expr);
                        sql.push_str(" NOT IN (");
                        for (i, item) in arr.iter().enumerate() {
                            if i > 0 {
                                sql.push_str(", ");
                            }
                            out_vals.push(item.clone());
                            write!(sql, "${}", out_vals.len()).unwrap();
                        }
                        sql.push(')');
                    }
                    _ => {
                        write!(sql, "TRUE /* {expr} NOT IN :{placeholder} */").unwrap();
                    }
                },
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
            if let Some(missing_placeholder) = required_placeholders.into_iter().find(|&ph| {
                if let Some(value) = values.get(ph) {
                    value.is_empty()
                } else {
                    true
                }
            }) {
                bail!("Missing required placeholder `{missing_placeholder}`");
            }
        }
        walk(self, &values, &mut sql, &mut out_vals);
        let sql = sql.split_whitespace().join(" ");
        #[cfg(test)]
        println!("SQL = {sql}");
        Ok((sql, out_vals))
    }
}
