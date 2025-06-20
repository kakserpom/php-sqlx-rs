use crate::utils::is_valid_ident;
use anyhow::bail;
use ext_php_rs::{ZvalConvert, php_class, php_impl};
use std::collections::HashMap;
use trim_in_place::TrimInPlace;

#[php_class]
#[php(name = "Sqlx\\SelectClause")]
#[php(rename = "none")]
pub struct SelectClause {
    pub(crate) defined_fields: HashMap<String, Option<String>>,
}
impl SelectClause {
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
impl SelectClause {
    /// Constructs an SelectClause helper with allowed sortable fields.
    ///
    /// # Arguments
    /// - `defined_fields`: Map of allowed SELECT fields
    ///
    /// # Example
    /// ```php
    /// $order_by = new Sqlx\SelectClause([
    ///     "name",
    ///     "age",
    ///     "department_name" => "dp.name"
    /// ]);
    /// ```

    pub fn __construct(defined_fields: HashMap<String, String>) -> anyhow::Result<Self> {
        SelectClause::new(defined_fields)
    }

    /// __invoke magic for apply()

    #[must_use]
    pub fn __invoke(&self, order_by: Vec<String>) -> SelectClauseRendered {
        self.internal_apply(order_by)
    }

    /// Applies rules to a user-defined input.
    ///
    /// # Arguments
    /// - `fields`: List of fields
    ///
    /// # Returns
    /// A `RenderedSelectClause` object containing validated SQL SELECT clauses
    /// The returning value is to be used as a placeholder value
    ///
    /// # Exceptions
    /// This method does not return an error but silently ignores unknown fields.
    /// Use validation separately if strict input is required.
    #[must_use]
    pub fn apply(&self, fields: Vec<String>) -> SelectClauseRendered {
        self.internal_apply(fields)
    }
}
impl SelectClause {
    #[must_use]
    pub fn internal_apply(&self, fields: Vec<String>) -> SelectClauseRendered {
        SelectClauseRendered {
            __inner: fields
                .into_iter()
                .filter_map(|mut field| {
                    self.defined_fields.get(field.trim_in_place()).map(|expr| {
                        SelectClauseRenderedField {
                            field,
                            expression: expr.clone(),
                        }
                    })
                })
                .collect(),
        }
    }
}
/// A rendered ORDER BY clause result for use in query generation.
#[derive(Clone, PartialEq, Debug, ZvalConvert)]
pub struct SelectClauseRendered {
    // @TODO: make it impossible to alter RenderedSelectClause from PHP side
    pub(crate) __inner: Vec<SelectClauseRenderedField>,
}
#[derive(Clone, PartialEq, Debug, ZvalConvert)]
pub struct SelectClauseRenderedField {
    pub(crate) field: String,
    pub(crate) expression: Option<String>,
}
impl SelectClauseRendered {
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.__inner.is_empty()
    }
}
