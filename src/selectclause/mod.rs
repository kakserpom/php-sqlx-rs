use crate::utils::is_valid_ident;
use anyhow::bail;
use ext_php_rs::{ZvalConvert, php_class, php_impl, prelude::ModuleBuilder};
use std::collections::HashMap;
use trim_in_place::TrimInPlace;

/// The `Sqlx\\SelectClause` class manages a set of allowed
/// columns for SQL SELECT operations and provides methods
/// to render validated column clauses from user input.
#[php_class]
#[php(name = "Sqlx\\SelectClause")]
pub struct SelectClause {
    /// Mapping of allowed column names to optional SQL expressions.
    pub(crate) defined_columns: HashMap<String, Option<String>>,
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
    pub fn new<K, V>(
        defined_columns: impl IntoIterator<Item = (K, V)>
    ) -> anyhow::Result<Self>
    where
        K: Into<String>,
        V: Into<String>,
    {
        let map = defined_columns
            .into_iter()
            .try_fold(
                HashMap::<String, Option<String>>::new(),
                |mut map, (key, value)| -> anyhow::Result<_> {
                    let key: String = key.into();
                    let value: String = value.into();
                    // Numeric keys mean value is the column name
                    if key.parse::<u32>().is_ok() {
                        if !is_valid_ident(&value) {
                            bail!("Invalid identifier: {}", value);
                        }
                        map.insert(value, None);
                    } else {
                        // Key is column alias, value is SQL expression
                        map.insert(key, Some(value));
                    }
                    Ok(map)
                },
            )?;
        Ok(Self { defined_columns: map })
    }

    /// Internal helper that filters and maps provided column names
    /// into a `SelectClauseRendered` structure based on `defined_columns`.
    #[must_use]
    pub fn internal_apply(&self, columns: Vec<String>) -> SelectClauseRendered {
        let rendered = columns
            .into_iter()
            .filter_map(|mut field| {
                // Trim whitespace and check if column is allowed
                let key = field.trim_in_place();
                self.defined_columns.get(key).map(|expr| {
                    SelectClauseRenderedColumn {
                        column: field,
                        expression: expr.clone(),
                    }
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
    /// - `defined_columns`: Associative array of allowed columns:
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
    pub fn __construct(
        defined_columns: HashMap<String, String>
    ) -> anyhow::Result<Self> {
        Self::new(defined_columns)
    }

    /// Magic `__invoke` method allowing the object to be
    /// used as a callable for rendering select clauses.
    #[must_use]
    pub fn __invoke(
        &self,
        columns: Vec<String>
    ) -> SelectClauseRendered {
        self.internal_apply(columns)
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
    pub fn apply(
        &self,
        columns: Vec<String>
    ) -> SelectClauseRendered {
        self.internal_apply(columns)
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
}

impl SelectClauseRendered {
    /// Returns true if no valid columns were rendered.
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.__inner.is_empty()
    }
}

/// Registers the `SelectClause` and `SelectClauseRendered`
/// classes with the given PHP module builder.
pub fn build(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<SelectClause>()
        .class::<SelectClauseRendered>()
}
