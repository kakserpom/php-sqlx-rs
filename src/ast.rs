use crate::byclause::ByClauseRenderedField;
use crate::paramvalue::{ParameterValue, ParamsMap, Placeholder};
use crate::selectclause::SelectClauseRenderedColumn;
use crate::utils::StripPrefixWordIgnoreAsciiCase;
use anyhow::bail;
use itertools::Itertools;
use std::fmt::Debug;
use std::fmt::Write;
use trim_in_place::TrimInPlace;

/// An Abstract Syntax Tree representation of a parameterized SQL query,
/// allowing for nested conditional blocks, placeholders, and special clauses.
#[derive(Debug, PartialEq, Clone)]
pub enum Ast {
    /// A sequence of AST nodes, used for nesting and grouping.
    Nested(Vec<Ast>),

    /// A literal fragment of SQL text.
    Sql(String),

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
pub struct ParsingSettings {
    /// Enable special parsing for `IN` and `NOT IN` clauses.
    pub collapsible_in_enabled: bool,
    /// Treat `''` inside single-quoted strings as an escaped quote.
    pub escaping_double_single_quotes: bool,
    /// Recognize `#` as the start of a line comment.
    pub comment_hash: bool,
}

/// Settings that determine how placeholders and identifiers are rendered:
/// whether to use backticks, dollar-sign placeholders, `@`-style, etc.
pub struct RenderingSettings {
    /// Wrap column names in backticks instead of double quotes.
    pub column_backticks: bool,
    /// Render placeholders as `$1`, `$2`, ...
    pub placeholder_dollar_sign: bool,
    /// Render placeholders as `@p1`, `@p2`, ...
    pub placeholder_at_sign: bool,
}


impl Ast {
    /// Parse an SQL string with embedded optional blocks `{{ ... }}`, named
    /// placeholders (`$name`, `:name`), and positional `?` markers. Ignores
    /// matches inside string literals and comments.
    ///
    /// # Arguments
    ///
    /// * `input` – The raw SQL text to parse.
    /// * `parsing_settings` – Controls comment syntax, escaping, and `IN` support.
    ///
    /// # Returns
    ///
    /// An `Ast::Root` containing the parsed branches and the list of
    /// placeholders required to render the final SQL.
    pub fn parse(input: &str, parsing_settings: &ParsingSettings) -> anyhow::Result<Ast> {
        fn inner<'s>(
            mut rest: &'s str,
            placeholders_out: &mut Vec<String>,
            branches: &mut Vec<Ast>,
            positional_counter: &mut usize,
            parsing_settings: &ParsingSettings,
        ) -> anyhow::Result<&'s str> {
            let mut buf = String::new();

            while !rest.is_empty() {
                if rest.starts_with('\'') {
                    let mut idx = 1;
                    let mut iter = rest.char_indices().skip(1).peekable();
                    while let Some((i, c)) = iter.next() {
                        if parsing_settings.escaping_double_single_quotes {
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
                if parsing_settings.comment_hash {
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
                    bail!("Unterminated block comment");
                }

                // Conditional block start
                if let Some(r) = rest.strip_prefix("{{") {
                    if !buf.is_empty() {
                        branches.push(Ast::Sql(std::mem::take(&mut buf)));
                    }
                    let mut inner_branches = Vec::new();
                    let mut inner_placeholders = Vec::new();
                    rest = inner(
                        r,
                        &mut inner_placeholders,
                        &mut inner_branches,
                        positional_counter,
                        parsing_settings,
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
                        branches.push(Ast::Sql(std::mem::take(&mut buf)));
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
                        branches.push(Ast::Sql(format!("{buf} ")));
                        buf.clear();
                        placeholders_out.push(name.to_string());
                        branches.push(Ast::PaginateClause { placeholder: name });
                        rest = &rest[consumed_len..];
                        continue;
                    }
                }

                if parsing_settings.collapsible_in_enabled {
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
                                    branches.push(Ast::Sql(format!("{left} ")));
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
                                    branches.push(Ast::Sql(format!("{left} ")));
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
                if let Some(after) = rest.strip_prefix(":") {
                    if let Some((name, rem)) = after
                        .char_indices()
                        .take_while(|(_, c)| c.is_alphanumeric() || *c == '_')
                        .map(|(i, _)| i)
                        .last()
                        .map(|i| after.split_at(i + 1))
                    {
                        if !buf.is_empty() {
                            branches.push(Ast::Sql(std::mem::take(&mut buf)));
                        }
                        branches.push(Ast::Placeholder(name.to_string()));
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
                            branches.push(Ast::Sql(std::mem::take(&mut buf)));
                        }
                        branches.push(Ast::Placeholder(name.to_string()));
                        placeholders_out.push(name.to_string());
                        rest = rem;
                        continue;
                    }
                }

                // --- ? positional placeholder ---
                if let Some(r) = rest.strip_prefix("?") {
                    if !buf.is_empty() {
                        branches.push(Ast::Sql(std::mem::take(&mut buf)));
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
                branches.push(Ast::Sql(buf));
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
            parsing_settings,
        )?;
        if !rest.trim().is_empty() {
            bail!("Unmatched `{{` or extra trailing content");
        }

        Ok(Ast::Root {
            branches,
            required_placeholders: placeholders,
        })
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
    /// * `rendering_settings` – Controls identifier quoting and placeholder style.
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
        values: I,
        rendering_settings: &RenderingSettings,
    ) -> anyhow::Result<(String, Vec<ParameterValue>)>
    where
        I: IntoIterator<Item = (K, V)> + Debug,
        K: Into<String>,
        V: Into<ParameterValue>,
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
            out_vals: &mut Vec<ParameterValue>,
            rendering_settings: &RenderingSettings,
        ) -> anyhow::Result<()> {
            match node {
                Ast::Root { branches, .. } | Ast::Nested(branches) => {
                    for n in branches {
                        walk(n, values, sql, out_vals, rendering_settings)?;
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
                            ParameterValue::SelectClauseRendered(columns) => {
                                for (i, SelectClauseRenderedColumn { column: field, expression }) in
                                    columns.__inner.iter().enumerate()
                                {
                                    if i > 0 {
                                        sql.push_str(", ");
                                    }
                                    if let Some(expression) = expression {
                                        if rendering_settings.column_backticks {
                                            write!(sql, "{expression} AS `{field}`")?;
                                        } else {
                                            write!(sql, "{expression} AS \"{field}\"")?;
                                        }
                                    } else if rendering_settings.column_backticks {
                                        write!(sql, "`{field}`")?;
                                    } else {
                                        write!(sql, "\"{field}\"")?;
                                    }
                                }
                            }
                            ParameterValue::ByClauseRendered(by) => {
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
                                    } else if rendering_settings.column_backticks {
                                        write!(sql, "`{expression_or_identifier}`")?;
                                    } else {
                                        write!(sql, "\"{expression_or_identifier}\"")?;
                                    }
                                    if *descending_order {
                                        sql.push_str(" DESC");
                                    }
                                }
                            }
                            ParameterValue::Array(arr) => {
                                for (i, item) in arr.iter().enumerate() {
                                    if i > 0 {
                                        sql.push_str(", ");
                                    }
                                    out_vals.push(item.clone());
                                    if rendering_settings.placeholder_dollar_sign {
                                        write!(sql, "${}", out_vals.len())?;
                                    } else if rendering_settings.placeholder_at_sign {
                                        write!(sql, "@p{}", out_vals.len())?;
                                    } else {
                                        sql.push('?');
                                    }
                                }
                            }
                            _ => {
                                out_vals.push(val.clone());
                                if rendering_settings.placeholder_dollar_sign {
                                    write!(sql, "${}", out_vals.len())?;
                                } else if rendering_settings.placeholder_at_sign {
                                    write!(sql, "@p{}", out_vals.len())?;
                                } else {
                                    sql.push('?');
                                }
                            }
                        }
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
                            walk(b, values, sql, out_vals, rendering_settings)?;
                        }
                    }
                }

                Ast::InClause { expr, placeholder } => match values.get(placeholder) {
                    Some(ParameterValue::Array(arr)) if !arr.is_empty() => {
                        sql.push_str(expr);
                        sql.push_str(" IN (");
                        for (i, item) in arr.iter().enumerate() {
                            if i > 0 {
                                sql.push_str(", ");
                            }
                            out_vals.push(item.clone());
                            if rendering_settings.placeholder_dollar_sign {
                                write!(sql, "${}", out_vals.len())?;
                            } else if rendering_settings.placeholder_at_sign {
                                write!(sql, "@p{}", out_vals.len())?;
                            } else {
                                sql.push('?');
                            }
                        }
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
                        for (i, item) in arr.iter().enumerate() {
                            if i > 0 {
                                sql.push_str(", ");
                            }
                            out_vals.push(item.clone());
                            if rendering_settings.placeholder_dollar_sign {
                                write!(sql, "${}", out_vals.len())?;
                            } else if rendering_settings.placeholder_at_sign {
                                write!(sql, "@p{}", out_vals.len())?;
                            } else {
                                sql.push('?');
                            }
                        }
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
                            out_vals.push(ParameterValue::Int(rendered.limit));
                            out_vals.push(ParameterValue::Int(rendered.offset));
                            if rendering_settings.placeholder_dollar_sign {
                                write!(
                                    sql,
                                    "LIMIT ${} OFFSET ${}",
                                    out_vals.len() - 1,
                                    out_vals.len()
                                )?;
                            } else if rendering_settings.placeholder_at_sign {
                                write!(
                                    sql,
                                    "LIMIT @p{} OFFSET @p{}",
                                    out_vals.len() - 1,
                                    out_vals.len()
                                )?;
                            } else {
                                sql.push_str("LIMIT ? OFFSET ?");
                            }
                        }
                        _ => {
                            bail!(
                                "PAGINATE accepts only Sqlx\\PaginateClause instance, given: {placeholder:?} = {value:?}"
                            );
                        }
                    }
                }
            }
            Ok(())
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

        if let Ast::Root {
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
        walk(self, &values, &mut sql, &mut out_vals, rendering_settings)?;
        let sql = sql.split_whitespace().join(" ");

        #[cfg(test)]
        println!("SQL = {sql}");
        Ok((sql, out_vals))
    }
}
