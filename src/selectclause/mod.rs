use crate::utils::is_valid_ident;
use anyhow::bail;
use ext_php_rs::{ZvalConvert, php_class, php_impl, prelude::ModuleBuilder};
use std::collections::HashMap;
use trim_in_place::TrimInPlace;

#[php_class]
#[php(name = "Sqlx\\SelectClause")]
#[php(rename = "none")]
pub struct SelectClause {
    pub(crate) defined_columns: HashMap<String, Option<String>>,
}
impl SelectClause {
    pub fn new<K, V>(defined_columns: impl IntoIterator<Item = (K, V)>) -> anyhow::Result<Self>
    where
        K: Into<String>,
        V: Into<String>,
    {
        Ok(Self {
            defined_columns: defined_columns.into_iter().try_fold(
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
    #[must_use]
    pub fn internal_apply(&self, columns: Vec<String>) -> SelectClauseRendered {
        SelectClauseRendered {
            __inner: columns
                .into_iter()
                .filter_map(|mut field| {
                    self.defined_columns.get(field.trim_in_place()).map(|expr| {
                        SelectClauseRenderedColumn {
                            column: field,
                            expression: expr.clone(),
                        }
                    })
                })
                .collect(),
        }
    }
}

#[php_impl]
impl SelectClause {
    /// Constructs an SelectClause helper with allowed sortable columns.
    ///
    /// # Arguments
    /// - `defined_columns`: Map of allowed SELECT columns
    ///
    /// # Example
    /// ```php
    /// $order_by = new Sqlx\SelectClause([
    ///     "name",
    ///     "age",
    ///     "department_name" => "dp.name"
    /// ]);
    /// ```
    pub fn __construct(defined_columns: HashMap<String, String>) -> anyhow::Result<Self> {
        Self::new(defined_columns)
    }

    /// __invoke magic for apply()
    #[must_use]
    pub fn __invoke(&self, order_by: Vec<String>) -> SelectClauseRendered {
        self.internal_apply(order_by)
    }

    /// Applies rules to a user-defined input.
    ///
    /// # Arguments
    /// - `columns`: List of columns
    ///
    /// # Returns
    /// A `RenderedSelectClause` object containing validated SQL SELECT clauses
    /// The returning value is to be used as a placeholder value
    ///
    /// # Exceptions
    /// This method does not return an error but silently ignores unknown columns.
    /// Use validation separately if strict input is required.
    #[must_use]
    pub fn apply(&self, columns: Vec<String>) -> SelectClauseRendered {
        self.internal_apply(columns)
    }
}
/// A rendered ORDER BY clause result for use in query generation.
#[derive(Clone, PartialEq, Debug)]
#[php_class]
#[php(name = "Sqlx\\SelectClauseRendered")]
#[php(rename = "none")]
pub struct SelectClauseRendered {
    // @TODO: make it impossible to alter RenderedSelectClause from PHP side
    pub(crate) __inner: Vec<SelectClauseRenderedColumn>,
}
#[derive(Clone, PartialEq, Debug, ZvalConvert)]
pub struct SelectClauseRenderedColumn {
    pub(crate) column: String,
    pub(crate) expression: Option<String>,
}
impl SelectClauseRendered {
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.__inner.is_empty()
    }
}

pub fn build(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<SelectClause>()
        .class::<SelectClauseRendered>()
}
