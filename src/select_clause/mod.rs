//! SELECT clause validation and rendering for php-sqlx.
//!
//! This module provides safe column selection that validates user input against
//! a whitelist of allowed columns. It prevents SQL injection by ensuring only
//! pre-approved columns can be selected.
//!
//! # PHP Usage
//!
//! ```php
//! // Define allowed columns
//! $select = new Sqlx\SelectClause([
//!     'id',
//!     'name',
//!     'email',
//!     'full_name' => "CONCAT(first_name, ' ', last_name)"
//! ]);
//!
//! // Apply user input (only allowed columns are included)
//! $rendered = $select->input(['id', 'name', 'unknown_column']);
//! // Result: only 'id' and 'name' are selected
//! ```

use crate::ast::Settings;
use crate::error::Error as SqlxError;
use crate::utils::ident::is_valid_ident;
use ext_php_rs::{ZvalConvert, php_class, php_impl, prelude::ModuleBuilder};
use itertools::Itertools;
use std::collections::BTreeMap;
use std::fmt::Write;
use trim_in_place::TrimInPlace;

/// Registers the `SelectClause` and `SelectClauseRendered` classes with the PHP module builder.
pub fn build(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<SelectClause>()
        .class::<SelectClauseRendered>()
}

/// The `Sqlx\\SelectClause` class manages a set of allowed
/// columns for SQL SELECT operations and provides methods
/// to render validated column clauses from user input.
#[php_class]
#[php(name = "Sqlx\\SelectClause")]
pub struct SelectClause {
    /// Mapping of allowed column names to optional SQL expressions.
    pub(crate) allowed_columns: BTreeMap<String, Option<String>>,
}

impl SelectClause {
    /// Creates a new `SelectClause` from an iterator of key/value pairs.
    ///
    /// Accepts items where the key is either an index (numeric string)
    /// or a column name. Numeric keys indicate simple column names without
    /// an expression; other keys map to provided SQL expressions.
    ///
    /// # Errors
    /// Returns an error if any provided expression is not a valid SQL identifier
    /// when the key is numeric.
    #[inline]
    pub fn _new<K, V>(
        allowed_columns: impl IntoIterator<Item = (K, V)>,
    ) -> crate::error::Result<Self>
    where
        K: Into<String>,
        V: Into<String>,
    {
        Ok(Self {
            allowed_columns: allowed_columns
                .into_iter()
                .map(|(key, value)| -> crate::error::Result<_> {
                    let key: String = key.into();
                    let value: String = value.into();
                    // Numeric keys mean value is the column name
                    if key.parse::<u32>().is_ok() {
                        if !is_valid_ident(&value) {
                            return Err(SqlxError::Other(format!("Invalid identifier: {value}")));
                        }
                        Ok((value, None))
                    } else {
                        // Key is column alias, value is SQL expression
                        Ok((key, Some(value)))
                    }
                })
                .try_collect()?,
        })
    }

    /// Internal helper that filters and maps provided column names
    /// into a `SelectClauseRendered` structure based on `allowed_columns`.
    #[must_use]
    pub fn render(&self, columns: Vec<String>) -> SelectClauseRendered {
        let rendered = columns
            .into_iter()
            .filter_map(|mut field| {
                // Trim whitespace and check if column is allowed
                let key = field.trim_in_place();
                self.allowed_columns
                    .get(key)
                    .map(|expr| SelectClauseRenderedColumn {
                        column: field,
                        expression: expr.clone(),
                        table_alias: None,
                    })
            })
            .collect();
        SelectClauseRendered { __inner: rendered }
    }
}

#[php_impl]
impl SelectClause {
    /// PHP constructor for `Sqlx\\SelectClause`.
    ///
    /// # Arguments
    /// - `allowed_columns`: Associative array of allowed columns:
    ///    - Numeric keys map to simple column names
    ///    - String keys map to SQL expressions
    ///
    /// # Example
    /// ```php
    /// $select = new Sqlx\\SelectClause([
    ///     "id",
    ///     "name",
    ///     "full_name" => "CONCAT(first, ' ', last)"
    /// ]);
    /// ```
    pub fn __construct(allowed_columns: BTreeMap<String, String>) -> crate::error::Result<Self> {
        Self::_new(allowed_columns)
    }
    /// Cnstructor for `Sqlx\\SelectClause`.
    ///
    /// # Arguments
    /// - `allowed_columns`: Associative array of allowed columns:
    ///    - Numeric keys map to simple column names
    ///    - String keys map to SQL expressions
    ///
    /// # Example
    /// ```php
    /// $select = new Sqlx\\SelectClause([
    ///     "id",
    ///     "name",
    ///     "full_name" => "CONCAT(first, ' ', last)"
    /// ]);
    /// ```
    pub fn allowed(allowed_columns: BTreeMap<String, String>) -> crate::error::Result<Self> {
        Self::_new(allowed_columns)
    }

    /// Magic `__invoke` method allowing the object to be
    /// used as a callable for rendering select clauses.
    #[must_use]
    pub fn __invoke(&self, columns: Vec<String>) -> SelectClauseRendered {
        self.render(columns)
    }

    /// Renders validated SELECT clause columns from user input.
    ///
    /// # Arguments
    /// - `columns`: List of column identifiers provided by user.
    ///
    /// # Returns
    /// A `SelectClauseRendered` containing only allowed columns.
    /// Unknown columns are silently ignored.
    #[must_use]
    pub fn input(&self, columns: Vec<String>) -> SelectClauseRendered {
        self.render(columns)
    }
}

/// The `SelectClauseRendered` struct holds validated
/// column clauses for SQL SELECT statements.
#[derive(Clone, PartialEq, Debug)]
#[php_class]
#[php(name = "Sqlx\\SelectClauseRendered")]
pub struct SelectClauseRendered {
    /// Inner list of rendered columns and optional expressions.
    pub(crate) __inner: Vec<SelectClauseRenderedColumn>,
}

/// Represents a single column clause with optional expression.
#[derive(Clone, PartialEq, Debug, ZvalConvert)]
pub struct SelectClauseRenderedColumn {
    /// The raw column identifier from user input.
    pub(crate) column: String,
    /// Optional SQL expression for the column.
    pub(crate) expression: Option<String>,
    /// Optional table alias qualifying the column, for multi-table hydration.
    /// When set (and no `expression`), renders as `alias."column" AS "alias.column"`.
    pub(crate) table_alias: Option<String>,
}

impl SelectClauseRendered {
    /// Builds a rendered clause from a plain list of column names.
    ///
    /// Each name becomes a quoted column with no SQL expression. Used to derive
    /// a `SELECT` list from a hydration target class's properties.
    #[must_use]
    pub(crate) fn from_columns(columns: impl IntoIterator<Item = String>) -> Self {
        Self {
            __inner: columns
                .into_iter()
                .map(|column| SelectClauseRenderedColumn {
                    column,
                    expression: None,
                    table_alias: None,
                })
                .collect(),
        }
    }

    /// Builds a rendered clause from `(alias, columns)` groups for multi-table
    /// hydration. Each column renders as `alias."column" AS "alias.column"`, so
    /// columns from different joined tables never collide in the result set.
    #[must_use]
    pub(crate) fn from_aliased_columns(
        groups: impl IntoIterator<Item = (String, Vec<String>)>,
    ) -> Self {
        let mut inner = Vec::new();
        for (alias, columns) in groups {
            for column in columns {
                inner.push(SelectClauseRenderedColumn {
                    column,
                    expression: None,
                    table_alias: Some(alias.clone()),
                });
            }
        }
        Self { __inner: inner }
    }

    /// Returns true if no valid columns were rendered.
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.__inner.is_empty()
    }

    #[inline]
    pub(crate) fn write_sql_to(
        &self,
        sql: &mut String,
        settings: &Settings,
    ) -> crate::error::Result<()> {
        for (
            i,
            SelectClauseRenderedColumn {
                column: field,
                expression,
                table_alias,
            },
        ) in self.__inner.iter().enumerate()
        {
            if i > 0 {
                sql.push_str(", ");
            }
            let backticks = settings.column_backticks;
            match (table_alias.as_deref(), expression.as_deref()) {
                // Alias-qualified column with an output alias: `t."col" AS "t.col"`.
                (Some(alias), None) => {
                    if backticks {
                        write!(sql, "{alias}.`{field}` AS `{alias}.{field}`")?;
                    } else {
                        write!(sql, "{alias}.\"{field}\" AS \"{alias}.{field}\"")?;
                    }
                }
                // Expression with an alias-prefixed output name, when an alias is set.
                (Some(alias), Some(expression)) => {
                    if backticks {
                        write!(sql, "{expression} AS `{alias}.{field}`")?;
                    } else {
                        write!(sql, "{expression} AS \"{alias}.{field}\"")?;
                    }
                }
                // Plain expression aliased to the column name.
                (None, Some(expression)) => {
                    if backticks {
                        write!(sql, "{expression} AS `{field}`")?;
                    } else {
                        write!(sql, "{expression} AS \"{field}\"")?;
                    }
                }
                // Plain column.
                (None, None) => {
                    if backticks {
                        write!(sql, "`{field}`")?;
                    } else {
                        write!(sql, "\"{field}\"")?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Settings;

    fn render(clause: &SelectClauseRendered, backticks: bool) -> String {
        let mut sql = String::new();
        clause
            .write_sql_to(
                &mut sql,
                &Settings {
                    column_backticks: backticks,
                    ..Default::default()
                },
            )
            .expect("render");
        sql
    }

    #[test]
    fn plain_columns_render_quoted() {
        let clause = SelectClauseRendered::from_columns(["id".to_string(), "email".to_string()]);
        assert_eq!(render(&clause, false), r#""id", "email""#);
        assert_eq!(render(&clause, true), "`id`, `email`");
    }

    #[test]
    fn aliased_columns_are_qualified_and_output_aliased() {
        let clause = SelectClauseRendered::from_aliased_columns([
            ("o".to_string(), vec!["id".to_string(), "total".to_string()]),
            ("u".to_string(), vec!["email".to_string()]),
        ]);
        assert_eq!(
            render(&clause, false),
            r#"o."id" AS "o.id", o."total" AS "o.total", u."email" AS "u.email""#
        );
        assert_eq!(
            render(&clause, true),
            "o.`id` AS `o.id`, o.`total` AS `o.total`, u.`email` AS `u.email`"
        );
    }
}
