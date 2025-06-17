use crate::is_valid_ident;
use anyhow::bail;
use ext_php_rs::{ZvalConvert, php_class, php_impl};
use std::collections::HashMap;

#[php_class(name = "Sqlx\\FieldsClause")]
pub struct FieldsClause {
    pub(crate) defined_fields: HashMap<String, Option<String>>,
}
impl FieldsClause {
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
impl FieldsClause {
    /// Constructs an FieldsClause helper with allowed sortable fields.
    ///
    /// # Arguments
    /// - `defined_fields`: Map of allowed SELECT fields
    ///
    /// # Example
    /// ```php
    /// $order_by = new Sqlx\FieldsClause([
    ///     "name",
    ///     "age",
    ///     "department_name" => "dp.name"
    /// ]);
    /// ```

    pub fn __construct(defined_fields: HashMap<String, String>) -> anyhow::Result<Self> {
        FieldsClause::new(defined_fields)
    }

    /// __invoke magic for apply()

    #[must_use]
    pub fn __invoke(&self, order_by: Vec<String>) -> RenderedFieldsClause {
        self.internal_apply(order_by)
    }

    /// Applies rules to a user-defined input.
    ///
    /// # Arguments
    /// - `fields`: List of fields
    ///
    /// # Returns
    /// A `RenderedFieldsClause` object containing validated SQL SELECT clauses
    /// The returning value is to be used as a placeholder value
    ///
    /// # Exceptions
    /// This method does not return an error but silently ignores unknown fields.
    /// Use validation separately if strict input is required.
    #[must_use]
    pub fn apply(&self, fields: Vec<String>) -> RenderedFieldsClause {
        self.internal_apply(fields)
    }
}
impl FieldsClause {
    #[must_use]
    pub fn internal_apply(&self, fields: Vec<String>) -> RenderedFieldsClause {
        RenderedFieldsClause {
            __inner: fields
                .into_iter()
                .filter_map(|field| {
                    let field = field.trim();
                    if let Some(definition) = self.defined_fields.get(field) {
                        if let Some(right_side) = definition {
                            Some(format!("{right_side} AS `{field}`"))
                        } else {
                            // @TODO: ` or "
                            Some(format!("{field}"))
                        }
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}
/// A rendered ORDER BY clause result for use in query generation.
#[derive(Clone, PartialEq, Debug, ZvalConvert)]
pub struct RenderedFieldsClause {
    // @TODO: make it impossible to alter RenderedFieldsClause from PHP side
    pub(crate) __inner: Vec<String>,
}
impl RenderedFieldsClause {
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.__inner.is_empty()
    }
}
