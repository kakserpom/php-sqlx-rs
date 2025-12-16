//! SQL Abstract Syntax Tree (AST) parsing and rendering for php-sqlx.
//!
//! This module provides AST-based SQL processing that enables:
//!
//! - **Conditional blocks**: `{{ ... }}` sections that render only when placeholders are provided
//! - **Named placeholders**: `$name` and `:name` style parameter binding
//! - **Positional placeholders**: `?` markers converted to ordinal references
//! - **IN clause optimization**: Automatic expansion of arrays and collapsing of empty lists
//! - **Pagination**: `PAGINATE :param` expansion into `LIMIT`/`OFFSET`
//!
//! # Example
//!
//! ```sql
//! SELECT * FROM users
//! WHERE status = :status
//! {{ AND role IN :roles }}
//! {{ AND created_at > :since }}
//! ```
//!
//! In this query:
//! - `:status` is required
//! - The `role IN :roles` clause only appears if `:roles` is provided and non-empty
//! - The `created_at > :since` clause only appears if `:since` is provided
//!
//! # Collapsible IN Clauses
//!
//! When `collapsible_in_enabled` is true:
//! - `column IN :empty_array` becomes `FALSE`
//! - `column NOT IN :empty_array` becomes `TRUE`

use crate::param_value::{ParameterValue, ParamsMap, Placeholder, write::ParamVecWriteSqlTo};
use crate::utils::strip_prefix::StripPrefixWordIgnoreAsciiCase;
use crate::error::Error as SqlxError;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::fmt::Write;
use trim_in_place::TrimInPlace;
#[cfg(test)]
mod tests;

/// An Abstract Syntax Tree representation of a parameterized SQL query,
/// allowing for nested conditional blocks, placeholders, and special clauses.
#[derive(Debug, PartialEq, Clone)]
pub enum Ast {
    /// A sequence of AST nodes, used for nesting and grouping.
    Nested(Vec<Ast>),

    /// A literal fragment of SQL text.
    Raw(String),

    /// A placeholder such as `$id`, `:param`, or `?` (converted to an ordinal).
    Placeholder(String),

    /// A conditional segment (`{{ ... }}`) which only renders if its
    /// `required_placeholders` are all provided and non-empty.
    ConditionalBlock {
        /// The possible AST branches inside the block.
        branches: Vec<Ast>,
        /// Placeholders that must be present to render this block.
        required_placeholders: Vec<Placeholder>,
    },

    /// The root of the AST, containing top-level branches and the
    /// placeholders required for the entire query.
    Root {
        /// Top-level AST branches.
        branches: Vec<Ast>,
        /// Placeholders that must be provided for rendering.
        required_placeholders: Vec<Placeholder>,
    },

    /// An `expr IN (...)` clause with a single placeholder for the list.
    InClause {
        /// The left-hand expression (e.g. a column name).
        expr: String,
        /// The placeholder name holding the list values.
        placeholder: String,
    },

    /// An `expr NOT IN (...)` clause with a single placeholder for the list.
    NotInClause {
        /// The left-hand expression (e.g. a column name).
        expr: String,
        /// The placeholder name holding the list values.
        placeholder: String,
    },

    /// A pagination helper that expands into `LIMIT`/`OFFSET`.
    PaginateClause {
        /// The placeholder name carrying limit/offset info.
        placeholder: String,
    },
}

/// Settings that control how the SQL parser handles comments, escaping, and
/// optional features like collapsible `IN` clauses.
/// Settings that determine how placeholders and identifiers are rendered:
/// whether to use backticks, dollar-sign placeholders, `@`-style, etc.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Default, Clone)]
pub struct Settings {
    /// Enable special parsing for `IN` and `NOT IN` clauses.
    pub collapsible_in_enabled: bool,
    /// Treat `''` inside single-quoted strings as an escaped quote.
    pub escaping_double_single_quotes: bool,
    /// Recognize `#` as the start of a line comment.
    pub comment_hash: bool,
    /// Wrap column names in backticks instead of double quotes.
    pub column_backticks: bool,
    /// Render placeholders as `$1`, `$2`, ...
    pub placeholder_dollar_sign: bool,
    /// Render placeholders as `@p1`, `@p2`, ...
    pub placeholder_at_sign: bool,
    /// Maximum number of placeholders
    pub max_placeholders: usize,

    /// Emit `true`/`false` instead of `1`/`0`
    pub booleans_as_literals: bool,
    /// Wrap string literals using MSSQL-style N'...' Unicode prefix
    pub strings_as_ntext: bool,
    /// JSON objects should be cast (e.g. `::jsonb` or `AS JSON`)
    pub cast_json: Option<&'static str>, // e.g. Some("::jsonb") or Some("AS JSON")
    /// Escape `\` in strings (`MySQL` legacy)
    pub escape_backslash: bool,
}

impl Ast {
    /// Returns the required placeholders for this AST if it's a Root node.
    ///
    /// # Returns
    /// - `Some(&Vec<Placeholder>)` if this is a Root with non-empty placeholders
    /// - `None` if this is a Root with no placeholders
    ///
    /// # Panics
    /// Panics if called on a non-Root AST node.
    pub fn get_placeholders_if_any(&self) -> Option<&Vec<Placeholder>> {
        match self {
            Ast::Root {
                required_placeholders,
                ..
            } if !required_placeholders.is_empty() => Some(required_placeholders),
            Ast::Root { .. } => None,
            _ => unimplemented!("get_root_placeholders_if_any may only be called on Ast::Root"),
        }
    }

    /// Parse an SQL string with embedded optional blocks `{{ ... }}`, named
    /// placeholders (`$name`, `:name`), and positional `?` markers. Ignores
    /// matches inside string literals and comments.
    ///
    /// # Arguments
    ///
    /// * `input` – The raw SQL text to parse.
    /// * `settings` – Controls comment syntax, escaping, and `IN` support.
    ///
    /// # Returns
    ///
    /// An `Ast::Root` containing the parsed branches and the list of
    /// placeholders required to render the final SQL.
    pub fn parse(input: &str, settings: &Settings) -> crate::error::Result<Ast> {
        fn inner<'s>(
            mut rest: &'s str,
            placeholders_out: &mut Vec<String>,
            branches: &mut Vec<Ast>,
            positional_counter: &mut usize,
            settings: &Settings,
        ) -> crate::error::Result<&'s str> {
            let mut buf = String::new();

            while !rest.is_empty() {
                if rest.starts_with('\'') {
                    let mut idx = 1;
                    let mut iter = rest.char_indices().skip(1).peekable();
                    while let Some((i, c)) = iter.next() {
                        if settings.escaping_double_single_quotes {
                            if c == '\'' {
                                if let Some((_, '\'')) = iter.peek() {
                                    iter.next();
                                    continue;
                                }
                                idx = i + c.len_utf8();
                                break;
                            }
                        } else if c == '\\' {
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
                // Handle line comment -- until newline
                if let Some(r) = rest.strip_prefix("--") {
                    // include '--' and content up to newline
                    let end = r.find('\n').map_or(r.len(), |i| i + 1);
                    let comment = &rest[..2 + end];
                    buf.push_str(comment);
                    rest = &rest[2 + end..];
                    continue;
                }
                if settings.comment_hash {
                    // Line comment #
                    if let Some(r) = rest.strip_prefix("#") {
                        let end = r.find('\n').map_or(r.len(), |i| i + 1);
                        buf.push_str(&rest[..=end]);
                        rest = &rest[1 + end..];
                        continue;
                    }
                }
                // Block comment /* */
                if let Some(r) = rest.strip_prefix("/*") {
                    if let Some(close) = r.find("*/") {
                        buf.push_str(&rest[..2 + close + 2]);
                        rest = &rest[2 + close + 2..];
                        continue;
                    }
                    return Err(SqlxError::Other("Unterminated block comment".to_string()));
                }

                // Conditional block start
                if let Some(r) = rest.strip_prefix("{{") {
                    if !buf.is_empty() {
                        branches.push(Ast::Raw(std::mem::take(&mut buf)));
                    }
                    let mut inner_branches = Vec::new();
                    let mut inner_placeholders = Vec::new();
                    rest = inner(
                        r,
                        &mut inner_placeholders,
                        &mut inner_branches,
                        positional_counter,
                        settings,
                    )?;
                    branches.push(Ast::ConditionalBlock {
                        branches: inner_branches,
                        required_placeholders: inner_placeholders,
                    });
                    continue;
                }

                // Conditional block end
                if let Some(r) = rest.strip_prefix("}}") {
                    if !buf.is_empty() {
                        branches.push(Ast::Raw(std::mem::take(&mut buf)));
                    }
                    return Ok(r);
                }

                if let Some(suffix) = rest.strip_prefix_word_ignore_ascii_case(&["PAGINATE"]) {
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
                        branches.push(Ast::Raw(format!("{buf} ")));
                        buf.clear();
                        placeholders_out.push(name.to_string());
                        branches.push(Ast::PaginateClause { placeholder: name });
                        rest = &rest[consumed_len..];
                        continue;
                    }
                }

                if settings.collapsible_in_enabled {
                    // NOT IN support (with or without parentheses)
                    if let Some(suffix) = rest.strip_prefix_word_ignore_ascii_case(&["NOT", "IN"]) {
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
                                consumed_len = offset + 2;
                                name_opt = Some(positional_counter.to_string());
                            }
                        }
                        if let Some(name) = name_opt {
                            buf.trim_end_in_place();
                            if let Some((left, expr)) = buf.rsplit_once(char::is_whitespace) {
                                if !left.is_empty() {
                                    branches.push(Ast::Raw(format!("{left} ")));
                                }
                                branches.push(Ast::NotInClause {
                                    expr: expr.to_string(),
                                    placeholder: name,
                                });
                            } else {
                                branches.push(Ast::NotInClause {
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
                    if let Some(rest_after_in) = rest.strip_prefix_word_ignore_ascii_case(&["IN"]) {
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
                        } else if let Some(stripped) = rest_after_in.strip_prefix("(")
                            && let Some(close_idx) = stripped.find(')')
                        {
                            let inside = &stripped[..close_idx].trim();
                            if let Some(id) = inside.strip_prefix(':') {
                                name_opt = Some(id.to_string());
                            } else if let Some(id) = inside.strip_prefix('$') {
                                name_opt = Some(id.to_string());
                            } else if *inside == "?" {
                                *positional_counter += 1;
                                name_opt = Some(positional_counter.to_string());
                            }
                            consumed_len = original_len - rest_after_in.len() + 1 + close_idx + 1;
                        }

                        if let Some(name) = name_opt {
                            buf.trim_end_in_place();
                            if let Some((left, expr)) = buf.rsplit_once(char::is_whitespace) {
                                if !left.is_empty() {
                                    branches.push(Ast::Raw(format!("{left} ")));
                                }
                                branches.push(Ast::InClause {
                                    expr: expr.to_string(),
                                    placeholder: name,
                                });
                            } else {
                                branches.push(Ast::InClause {
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
                if let Some(after) = rest.strip_prefix(":")
                    && let Some((name, rem)) = after
                        .char_indices()
                        .take_while(|(_, c)| c.is_alphanumeric() || *c == '_')
                        .map(|(i, _)| i)
                        .last()
                        .map(|i| after.split_at(i + 1))
                {
                    if !buf.is_empty() {
                        branches.push(Ast::Raw(std::mem::take(&mut buf)));
                    }
                    branches.push(Ast::Placeholder(name.to_string()));
                    placeholders_out.push(name.to_string());
                    rest = rem;
                    continue;
                }

                // --- $named placeholder ---
                if let Some(after) = rest.strip_prefix("$")
                    && let Some((name, rem)) = after
                        .char_indices()
                        .take_while(|(_, c)| c.is_alphanumeric() || *c == '_')
                        .map(|(i, _)| i)
                        .last()
                        .map(|i| after.split_at(i + 1))
                {
                    if !buf.is_empty() {
                        branches.push(Ast::Raw(std::mem::take(&mut buf)));
                    }
                    branches.push(Ast::Placeholder(name.to_string()));
                    placeholders_out.push(name.to_string());
                    rest = rem;
                    continue;
                }

                // --- ? positional placeholder ---
                if let Some(r) = rest.strip_prefix("?") {
                    if !buf.is_empty() {
                        branches.push(Ast::Raw(std::mem::take(&mut buf)));
                    }
                    *positional_counter += 1;
                    let name = positional_counter.to_string();
                    branches.push(Ast::Placeholder(name.clone()));
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
                branches.push(Ast::Raw(buf));
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
            settings,
        )?;
        if !rest.trim().is_empty() {
            return Err(SqlxError::Other("Unmatched `{{` or extra trailing content".to_string()));
        }

        Ok(Ast::Root {
            branches,
            required_placeholders: placeholders,
        })
    }

    /// Transforms the AST into a SQL string using only named placeholders, replacing all positional
    /// placeholders (`?`, `:1`, `$1`) with unique names if they conflict with existing `placeholders`.
    ///
    /// Named placeholders that do not conflict will be preserved.
    /// Also extends the `parameters_bucket` with values from `parameters`, accounting for
    /// renaming of positional or conflicting placeholders.
    pub fn merge_into<I, K, V>(
        &self,
        placeholders: &mut BTreeSet<String>,
        parameters: I,
        parameters_bucket: &mut ParamsMap,
    ) -> crate::error::Result<String>
    where
        I: IntoIterator<Item = (K, V)> + Debug,
        K: Into<String>,
        V: Into<ParameterValue>,
    {
        fn walk(
            node: &Ast,
            sql: &mut String,
            placeholders: &mut BTreeSet<String>,
            param_map: &mut ParamsMap,
            parameters_bucket: &mut ParamsMap,
            index: &mut usize,
        ) -> crate::error::Result<()> {
            match node {
                Ast::Root { branches, .. }
                | Ast::Nested(branches)
                | Ast::ConditionalBlock { branches, .. } => {
                    for b in branches {
                        walk(b, sql, placeholders, param_map, parameters_bucket, index)?;
                    }
                }
                Ast::Raw(s) => sql.push_str(s),
                Ast::Placeholder(name) => {
                    let new_name = resolve_placeholder_name(name, placeholders, index);
                    if let Some(value) = param_map.remove(name) {
                        parameters_bucket.insert(new_name.clone(), value);
                    }
                    write!(sql, ":{new_name}")?;
                }
                Ast::InClause { expr, placeholder } | Ast::NotInClause { expr, placeholder } => {
                    let new_name = resolve_placeholder_name(placeholder, placeholders, index);
                    if let Some(value) = param_map.remove(placeholder) {
                        parameters_bucket.insert(new_name.clone(), value);
                    }
                    let keyword = if matches!(node, Ast::InClause { .. }) {
                        "IN"
                    } else {
                        "NOT IN"
                    };
                    write!(sql, "{expr} {keyword} (:{new_name})")?;
                }
                Ast::PaginateClause { placeholder } => {
                    let new_name = resolve_placeholder_name(placeholder, placeholders, index);
                    if let Some(value) = param_map.remove(placeholder) {
                        parameters_bucket.insert(new_name.clone(), value);
                    }
                    write!(sql, "PAGINATE :{new_name}")?; // placeholder logic
                }
            }
            Ok(())
        }

        fn resolve_placeholder_name(
            original: &str,
            placeholders: &mut BTreeSet<String>,
            index: &mut usize,
        ) -> String {
            if placeholders.contains(original) {
                loop {
                    let candidate = format!("{original}_auto_{index}");
                    *index += 1;
                    if placeholders.insert(candidate.clone()) {
                        break candidate;
                    }
                }
            } else {
                placeholders.insert(original.to_string());
                original.to_string()
            }
        }

        let mut sql = String::new();
        let mut index = 1;
        let mut param_map: ParamsMap = parameters
            .into_iter()
            .map(|(k, v)| {
                let mut k = k.into();
                if let Ok(n) = k.parse::<u32>() {
                    k = (n + 1).to_string();
                }
                (k, v.into())
            })
            .collect();

        walk(
            self,
            &mut sql,
            placeholders,
            &mut param_map,
            parameters_bucket,
            &mut index,
        )?;

        Ok(sql)
    }

    /// Render the AST into a finalized SQL string with positional or named
    /// placeholders, expanding arrays for `IN`/`NOT IN` clauses and
    /// injecting `LIMIT/OFFSET` for pagination.
    ///
    /// # Type Parameters
    ///
    /// * `I` – An iterable of key/value pairs supplying placeholder values.
    /// * `K` – Key type, convertible to `String` (placeholder name or index).
    /// * `V` – Value type, convertible to `ParameterValue`.
    ///
    /// # Arguments
    ///
    /// * `values` – An iterator of `(key, value)` pairs for all placeholders.
    /// * `settings` – Controls identifier quoting and placeholder style.
    ///
    /// # Returns
    ///
    /// A tuple `(sql, params)` where:
    /// - `sql` is the whitespace-normalized SQL string with `?`, `$n`, or `@pn`.
    /// - `params` is the vector of `ParameterValue` in order of appearance.
    ///
    /// # Errors
    ///
    /// Returns an error if a required placeholder is missing, or if a
    /// `PaginateClause` is provided with an incorrect value type.
    pub fn render<I, K, V>(
        &self,
        parameters: I,
        settings: &Settings,
    ) -> crate::error::Result<(String, Vec<ParameterValue>)>
    where
        I: IntoIterator<Item = (K, V)> + Debug,
        K: Into<String>,
        V: Into<ParameterValue>,
    {
        fn walk(
            node: &Ast,
            values: &ParamsMap,
            sql: &mut String,
            out_vals: &mut Vec<ParameterValue>,
            settings: &Settings,
        ) -> crate::error::Result<()> {
            match node {
                Ast::Root { branches, .. } | Ast::Nested(branches) => {
                    for n in branches {
                        walk(n, values, sql, out_vals, settings)?;
                    }
                }
                Ast::Raw(s) => sql.push_str(s),
                Ast::Placeholder(name) => {
                    #[cfg(test)]
                    {
                        println!("values = {values:?}");
                        println!("{name:?} ==> {:?}", values.get(name));
                    }
                    if let Some(val) = values.get(name) {
                        val.write_sql_to(sql, out_vals, settings)?;
                    }
                }
                Ast::ConditionalBlock {
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
                            walk(b, values, sql, out_vals, settings)?;
                        }
                    }
                }

                Ast::InClause { expr, placeholder } => match values.get(placeholder) {
                    Some(ParameterValue::Array(arr)) if !arr.is_empty() => {
                        sql.push_str(expr);
                        sql.push_str(" IN (");
                        arr.write_sql_to(sql, out_vals, settings)?;
                        sql.push(')');
                    }
                    _ => {
                        write!(sql, "FALSE /* {expr} IN :{placeholder} */").unwrap();
                    }
                },
                Ast::NotInClause { expr, placeholder } => match values.get(placeholder) {
                    Some(ParameterValue::Array(arr)) if !arr.is_empty() => {
                        sql.reserve(expr.len() + 9 + arr.len() * 2 + (arr.len() - 1) * 2);
                        write!(sql, "{expr} NOT IN (")?;
                        arr.write_sql_to(sql, out_vals, settings)?;
                        sql.push(')');
                    }
                    _ => {
                        write!(sql, "TRUE /* {expr} NOT IN :{placeholder} */")?;
                    }
                },
                Ast::PaginateClause { placeholder } => {
                    let value = values.get(placeholder);
                    match value {
                        Some(ParameterValue::PaginateClauseRendered(rendered)) => {
                            rendered.write_sql_to(sql, out_vals, settings)?;
                        }
                        _ => {
                            return Err(SqlxError::Other(format!(
                                "PAGINATE accepts only Sqlx\\PaginateClause instance, given: {placeholder:?} = {value:?}"
                            )));
                        }
                    }
                }
            }
            Ok(())
        }
        #[cfg(test)]
        {
            println!("AST = {self:?}");
            println!("VALUES = {parameters:?}");
        }
        let values: ParamsMap = parameters
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

        if let Ast::Root {
            required_placeholders,
            ..
        } = self
            && let Some(missing_placeholder) = required_placeholders.iter().find(|&ph| {
                if let Some(value) = values.get(ph) {
                    value.is_empty()
                } else {
                    true
                }
            })
        {
            return Err(SqlxError::MissingPlaceholder { name: missing_placeholder.clone() });
        }
        walk(self, &values, &mut sql, &mut out_vals, settings)?;
        #[cfg(test)]
        println!("SQL = {sql}");
        Ok((sql, out_vals))
    }
}
