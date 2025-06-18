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

#[derive(Debug, Clone)]
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
    RenderedSelectClause(SelectClauseRendered),
    RenderedByClause(ByClauseRendered),
}
impl MySqlParameterValue {
    #[must_use]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        match self {
            MySqlParameterValue::RenderedByClause(x) => x.is_empty(),
            MySqlParameterValue::RenderedSelectClause(x) => x.is_empty(),
            MySqlParameterValue::Array(array) => array.is_empty(),
            MySqlParameterValue::Str(_)
            | MySqlParameterValue::Int(_)
            | MySqlParameterValue::Float(_)
            | MySqlParameterValue::Bool(_)
            | MySqlParameterValue::Object(_) => false,
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
    /// but ignores them inside string literals and comments, with support for escaping via `\\`.
    /// Returns an `AST::Nested` of top-level branches.
    pub fn parse(input: &str) -> Result<MySqlAst, String> {
        fn inner<'s>(
            mut rest: &'s str,
            placeholders_out: &mut Vec<String>,
            branches: &mut Vec<MySqlAst>,
            positional_counter: &mut usize,
        ) -> Result<&'s str, String> {
            let mut buf = String::new();

            while !rest.is_empty() {
                // Conditional block start
                if let Some(r) = rest.strip_prefix("{{") {
                    if !buf.is_empty() {
                        branches.push(MySqlAst::Sql(std::mem::take(&mut buf)));
                    }
                    let mut inner_branches = Vec::new();
                    let mut inner_ph = Vec::new();
                    rest = inner(r, &mut inner_ph, &mut inner_branches, positional_counter)?;
                    branches.push(MySqlAst::ConditionalBlock {
                        branches: inner_branches,
                        required_placeholders: inner_ph,
                    });
                    continue;
                }

                // Conditional block end
                if let Some(r) = rest.strip_prefix("}}") {
                    if !buf.is_empty() {
                        branches.push(MySqlAst::Sql(std::mem::take(&mut (buf))));
                    }
                    return Ok(r);
                }

                // --- NOT IN (...) ---
                if let Some(suffix) = rest.strip_prefix("NOT IN") {
                    if let Some(open) = suffix.find('(') {
                        if let Some(close_idx) = suffix[open + 1..].find(')') {
                            let inside = &suffix[open + 1..open + 1 + close_idx].trim();
                            // determine placeholder name and handle '?' as positional
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

                            // split buf into prefix (with trailing space) and expr
                            let trimmed = buf.trim_end();
                            let (pre, expr) = match trimmed.rsplit_once(char::is_whitespace) {
                                Some((a, b)) => (format!("{} ", a), b),
                                None => ("".to_string(), trimmed),
                            };
                            if !pre.is_empty() {
                                branches.push(MySqlAst::Sql(pre));
                            }
                            branches.push(MySqlAst::NotInClause {
                                expr: expr.to_string(),
                                placeholder: name.clone(),
                            });
                            placeholders_out.push(name);
                            buf.clear();
                            rest = &suffix[open + 1 + close_idx + 1..];
                            continue;
                        }
                    }
                }

                // --- IN ... ---
                if let Some(rest_after_in) = rest.strip_prefix("IN") {
                    let rest_after_in = rest_after_in.trim_start();
                    let original_len = rest.len();

                    // determine placeholder/name and consumed length
                    let (name_opt, consumed_len) =
                        if let Some(sfx) = rest_after_in.strip_prefix(':') {
                            let ident: String = sfx
                                .chars()
                                .take_while(|c| c.is_alphanumeric() || *c == '_')
                                .collect();
                            (
                                Some(ident.clone()),
                                original_len - rest_after_in.len() + 1 + ident.len(),
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
                            // positional inside IN
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
                        // split buf into prefix (with trailing space) and expr
                        let trimmed = buf.trim_end();
                        let (pre, expr) = match trimmed.rsplit_once(char::is_whitespace) {
                            Some((a, b)) => (format!("{} ", a), b),
                            None => ("".to_string(), trimmed),
                        };
                        if !pre.is_empty() {
                            branches.push(MySqlAst::Sql(pre));
                        }
                        branches.push(MySqlAst::InClause {
                            expr: expr.to_string(),
                            placeholder: name.clone(),
                        });
                        placeholders_out.push(name);
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
                            branches.push(MySqlAst::Sql(std::mem::take(&mut buf)));
                        }
                        branches.push(MySqlAst::Placeholder(name.to_string()));
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
                            branches.push(MySqlAst::Sql(std::mem::take(&mut buf)));
                        }
                        branches.push(MySqlAst::Placeholder(name.to_string()));
                        placeholders_out.push(name.to_string());
                        rest = rem;
                        continue;
                    }
                }

                // --- ? positional placeholder ---
                if let Some(r) = rest.strip_prefix("?") {
                    if !buf.is_empty() {
                        branches.push(MySqlAst::Sql(std::mem::take(&mut buf)));
                    }
                    *positional_counter += 1;
                    let name = positional_counter.to_string();
                    branches.push(MySqlAst::Placeholder(name.clone()));
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
                branches.push(MySqlAst::Sql(buf));
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

        Ok(MySqlAst::Root {
            branches,
            required_placeholders: placeholders,
        })
    }

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
        ) {
            match node {
                MySqlAst::Root { branches, .. } | MySqlAst::Nested(branches) => {
                    for n in branches {
                        walk(n, values, sql, out_vals);
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
                            MySqlParameterValue::RenderedSelectClause(fields) => {
                                for (i, SelectClauseRenderedField { field, expression }) in
                                    fields.__inner.iter().enumerate()
                                {
                                    if i > 0 {
                                        sql.push_str(", ");
                                    }
                                    if let Some(expression) = expression {
                                        sql.push_str(&format!("{expression} AS `{field}`"));
                                    } else {
                                        sql.push_str(&format!("`{field}`"));
                                    }
                                }
                            }
                            MySqlParameterValue::RenderedByClause(order_by) => {
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
                                        sql.push_str(&format!("`{expression_or_identifier}`"));
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
                            walk(b, values, sql, out_vals);
                        }
                    }
                }
                MySqlAst::InClause { expr, placeholder } => {
                    if let Some(MySqlParameterValue::Array(arr)) = values.get(placeholder) {
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
                }
                MySqlAst::NotInClause { expr, placeholder } => {
                    if let Some(MySqlParameterValue::Array(arr)) = values.get(placeholder) {
                        sql.push_str(expr);
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
        walk(self, &values, &mut sql, &mut out_vals);
        let sql = sql.split_whitespace().join(" ");

        #[cfg(test)]
        println!("SQL = {sql}");
        Ok((sql, out_vals))
    }
}
