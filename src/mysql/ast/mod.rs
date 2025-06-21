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

#[derive(Debug, PartialEq, Clone)]
pub enum MySqlAst {
    Nested(Vec<MySqlAst>),
    /// Literal SQL text
    Sql(String),
    /// Placeholder like `$id`, `:param`, positional `?` replaced with ordinal number
    Placeholder(String),
    /// Optional segment with its own nested branches and collected placeholders
    ConditionalBlock {
        branches: Vec<MySqlAst>,
        required_placeholders: Vec<Placeholder>,
    },
    Root {
        branches: Vec<MySqlAst>,
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
pub type Placeholder = String;

/// Supported parameter types
#[derive(ZvalConvert, Debug, Clone, PartialEq)]
pub enum MySqlParameterValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<MySqlParameterValue>),
    Object(HashMap<String, MySqlParameterValue>),
    SelectClauseRendered(SelectClauseRendered),
    ByClauseRendered(ByClauseRendered),
    PaginateClauseRendered(PaginateClauseRendered),
}
impl MySqlParameterValue {
    #[must_use]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        match self {
            MySqlParameterValue::ByClauseRendered(x) => x.is_empty(),
            MySqlParameterValue::SelectClauseRendered(x) => x.is_empty(),
            MySqlParameterValue::Array(array) => array.is_empty(),
            MySqlParameterValue::Str(_)
            | MySqlParameterValue::Int(_)
            | MySqlParameterValue::Float(_)
            | MySqlParameterValue::Bool(_)
            | MySqlParameterValue::Object(_) => false,
            MySqlParameterValue::PaginateClauseRendered(_) => false,
        }
    }
}
pub type ParamsMap = BTreeMap<String, MySqlParameterValue>;
impl From<&str> for MySqlParameterValue {
    fn from(s: &str) -> Self {
        MySqlParameterValue::Str(s.to_string())
    }
}
impl From<String> for MySqlParameterValue {
    fn from(s: String) -> Self {
        MySqlParameterValue::Str(s)
    }
}

impl From<i64> for MySqlParameterValue {
    fn from(s: i64) -> Self {
        MySqlParameterValue::Int(s)
    }
}

impl From<bool> for MySqlParameterValue {
    fn from(s: bool) -> Self {
        MySqlParameterValue::Bool(s)
    }
}

impl MySqlAst {
    /// Parses an input SQL query containing optional blocks `{{ ... }}`, placeholders `$...`, `:param`, `?`,
    /// but ignores them inside string literals and comments.
    pub fn parse(input: &str, collapsible_in_enabled: bool) -> anyhow::Result<MySqlAst> {
        fn inner<'s>(
            mut rest: &'s str,
            placeholders_out: &mut Vec<String>,
            branches: &mut Vec<MySqlAst>,
            positional_counter: &mut usize,
            collapsible_in_enabled: bool,
        ) -> anyhow::Result<&'s str> {
            let mut buf = String::new();

            while !rest.is_empty() {
                // Handle single-quoted string literal with backslash and '' escapes
                if rest.starts_with('\'') {
                    let mut idx = 1;
                    let mut iter = rest.char_indices().skip(1).peekable();
                    while let Some((i, c)) = iter.next() {
                        if c == '\\' {
                            if let Some((j, _)) = iter.next() {
                                idx = j + 1;
                            }
                        } else if c == '\'' {
                            if let Some((_, '\'')) = iter.peek().copied() {
                                iter.next();
                                idx = i + 2;
                                continue;
                            }
                            idx = i + 1;
                            break;
                        }
                    }
                    let literal = &rest[..idx];
                    buf.push_str(literal);
                    rest = &rest[idx..];
                    continue;
                }
                // Line comment --
                if let Some(r) = rest.strip_prefix("--") {
                    let end = r.find('\n').map_or(r.len(), |i| i + 1);
                    buf.push_str(&rest[..2 + end]);
                    rest = &rest[2 + end..];
                    continue;
                }
                // Line comment #
                if let Some(r) = rest.strip_prefix("#") {
                    let end = r.find('\n').map_or(r.len(), |i| i + 1);
                    buf.push_str(&rest[..=end]);
                    rest = &rest[1 + end..];
                    continue;
                }
                // Block comment /* */
                if let Some(r) = rest.strip_prefix("/*") {
                    if let Some(close) = r.find("*/") {
                        buf.push_str(&rest[..2 + close + 2]);
                        rest = &rest[2 + close + 2..];
                        continue;
                    }
                    bail!("Unterminated block comment");
                }
                // Conditional block start
                if let Some(r) = rest.strip_prefix("{{") {
                    if !buf.is_empty() {
                        branches.push(MySqlAst::Sql(std::mem::take(&mut buf)));
                    }
                    let mut inner_br = Vec::new();
                    let mut inner_ph = Vec::new();
                    rest = inner(
                        r,
                        &mut inner_ph,
                        &mut inner_br,
                        positional_counter,
                        collapsible_in_enabled,
                    )?;
                    branches.push(MySqlAst::ConditionalBlock {
                        branches: inner_br,
                        required_placeholders: inner_ph,
                    });
                    continue;
                }
                // }} conditional end
                if let Some(r) = rest.strip_prefix("}}") {
                    if !buf.is_empty() {
                        branches.push(MySqlAst::Sql(std::mem::take(&mut buf)));
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
                        branches.push(MySqlAst::Sql(format!("{buf} ")));
                        buf.clear();
                        placeholders_out.push(name.to_string());
                        branches.push(MySqlAst::PaginateClause { placeholder: name });
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
                        // parentheses form
                        if let Some(stripped) = rest_after_in.strip_prefix('(') {
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
                                let ident: String = sfx
                                    .chars()
                                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                                    .collect();
                                consumed_len = offset + 2 + ident.len();
                                name_opt = Some(ident);
                            } else if let Some(sfx) = rest_after_in.strip_prefix('$') {
                                let ident: String = sfx
                                    .chars()
                                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                                    .collect();
                                consumed_len = offset + 2 + ident.len();
                                name_opt = Some(ident);
                            } else if rest_after_in.starts_with('?') {
                                *positional_counter += 1;
                                let num = positional_counter.to_string();
                                consumed_len = offset + 2;
                                name_opt = Some(num);
                            }
                        }
                        if let Some(name) = name_opt {
                            let trimmed = buf.trim_end();
                            let (pre, expr) = match trimmed.rsplit_once(char::is_whitespace) {
                                Some((a, b)) => (format!("{a} "), b),
                                None => (String::new(), trimmed),
                            };
                            if !pre.is_empty() {
                                branches.push(MySqlAst::Sql(pre));
                            }
                            branches.push(MySqlAst::NotInClause {
                                expr: expr.to_string(),
                                placeholder: name,
                            });
                            buf.clear();
                            rest = &rest[consumed_len..];
                            continue;
                        }
                    }
                    // IN support
                    if let Some(r2) = rest.strip_prefix_ignore_ascii_case("IN") {
                        let rest_after_in = r2.trim_start();
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
                            let ident: String = sfx
                                .chars()
                                .take_while(|c| c.is_alphanumeric() || *c == '_')
                                .collect();
                            consumed_len = original_len - rest_after_in.len() + 1 + ident.len();
                            name_opt = Some(ident);
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
                            let trimmed = buf.trim_end();
                            let (pre, expr) = match trimmed.rsplit_once(char::is_whitespace) {
                                Some((a, b)) => (format!("{a} "), b),
                                None => (String::new(), trimmed),
                            };
                            if !pre.is_empty() {
                                branches.push(MySqlAst::Sql(pre));
                            }
                            branches.push(MySqlAst::InClause {
                                expr: expr.to_string(),
                                placeholder: name,
                            });
                            buf.clear();
                            rest = &rest[consumed_len..];
                            continue;
                        }
                    }
                }
                // :param
                if let Some(a) = rest.strip_prefix(":") {
                    if let Some((nm, rm)) = a
                        .char_indices()
                        .take_while(|(_, c)| c.is_alphanumeric() || *c == '_')
                        .map(|(i, _)| i)
                        .last()
                        .map(|i| a.split_at(i + 1))
                    {
                        if !buf.is_empty() {
                            branches.push(MySqlAst::Sql(std::mem::take(&mut buf)));
                        }
                        branches.push(MySqlAst::Placeholder(nm.to_string()));
                        placeholders_out.push(nm.to_string());
                        rest = rm;
                        continue;
                    }
                }
                // $param
                if let Some(a) = rest.strip_prefix("$") {
                    if let Some((nm, rm)) = a
                        .char_indices()
                        .take_while(|(_, c)| c.is_alphanumeric() || *c == '_')
                        .map(|(i, _)| i)
                        .last()
                        .map(|i| a.split_at(i + 1))
                    {
                        if !buf.is_empty() {
                            branches.push(MySqlAst::Sql(std::mem::take(&mut buf)));
                        }
                        branches.push(MySqlAst::Placeholder(nm.to_string()));
                        placeholders_out.push(nm.to_string());
                        rest = rm;
                        continue;
                    }
                }
                // ? positional
                if let Some(rp) = rest.strip_prefix("?") {
                    if !buf.is_empty() {
                        branches.push(MySqlAst::Sql(std::mem::take(&mut buf)));
                    }
                    *positional_counter += 1;
                    let nm = positional_counter.to_string();
                    branches.push(MySqlAst::Placeholder(nm.clone()));
                    placeholders_out.push(nm);
                    rest = rp;
                    continue;
                }
                // :: cast
                if let Some(rp) = rest.strip_prefix("::") {
                    buf.push_str("::");
                    rest = rp;
                    continue;
                }
                // default char
                let ch = rest.chars().next().unwrap();
                let l = ch.len_utf8();
                buf.push_str(&rest[..l]);
                rest = &rest[l..];
            }
            if !buf.is_empty() {
                branches.push(MySqlAst::Sql(buf));
            }
            Ok(rest)
        }
        let mut branches = Vec::new();
        let mut placeholders = Vec::new();
        let mut cnt = 0;
        let rest = inner(
            input,
            &mut placeholders,
            &mut branches,
            &mut cnt,
            collapsible_in_enabled,
        )?;
        if !rest.trim().is_empty() {
            bail!("Unmatched `{{` or extra trailing content");
        }
        Ok(MySqlAst::Root {
            branches,
            required_placeholders: placeholders,
        })
    }
}

impl MySqlAst {
    /// Renders the AST into an SQL string with numbered placeholders like `$1`, `$2`, ...
    /// `values` can be any iterable of (key, value) pairs. Keys convertible to String; values convertible to Value.
    pub fn render<I, K, V>(&self, values: I) -> anyhow::Result<(String, Vec<MySqlParameterValue>)>
    where
        I: IntoIterator<Item = (K, V)> + Debug,
        K: Into<String>,
        V: Into<MySqlParameterValue>,
    {
        #[cfg(test)]
        {
            println!("AST = {:?}", self);
            println!("VALUES = {:?}", values);
        }
        fn walk(
            node: &MySqlAst,
            values: &ParamsMap,
            sql: &mut String,
            out_vals: &mut Vec<MySqlParameterValue>,
        ) -> anyhow::Result<()> {
            Ok(match node {
                MySqlAst::Root { branches, .. } | MySqlAst::Nested(branches) => {
                    for n in branches {
                        walk(n, values, sql, out_vals)?;
                    }
                }
                MySqlAst::Sql(s) => sql.push_str(s),
                MySqlAst::Placeholder(name) => {
                    #[cfg(test)]
                    {
                        println!("values = {values:?}");
                        println!("{name:?} ==> {:?}", values.get(name));
                    }
                    if let Some(val) = values.get(name) {
                        match val {
                            MySqlParameterValue::SelectClauseRendered(fields) => {
                                for (i, SelectClauseRenderedField { field, expression }) in
                                    fields.__inner.iter().enumerate()
                                {
                                    if i > 0 {
                                        sql.push_str(", ");
                                    }
                                    if let Some(expression) = expression {
                                        write!(sql, "{expression} AS `{field}`").unwrap();
                                    } else {
                                        write!(sql, "`{field}`").unwrap();
                                    }
                                }
                            }
                            MySqlParameterValue::ByClauseRendered(by) => {
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
                                        write!(sql, "`{expression_or_identifier}`").unwrap();
                                    }
                                    if *descending_order {
                                        sql.push_str(" DESC");
                                    }
                                }
                            }
                            MySqlParameterValue::Array(arr) => {
                                for (i, item) in arr.iter().enumerate() {
                                    if i > 0 {
                                        sql.push_str(", ");
                                    }
                                    write!(sql, "?").unwrap();
                                    out_vals.push(item.clone());
                                }
                            }
                            _ => {
                                write!(sql, "?").unwrap();
                                out_vals.push(val.clone());
                            }
                        }
                    }
                }
                MySqlAst::ConditionalBlock {
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

                MySqlAst::InClause { expr, placeholder } => match values.get(placeholder) {
                    Some(MySqlParameterValue::Array(arr)) if !arr.is_empty() => {
                        sql.reserve(expr.len() + 5 + arr.len() + (arr.len() - 1) * 2);
                        sql.push_str(expr);
                        sql.push_str(" IN (");
                        for (i, item) in arr.iter().enumerate() {
                            if i > 0 {
                                sql.push_str(", ");
                            }
                            out_vals.push(item.clone());
                            sql.push('?');
                        }
                        sql.push(')');
                    }
                    _ => {
                        write!(sql, "FALSE /* {expr} IN :{placeholder} */").unwrap();
                    }
                },
                MySqlAst::NotInClause { expr, placeholder } => match values.get(placeholder) {
                    Some(MySqlParameterValue::Array(arr)) if !arr.is_empty() => {
                        sql.reserve(expr.len() + 9 + arr.len() + (arr.len() - 1) * 2);
                        write!(sql, "{expr} NOT IN (").unwrap();
                        sql.push_str(" NOT IN (");
                        for (i, item) in arr.iter().enumerate() {
                            if i > 0 {
                                sql.push_str(", ");
                            }
                            out_vals.push(item.clone());
                            sql.push('?');
                        }
                        sql.push(')');
                    }
                    _ => {
                        write!(sql, "TRUE /* {expr} IN :{placeholder} */").unwrap();
                    }
                },
                MySqlAst::PaginateClause { placeholder } => match values.get(placeholder) {
                    Some(MySqlParameterValue::PaginateClauseRendered(rendered)) => {
                        out_vals.push(MySqlParameterValue::Int(rendered.limit));
                        out_vals.push(MySqlParameterValue::Int(rendered.offset));
                        sql.push_str("LIMIT ? OFFSET ?");
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
        if let MySqlAst::Root {
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
