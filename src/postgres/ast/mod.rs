#![allow(clippy::inline_always)]
#[cfg(test)]
mod tests;

use crate::ByClauseRendered;
use crate::byclause::ByClauseRenderedField;
use crate::paginateclause::PaginateClauseRendered;
use crate::selectclause::{SelectClauseRendered, SelectClauseRenderedField};
use crate::utils::StripPrefixIgnoreAsciiCase;
use anyhow::bail;
use ext_php_rs::ZvalConvert;
use itertools::Itertools;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Write};
use trim_in_place::TrimInPlace;

#[derive(Debug, Clone)]
#[derive(PartialEq)]
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
    PaginateClause {
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
    ByClauseRendered(ByClauseRendered),
    SelectClauseRendered(SelectClauseRendered),
    PaginateClauseRendered(PaginateClauseRendered),
}
impl PgParameterValue {
    #[must_use]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        match self {
            PgParameterValue::ByClauseRendered(x) => x.is_empty(),
            PgParameterValue::SelectClauseRendered(x) => x.is_empty(),
            PgParameterValue::Array(array) => array.is_empty(),
            PgParameterValue::Str(_)
            | PgParameterValue::Int(_)
            | PgParameterValue::Float(_)
            | PgParameterValue::Bool(_)
            | PgParameterValue::Object(_) => false,
            PgParameterValue::PaginateClauseRendered(_) => false,
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
    pub fn parse(input: &str, collapsible_in_enabled: bool) -> anyhow::Result<PgAst> {
        fn inner<'s>(
            mut rest: &'s str,
            placeholders_out: &mut Vec<String>,
            branches: &mut Vec<PgAst>,
            positional_counter: &mut usize,
            collapsible_in_enabled: bool,
        ) -> anyhow::Result<&'s str> {
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
                    let end = r.find('\n').map_or(r.len(), |i| i + 1);
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
                    }
                    bail!("Unterminated block comment");
                }

                // Conditional block start
                if let Some(r) = rest.strip_prefix("{{") {
                    if !buf.is_empty() {
                        branches.push(PgAst::Sql(std::mem::take(&mut buf)));
                    }
                    let mut inner_branches = Vec::new();
                    let mut inner_ph = Vec::new();
                    rest = inner(
                        r,
                        &mut inner_ph,
                        &mut inner_branches,
                        positional_counter,
                        collapsible_in_enabled,
                    )?;
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

                if let Some(suffix) = rest.strip_prefix_ignore_ascii_case("PAGINATE") {
                    let rest_after_in = suffix.trim_start();
                    let offset = rest.len() - suffix.len();
                    let mut consumed_len = 0;
                    let mut name_opt = None;
                    if let Some(sfx) = rest_after_in.strip_prefix(':') {
                        let name: String = sfx
                            .chars()
                            .take_while(|c| c.is_alphanumeric() || *c == '_')
                            .collect();
                        consumed_len = offset + 2 + name.len();
                        name_opt = Some(name);
                    } else if let Some(sfx) = rest_after_in.strip_prefix('$') {
                        let name: String = sfx
                            .chars()
                            .take_while(|c| c.is_alphanumeric() || *c == '_')
                            .collect();
                        consumed_len = offset + 2 + name.len();
                        name_opt = Some(name);
                    } else if rest_after_in.starts_with('?') {
                        *positional_counter += 1;
                        consumed_len = offset + 2;
                        name_opt = Some(positional_counter.to_string());
                    }
                    if let Some(name) = name_opt {
                        branches.push(PgAst::Sql(format!("{buf} ")));
                        buf.clear();
                        placeholders_out.push(name.to_string());
                        branches.push(PgAst::PaginateClause { placeholder: name });
                        rest = &rest[consumed_len..];
                        continue;
                    }
                }

                if collapsible_in_enabled {
                    // NOT IN support (with or without parentheses)
                    if let Some(suffix) = rest.strip_prefix_ignore_ascii_case("NOT IN") {
                        let rest_after_in = suffix.trim_start();
                        let offset = rest.len() - suffix.len();
                        let mut consumed_len = 0;
                        let mut name_opt = None;
                        if let Some(stripped) = rest_after_in.strip_prefix('(') {
                            // parentheses form
                            if let Some(cl) = stripped.find(')') {
                                let inside = &stripped[..cl].trim();
                                if let Some(id) = inside.strip_prefix(':') {
                                    consumed_len = offset + 1 + cl + 2;
                                    name_opt = Some(id.to_string());
                                } else if let Some(id) = inside.strip_prefix('$') {
                                    consumed_len = offset + 1 + cl + 2;
                                    name_opt = Some(id.to_string());
                                } else if *inside == "?" {
                                    *positional_counter += 1;
                                    consumed_len = offset + 1 + cl + 2;
                                    name_opt = Some(positional_counter.to_string());
                                }
                            }
                        } else {
                            // non-parentheses form
                            if let Some(sfx) = rest_after_in.strip_prefix(':') {
                                let name: String = sfx
                                    .chars()
                                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                                    .collect();
                                consumed_len = offset + 2 + name.len();
                                name_opt = Some(name);
                            } else if let Some(sfx) = rest_after_in.strip_prefix('$') {
                                let name: String = sfx
                                    .chars()
                                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                                    .collect();
                                consumed_len = offset + 2 + name.len();
                                name_opt = Some(name);
                            } else if rest_after_in.starts_with('?') {
                                *positional_counter += 1;
                                consumed_len = offset + 2;
                                name_opt = Some(positional_counter.to_string());
                            }
                        }
                        if let Some(name) = name_opt {
                            buf.trim_end_in_place();
                            if let Some((left, expr)) = buf.rsplit_once(char::is_whitespace) {
                                if !left.is_empty() {
                                    branches.push(PgAst::Sql(format!("{left} ")));
                                }
                                branches.push(PgAst::NotInClause {
                                    expr: expr.to_string(),
                                    placeholder: name,
                                });
                            } else {
                                branches.push(PgAst::NotInClause {
                                    expr: buf.clone(),
                                    placeholder: name,
                                });
                            }
                            buf.clear();
                            rest = &rest[consumed_len..];
                            continue;
                        }
                    }

                    // --- IN ... ---
                    if let Some(rest_after_in) = rest.strip_prefix_ignore_ascii_case("IN") {
                        let rest_after_in = rest_after_in.trim_start();
                        let original_len = rest.len();

                        let mut consumed_len = 0;
                        let mut name_opt = None;
                        if let Some(sfx) = rest_after_in.strip_prefix(':') {
                            let ident: String = sfx
                                .chars()
                                .take_while(|c| c.is_alphanumeric() || *c == '_')
                                .collect();
                            consumed_len = original_len - rest_after_in.len() + 1 + ident.len();
                            name_opt = Some(ident);
                        } else if let Some(sfx) = rest_after_in.strip_prefix('$') {
                            consumed_len = original_len - rest_after_in.len();
                            name_opt = Some(
                                sfx.chars()
                                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                                    .collect(),
                            );
                        } else if rest_after_in.starts_with('?') {
                            *positional_counter += 1;
                            consumed_len = original_len - rest_after_in.len() + 1;
                            name_opt = Some(positional_counter.to_string());
                        } else if let Some(stripped) = rest_after_in.strip_prefix("(") {
                            if let Some(close_idx) = stripped.find(')') {
                                let inside = &stripped[..close_idx].trim();
                                if let Some(id) = inside.strip_prefix(':') {
                                    name_opt = Some(id.to_string());
                                } else if let Some(id) = inside.strip_prefix('$') {
                                    name_opt = Some(id.to_string());
                                } else if *inside == "?" {
                                    *positional_counter += 1;
                                    name_opt = Some(positional_counter.to_string());
                                }
                                consumed_len =
                                    original_len - rest_after_in.len() + 1 + close_idx + 1;
                            }
                        }

                        if let Some(name) = name_opt {
                            buf.trim_end_in_place();
                            if let Some((left, expr)) = buf.rsplit_once(char::is_whitespace) {
                                if !left.is_empty() {
                                    branches.push(PgAst::Sql(format!("{left} ")));
                                }
                                branches.push(PgAst::InClause {
                                    expr: expr.to_string(),
                                    placeholder: name,
                                });
                            } else {
                                branches.push(PgAst::InClause {
                                    expr: buf.clone(),
                                    placeholder: name,
                                });
                            }
                            buf.clear();
                            rest = &rest[consumed_len..];
                            continue;
                        }
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
        let rest = inner(
            input,
            &mut placeholders,
            &mut branches,
            &mut counter,
            collapsible_in_enabled,
        )?;
        if !rest.trim().is_empty() {
            bail!("Unmatched `{{` or extra trailing content");
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
    ///
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
        ) -> anyhow::Result<()> {
            Ok(match node {
                PgAst::Root { branches, .. } | PgAst::Nested(branches) => {
                    for n in branches {
                        walk(n, values, sql, out_vals)?;
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
                            PgParameterValue::SelectClauseRendered(fields) => {
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
                                        write!(sql, "{expr} AS \"{field}\"").unwrap();
                                    } else {
                                        write!(sql, "\"{field}\"").unwrap();
                                    }
                                }
                            }
                            PgParameterValue::ByClauseRendered(by) => {
                                for (
                                    i,
                                    ByClauseRenderedField {
                                        expression_or_identifier,
                                        is_expression,
                                        descending_order,
                                    },
                                ) in by.__inner.iter().enumerate()
                                {
                                    if i > 0 {
                                        sql.push_str(", ");
                                    }
                                    if *is_expression {
                                        sql.push_str(expression_or_identifier);
                                    } else {
                                        write!(sql, "\"{expression_or_identifier}\"").unwrap();
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
                            walk(b, values, sql, out_vals)?;
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
                            write!(sql, "${}", out_vals.len())?;
                        }
                        sql.push(')');
                    }
                    _ => {
                        write!(sql, "TRUE /* {expr} NOT IN :{placeholder} */")?;
                    }
                },
                PgAst::PaginateClause { placeholder } => match values.get(placeholder) {
                    Some(PgParameterValue::PaginateClauseRendered(rendered)) => {
                        out_vals.push(PgParameterValue::Int(rendered.limit));
                        out_vals.push(PgParameterValue::Int(rendered.offset));
                        write!(
                            sql,
                            "LIMIT ${} OFFSET ${}",
                            out_vals.len() - 1,
                            out_vals.len()
                        )?;
                    }
                    _ => {
                        bail!("PAGINATE may only accept Sqlx\\PaginateClause instance");
                    }
                },
            })
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
            if let Some(missing_placeholder) = required_placeholders.iter().find(|&ph| {
                if let Some(value) = values.get(ph) {
                    value.is_empty()
                } else {
                    true
                }
            }) {
                bail!("Missing required placeholder `{missing_placeholder}`");
            }
        }
        walk(self, &values, &mut sql, &mut out_vals)?;
        let sql = sql.split_whitespace().join(" ");
        #[cfg(test)]
        println!("SQL = {sql}");
        Ok((sql, out_vals))
    }
}
