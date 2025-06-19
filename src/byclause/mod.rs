use crate::utils::is_valid_ident;
use anyhow::bail;
use ext_php_rs::{ZvalConvert, php_class, php_impl};
use std::collections::HashMap;
use trim_in_place::TrimInPlace;

#[php_class(name = "Sqlx\\ByClause")]
pub struct ByClause {
    pub(crate) defined_fields: HashMap<String, Option<String>>,
}

#[derive(ZvalConvert, Debug)]
pub enum ByClauseFieldDefinition {
    Full(Vec<String>),
    Short(String),
}
impl ByClause {
    /// Ascending order (A to Z)
    pub const _ASC: &'static str = "ASC";
    /// Descending order (Z to A)
    pub const _DESC: &'static str = "DESC";
}

impl ByClause {
    pub fn new<K, V>(defined_fields: impl IntoIterator<Item = (K, V)>) -> anyhow::Result<Self>
    where
        K: Into<String>,
        V: Into<String>,
    {
        Ok(Self {
            defined_fields: defined_fields.into_iter().try_fold(
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
}

#[php_impl]
impl ByClause {
    /// Ascending order (A to Z)
    const ASC: &'static str = "ASC";
    /// Descending order (Z to A)
    const DESC: &'static str = "DESC";

    /// Constructs an ByClause helper with allowed sortable fields.
    ///
    /// # Arguments
    /// - `defined_fields`: Map of allowed sort fields (key = user input, value = SQL expression)
    ///
    /// # Example
    /// ```php
    /// $order_by = new Sqlx\ByClause([
    ///     "name",
    ///     "age",
    ///     "total_posts" => "COUNT(posts.*)"
    /// ]);
    /// ```

    pub fn __construct(defined_fields: HashMap<String, String>) -> anyhow::Result<Self> {
        ByClause::new(defined_fields)
    }

    /// __invoke magic for apply()

    #[must_use]
    pub fn __invoke(&self, order_by: Vec<ByClauseFieldDefinition>) -> ByClauseRendered {
        self.internal_apply(order_by)
    }

    /// Applies ordering rules to a user-defined input.
    ///
    /// # Arguments
    /// - `order_by`: List of fields (as strings or [field, direction] arrays)
    ///
    /// # Returns
    /// A `RenderedByClause` object containing validated SQL ORDER BY clauses
    /// The returning value is to be used as a placeholder value
    ///
    /// # Exceptions
    /// This method does not return an error but silently ignores unknown fields.
    /// Use validation separately if strict input is required.
    #[must_use]
    pub fn apply(&self, order_by: Vec<ByClauseFieldDefinition>) -> ByClauseRendered {
        self.internal_apply(order_by)
    }
}
impl ByClause {
    #[must_use]
    pub fn internal_apply(&self, order_by: Vec<ByClauseFieldDefinition>) -> ByClauseRendered {
        ByClauseRendered {
            __inner: order_by
                .into_iter()
                .filter_map(|definition| {
                    let (mut field, descending_order) = match definition {
                        ByClauseFieldDefinition::Short(name) => (name, false),
                        ByClauseFieldDefinition::Full(vec) => (
                            vec.first()?.clone(),
                            matches!(vec.get(1), Some(str) if str.trim().eq_ignore_ascii_case(Self::_DESC))
                        ),
                    };
                    self.defined_fields
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
/// A rendered ORDER BY clause result for use in query generation.
#[derive(Clone, PartialEq, Debug, ZvalConvert)]
pub struct ByClauseRendered {
    // @TODO: make it impossible to alter RenderedByClause from PHP side
    pub(crate) __inner: Vec<ByClauseRenderedField>,
}

#[derive(Clone, PartialEq, Debug, ZvalConvert)]
pub struct ByClauseRenderedField {
    pub(crate) expression_or_identifier: String,
    pub(crate) is_expression: bool,
    pub(crate) descending_order: bool,
}

impl ByClauseRendered {
    #[must_use]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    pub(crate) fn is_empty(&self) -> bool {
        self.__inner.is_empty()
    }
}
