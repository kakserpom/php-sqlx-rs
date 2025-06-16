use ext_php_rs::{ZvalConvert, php_class, php_impl};
use std::collections::HashMap;

#[php_class(name = "Sqlx\\OrderBy")]
pub struct OrderBy {
    pub(crate) defined_fields: HashMap<String, Option<String>>,
}

#[derive(ZvalConvert, Debug)]
pub enum OrderFieldDefinition {
    Full(Vec<String>),
    Short(String),
}
impl OrderBy {
    /// Ascending order (A to Z)
    pub const _ASC: &'static str = "ASC";
    /// Descending order (Z to A)
    pub const _DESC: &'static str = "DESC";
}

impl OrderBy {
    pub fn new<K, V>(defined_fields: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            defined_fields: defined_fields
                .into_iter()
                .map(|(key, value)| {
                    let key: String = key.into();
                    let value: String = value.into();
                    if key.parse::<u32>().is_ok() {
                        (value, None)
                    } else {
                        (key, Some(value))
                    }
                })
                .collect(),
        }
    }
}

#[php_impl]
impl OrderBy {
    /// Ascending order (A to Z)
    const ASC: &'static str = "ASC";
    /// Descending order (Z to A)
    const DESC: &'static str = "DESC";

    /// Constructs an OrderBy helper with allowed sortable fields.
    ///
    /// # Arguments
    /// - `defined_fields`: Map of allowed sort fields (key = user input, value = SQL expression)
    ///
    /// # Example
    /// ```php
    /// $order_by = new Sqlx\OrderBy([
    ///     "name",
    ///     "age",
    ///     "total_posts" => "COUNT(posts.*)"
    /// ]);
    /// ```

    pub fn __construct(defined_fields: HashMap<String, String>) -> Self {
        OrderBy::new(defined_fields)
    }

    /// __invoke magic for apply()

    #[must_use]
    pub fn __invoke(&self, order_by: Vec<OrderFieldDefinition>) -> RenderedOrderBy {
        self.internal_apply(order_by)
    }

    /// Applies ordering rules to a user-defined input.
    ///
    /// # Arguments
    /// - `order_by`: List of fields (as strings or [field, direction] arrays)
    ///
    /// # Returns
    /// A `RenderedOrderBy` object containing validated SQL ORDER BY clauses
    /// The returning value is to be used as a placeholder value
    ///
    /// # Exceptions
    /// This method does not return an error but silently ignores unknown fields.
    /// Use validation separately if strict input is required.
    #[must_use]
    pub fn apply(&self, order_by: Vec<OrderFieldDefinition>) -> RenderedOrderBy {
        self.internal_apply(order_by)
    }
}
impl OrderBy {
    #[must_use]
    pub fn internal_apply(&self, order_by: Vec<OrderFieldDefinition>) -> RenderedOrderBy {
        RenderedOrderBy {
            __inner: order_by
                .into_iter()
                .filter_map(|definition| {
                    let (field, dir) = match definition {
                        OrderFieldDefinition::Short(name) => (name, OrderBy::ASC),
                        OrderFieldDefinition::Full(vec) => (
                            vec.first()?.clone(),
                            match vec.get(1) {
                                Some(str) if str.trim().eq_ignore_ascii_case("DESC") => {
                                    OrderBy::DESC
                                }
                                _ => OrderBy::ASC,
                            },
                        ),
                    };
                    let field = field.trim();
                    if let Some(definition) = self.defined_fields.get(field) {
                        if let Some(right_side) = definition {
                            Some(format!("{right_side} {dir}"))
                        } else {
                            Some(format!("`{field}` {dir}"))
                        }
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}
#[derive(ZvalConvert)]
pub struct ApplyOrderBy {
    inner: Vec<Vec<String>>,
}
/// A rendered ORDER BY clause result for use in query generation.
#[derive(Clone, PartialEq, Debug, ZvalConvert)]
pub struct RenderedOrderBy {
    // @TODO: make it impossible to alter RenderedOrderBy from PHP side
    pub(crate) __inner: Vec<String>,
}

impl RenderedOrderBy {
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.__inner.is_empty()
    }
}
