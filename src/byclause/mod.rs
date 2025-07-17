use crate::utils::is_valid_ident;
use anyhow::bail;
use ext_php_rs::builders::ModuleBuilder;
use ext_php_rs::{ZvalConvert, php_class, php_impl};
use std::collections::HashMap;
use std::fmt::Write;
use trim_in_place::TrimInPlace;
use crate::ast::Settings;

/// Represents a dynamic ORDER BY / GROUP BY clause generator.
///
/// This struct allows validating and mapping user input (e.g. from HTTP parameters)
/// to a known set of allowed sortable fields or SQL expressions.
///
/// It supports two modes:
/// - `"name"` (auto-mapped to `"name"`)
/// - `"posts" => "COUNT(posts.*)"` (maps user field to custom SQL)
///
/// Use with `ByClauseRendered` to safely inject into a query as a single placeholder.
#[php_class]
#[php(name = "Sqlx\\ByClause")]
pub struct ByClause {
    pub(crate) allowed_columns: HashMap<String, Option<String>>,
}

/// A user-defined ORDER BY column configuration.
#[derive(ZvalConvert, Debug)]
pub enum ByClauseColumnDefinition {
    /// Single field (implied ascending order)
    Full(Vec<String>),
    /// Explicit [field, order] pair
    Short(String),
}

impl ByClause {
    /// Ascending order (A to Z)
    pub const _ASC: &'static str = "ASC";
    /// Descending order (Z to A)
    pub const _DESC: &'static str = "DESC";
}

impl ByClause {
    /// Constructs a new `ByClause` from a list of valid columns.
    ///
    /// # Arguments
    /// - A key-value pair list:
    ///   - Key as column alias or numeric index
    ///   - Value as SQL identifier or expression
    ///
    /// # Errors
    /// Returns an error if a numeric key maps to an invalid SQL identifier.
    pub fn allowed<K, V>(allowed_columns: impl IntoIterator<Item = (K, V)>) -> anyhow::Result<Self>
    where
        K: Into<String>,
        V: Into<String>,
    {
        Ok(Self {
            allowed_columns: allowed_columns.into_iter().try_fold(
                HashMap::<String, Option<String>>::new(),
                |mut map, (key, value)| -> anyhow::Result<_> {
                    let key: String = key.into();
                    let value: String = value.into();
                    if key.parse::<u32>().is_ok() {
                        if !is_valid_ident(&value) {
                            bail!("Invalid identifier: {}", value);
                        }
                        map.insert(value, None);
                    } else {
                        map.insert(key, Some(value));
                    }
                    Ok(map)
                },
            )?,
        })
    }

    /// Internal method that transforms user-specified ordering into SQL-safe representations.
    ///
    /// # Arguments
    /// - A list of `ByClauseColumnDefinition`, which may be simple names or `[name, direction]` arrays.
    ///
    /// # Returns
    /// A `ByClauseRendered` containing a validated list of SQL clauses.
    ///
    /// Unknown or disallowed fields are ignored silently.
    #[must_use]
    pub fn render(&self, columns: Vec<ByClauseColumnDefinition>) -> ByClauseRendered {
        ByClauseRendered {
            __inner: columns
                .into_iter()
                .filter_map(|definition| {
                    let (mut field, descending_order) = match definition {
                        ByClauseColumnDefinition::Short(name) => (name, false),
                        ByClauseColumnDefinition::Full(vec) => (
                            vec.first()?.clone(),
                            matches!(vec.get(1), Some(str) if str.trim().eq_ignore_ascii_case(Self::_DESC))
                        ),
                    };
                    self.allowed_columns
                        .get(field.trim_in_place())
                        .map(|definition| {
                            if let Some(expression) = definition {
                                ByClauseRenderedField {
                                    expression_or_identifier: expression.clone(),
                                    is_expression: true,
                                    descending_order,
                                }
                            } else {
                                ByClauseRenderedField {
                                    expression_or_identifier: field,
                                    is_expression: false,
                                    descending_order,
                                }
                            }
                        })
                })
                .collect(),
        }
    }
}

#[php_impl]
impl ByClause {
    /// Ascending order (A to Z)
    const ASC: &'static str = "ASC";
    /// Descending order (Z to A)
    const DESC: &'static str = "DESC";

    /// Constructs a `ByClause` helper with allowed sortable columns.
    ///
    /// # Arguments
    /// - `allowed_columns`: Map of allowed sort columns (key = user input, value = SQL expression)
    ///
    /// # Example
    /// ```php
    /// $order_by = new Sqlx\ByClause([
    ///     "name",
    ///     "age",
    ///     "total_posts" => "COUNT(posts.*)"
    /// ]);
    /// ```
    pub fn __construct(allowed_columns: HashMap<String, String>) -> anyhow::Result<Self> {
        Self::allowed(allowed_columns)
    }

    /// `__invoke` magic for apply().
    #[must_use]
    pub fn __invoke(&self, columns: Vec<ByClauseColumnDefinition>) -> ByClauseRendered {
        self.render(columns)
    }

    /// Applies ordering rules to a user-defined input.
    ///
    /// # Arguments
    /// - `columns`: List of columns (as strings or [field, direction] arrays)
    ///
    /// # Returns
    /// A `ByClauseRendered` object containing validated SQL ORDER BY clauses.
    /// The resulting value is to be used as a placeholder in query bindings.
    ///
    /// # Notes
    /// Unknown or disallowed fields are silently ignored.
    #[must_use]
    pub fn input(&self, columns: Vec<ByClauseColumnDefinition>) -> ByClauseRendered {
        self.render(columns)
    }
}

/// A rendered ORDER BY / GROUP BY clause result for use in query generation.
#[derive(Clone, PartialEq, Debug)]
#[php_class]
#[php(name = "Sqlx\\ByClauseRendered")]
pub struct ByClauseRendered {
    pub(crate) __inner: Vec<ByClauseRenderedField>,
}

/// A single ORDER BY / GROUP BY element, either an identifier or SQL expression.
#[derive(Clone, PartialEq, Debug, ZvalConvert)]
pub struct ByClauseRenderedField {
    pub(crate) expression_or_identifier: String,
    pub(crate) is_expression: bool,
    pub(crate) descending_order: bool,
}

impl ByClauseRendered {
    /// Returns whether the rendered clause is empty.
    #[must_use]
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.__inner.is_empty()
    }

    #[inline]
    pub(crate) fn write_sql_to(
        &self,
        sql: &mut String,
        settings: &Settings,
    ) -> anyhow::Result<()> {
        for (
            i,
            ByClauseRenderedField {
                expression_or_identifier,
                is_expression,
                descending_order,
            },
        ) in self.__inner.iter().enumerate()
        {
            if i > 0 {
                sql.push_str(", ");
            }
            if *is_expression {
                sql.push_str(expression_or_identifier);
            } else if settings.column_backticks {
                write!(sql, "`{expression_or_identifier}`")?;
            } else {
                write!(sql, "\"{expression_or_identifier}\"")?;
            }
            if *descending_order {
                sql.push_str(" DESC");
            }
        }
        Ok(())
    }
}

/// Registers `ByClause` and `ByClauseRendered` with the PHP module.
pub fn build(module: ModuleBuilder) -> ModuleBuilder {
    module.class::<ByClause>().class::<ByClauseRendered>()
}
