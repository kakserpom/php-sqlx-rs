use crate::error::Error as SqlxError;
#[cfg(feature = "lazy-row")]
use crate::lazy_row::{LazyRow, LazyRowJson};
use ext_php_rs::boxed::ZBox;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::ffi::zend_array;
use ext_php_rs::ffi::zend_object;
#[cfg(feature = "lazy-row")]
use ext_php_rs::types::ZendClassObject;
use crate::param_value::ParameterValue;
use crate::select_clause::SelectClauseRendered;
use ext_php_rs::ZvalConvert;
use ext_php_rs::types::{ArrayKey, Zval};
use ext_php_rs::zend::ClassEntry;
use sqlx_oldapi::Column;
use sqlx_oldapi::Row;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

/// Resolves a PHP class by name, erroring if it is not registered.
///
/// Used by the `query*Into(...)` hydration methods to map result rows into a
/// caller-supplied class.
pub(crate) fn resolve_class(name: &str) -> crate::error::Result<&'static ClassEntry> {
    ClassEntry::try_find(name).ok_or_else(|| SqlxError::Other(format!("Class `{name}` not found")))
}

/// Returns the declared property names of a class, in declaration order.
///
/// Reads the class's `properties_info` table, which lists every declared
/// property by its plain (unmangled) name regardless of visibility or whether a
/// typed property has been initialized — so a plain DTO like
/// `class User { public int $id; public string $email; }` yields `["id", "email"]`.
/// These are the same names [`Conversion::into_class_zval`] hydrates into.
///
/// Note: static properties are not filtered out; hydration DTOs are not expected
/// to declare them.
pub(crate) fn class_columns(ce: &ClassEntry) -> Vec<String> {
    ce.properties_info
        .iter()
        .map(|(key, _)| key.to_string())
        .collect()
}

/// The hydration target passed to `query*Into(...)` from PHP.
///
/// Either a single class name (each row becomes one instance of that class) or
/// an `alias => class` map (each row becomes a `stdClass` with one property per
/// alias, each holding an instance hydrated from that alias's columns).
#[derive(ZvalConvert)]
pub enum HydrationTarget {
    /// `queryAllInto(User::class, ...)` — rows hydrate directly into `User`.
    Single(String),
    /// `queryAllInto(['u' => User::class, 'o' => Order::class], ...)` — rows
    /// hydrate into a `stdClass { u: User, o: Order }`.
    Map(BTreeMap<String, String>),
}

/// A [`HydrationTarget`] with its class names resolved to class entries.
pub enum ResolvedTarget {
    /// Single class; rows hydrate directly into it.
    Single(&'static ClassEntry),
    /// Ordered `(alias, class)` pairs; rows hydrate into an alias-keyed `stdClass`.
    Map(Vec<(String, &'static ClassEntry)>),
}

/// Resolves every class name in a [`HydrationTarget`], erroring if any is unknown.
pub(crate) fn resolve_target(target: &HydrationTarget) -> crate::error::Result<ResolvedTarget> {
    Ok(match target {
        HydrationTarget::Single(class) => ResolvedTarget::Single(resolve_class(class)?),
        HydrationTarget::Map(map) => {
            let mut aliases = Vec::with_capacity(map.len());
            for (alias, class) in map {
                aliases.push((alias.clone(), resolve_class(class)?));
            }
            ResolvedTarget::Map(aliases)
        }
    })
}

/// Injects a `select` parameter derived from the target's class(es) when the
/// query references the `:select` / `$select` placeholder and the caller has not
/// already provided one.
///
/// This lets `query*Into(...)` write `SELECT :select FROM ...` and have the
/// column list filled in from the target instead of being enumerated by hand.
/// For an alias map each column is qualified and output-aliased
/// (`o."id" AS "o.id"`) so columns from different tables never collide.
pub(crate) fn with_target_select(
    target: &ResolvedTarget,
    query: &str,
    parameters: Option<BTreeMap<String, ParameterValue>>,
) -> Option<BTreeMap<String, ParameterValue>> {
    if !(query.contains(":select") || query.contains("$select")) {
        return parameters;
    }
    let mut parameters = parameters.unwrap_or_default();
    if !parameters.contains_key("select") {
        let rendered = match target {
            ResolvedTarget::Single(ce) => SelectClauseRendered::from_columns(class_columns(ce)),
            ResolvedTarget::Map(aliases) => SelectClauseRendered::from_aliased_columns(
                aliases
                    .iter()
                    .map(|(alias, ce)| (alias.clone(), class_columns(ce))),
            ),
        };
        parameters.insert(
            "select".to_string(),
            ParameterValue::SelectClauseRendered(rendered),
        );
    }
    Some(parameters)
}

/// Trait to convert a row into a PHP value.
pub trait Conversion: Row {
    /// Convert the row into a PHP value.
    ///
    /// When `lazy-row` feature is enabled, returns a `LazyRow` object that:
    /// - Extends `stdClass` (passes `instanceof stdClass`)
    /// - Implements `ArrayAccess` for `$row['column']` access
    /// - Implements `Iterator` for `foreach` loops
    /// - Implements `JsonSerializable` for `json_encode()`
    /// - Lazily decodes large JSON values via `__get`
    ///
    /// The `associative_arrays` parameter controls how nested JSON objects are decoded.
    fn into_zval(self, associative_arrays: bool) -> crate::error::Result<Zval>
    where
        Self: Sized,
    {
        let columns = self.columns();

        // Build the array with all column values
        #[cfg(feature = "lazy-row")]
        let mut has_lazy_json = false;

        let array = columns.iter().try_fold(
            zend_array::with_capacity(u32::try_from(columns.len())?),
            |mut array, column| -> crate::error::Result<ZBox<zend_array>> {
                let column_name = column.name();
                let value = self.column_value_into_zval(column, associative_arrays)?;

                // Check if the value contains a LazyRowJson (deferred JSON parsing)
                #[cfg(feature = "lazy-row")]
                if !has_lazy_json
                    && let Some(obj) = value.object()
                    && ZendClassObject::<LazyRowJson>::from_zend_obj(obj).is_some()
                {
                    has_lazy_json = true;
                }

                if !column_name.is_empty() && column_name != "?column?" {
                    array
                        .insert(column.name(), value)
                        .map_err(|err| SqlxError::Conversion {
                            message: format!("{err:?}"),
                        })?;
                } else {
                    array
                        .insert(i64::try_from(column.ordinal())?, value)
                        .map_err(|err| SqlxError::Conversion {
                            message: format!("{err:?}"),
                        })?;
                }
                Ok(array)
            },
        )?;

        // With lazy-row feature: always return LazyRow if there's lazy JSON,
        // otherwise return plain array (for assoc mode) or LazyRow (for object mode)
        #[cfg(feature = "lazy-row")]
        {
            // Return LazyRow if there's lazy JSON OR if we're in object mode
            // (LazyRow extends stdClass, so it works as an object)
            if has_lazy_json || !associative_arrays {
                return LazyRow::new(array)
                    .into_zval(false)
                    .map_err(|err| SqlxError::Conversion {
                        message: format!("{err:?}"),
                    });
            }
            // No lazy JSON and array mode requested - return plain array
            array.into_zval(false).map_err(|err| SqlxError::Conversion {
                message: format!("{err:?}"),
            })
        }

        // Without lazy-row feature: return array or stdClass based on mode
        #[cfg(not(feature = "lazy-row"))]
        {
            if associative_arrays {
                array.into_zval(false).map_err(|err| SqlxError::Conversion {
                    message: format!("{err:?}"),
                })
            } else {
                // Convert array to stdClass
                let object = array.iter().try_fold(
                    zend_object::new_stdclass(),
                    |mut object: ZBox<zend_object>, (key, value)| {
                        let prop_name = match key {
                            ArrayKey::Long(n) => format!("_{n}"),
                            ArrayKey::String(s) => s,
                            ArrayKey::Str(s) => s.to_string(),
                        };
                        object
                            .set_property(&prop_name, value.shallow_clone())
                            .map(|()| object)
                            .map_err(|err| SqlxError::Conversion {
                                message: format!("{err:?}"),
                            })
                    },
                )?;
                object
                    .into_zval(false)
                    .map_err(|err| SqlxError::Conversion {
                        message: format!("{err:?}"),
                    })
            }
        }
    }

    /// Converts a row into an instance of the PHP class `ce`.
    ///
    /// Each column is assigned to a public property of the same name on a fresh
    /// instance built with [`ext_php_rs::types::ZendObject::new`] — the
    /// constructor is **not** invoked (matching PDO's `FETCH_CLASS` behavior).
    /// Columns without a usable name fall back to an `_{ordinal}` property.
    ///
    /// `associative_arrays` controls how nested JSON values inside columns are
    /// decoded, exactly as in [`Conversion::into_zval`].
    fn into_class_zval(
        self,
        ce: &ClassEntry,
        associative_arrays: bool,
    ) -> crate::error::Result<Zval>
    where
        Self: Sized,
    {
        let mut object = zend_object::new(ce);
        for column in self.columns() {
            let column_name = column.name();
            let value = self.column_value_into_zval(column, associative_arrays)?;
            let prop_name = if !column_name.is_empty() && column_name != "?column?" {
                Cow::Borrowed(column_name)
            } else {
                Cow::Owned(format!("_{}", column.ordinal()))
            };
            object
                .set_property(&prop_name, value)
                .map_err(|err| SqlxError::Conversion {
                    message: format!("{err:?}"),
                })?;
        }
        object.into_zval(false).map_err(|err| SqlxError::Conversion {
            message: format!("{err:?}"),
        })
    }

    /// Converts a row according to a [`ResolvedTarget`].
    ///
    /// Dispatches to [`Conversion::into_class_zval`] for a single class, or
    /// [`Conversion::into_aliased_row_zval`] for an alias map.
    fn into_target_zval(
        self,
        target: &ResolvedTarget,
        associative_arrays: bool,
    ) -> crate::error::Result<Zval>
    where
        Self: Sized,
    {
        match target {
            ResolvedTarget::Single(ce) => self.into_class_zval(ce, associative_arrays),
            ResolvedTarget::Map(aliases) => self.into_aliased_row_zval(aliases, associative_arrays),
        }
    }

    /// Converts a row into a `stdClass` with one property per alias.
    ///
    /// `:select` emits columns named `alias.column` (e.g. `o.id`), so each alias's
    /// instance is built from the columns carrying its `alias.` prefix (the prefix
    /// is stripped to recover the property name). Columns matching no alias prefix
    /// — e.g. hand-written extras — are ignored.
    fn into_aliased_row_zval(
        self,
        aliases: &[(String, &'static ClassEntry)],
        associative_arrays: bool,
    ) -> crate::error::Result<Zval>
    where
        Self: Sized,
    {
        let columns = self.columns();
        let mut row = zend_object::new_stdclass();
        for (alias, ce) in aliases {
            let prefix = format!("{alias}.");
            let mut instance = zend_object::new(ce);
            for column in columns {
                if let Some(prop_name) = column.name().strip_prefix(prefix.as_str()) {
                    let value = self.column_value_into_zval(column, associative_arrays)?;
                    instance
                        .set_property(prop_name, value)
                        .map_err(|err| SqlxError::Conversion {
                            message: format!("{err:?}"),
                        })?;
                }
            }
            let instance = instance.into_zval(false).map_err(|err| SqlxError::Conversion {
                message: format!("{err:?}"),
            })?;
            row.set_property(alias, instance)
                .map_err(|err| SqlxError::Conversion {
                    message: format!("{err:?}"),
                })?;
        }
        row.into_zval(false).map_err(|err| SqlxError::Conversion {
            message: format!("{err:?}"),
        })
    }

    /// Converts a specific column from a row to a PHP value.
    ///
    /// # Arguments
    /// - `column`: Reference to the column in the row.
    /// - `associative_arrays`: Whether to render complex types as associative arrays
    ///
    /// # Returns
    /// A PHP-compatible `Zval` value
    fn column_value_into_zval<C, D>(
        &self,
        column: &C,
        associative_arrays: bool,
    ) -> crate::error::Result<Zval>
    where
        C: Column<Database = D>;

    fn column_value_into_array_key<'a, C, D>(
        &self,
        column: &C,
    ) -> crate::error::Result<ArrayKey<'a>>
    where
        C: Column<Database = D>;
}

/// Converts a JSON value into a PHP value, respecting associative array settings.
///
/// # Arguments
/// - `value`: A `serde_json::Value` to convert
/// - `associative_arrays`: Whether to convert objects into PHP associative arrays or `stdClass`
///
/// # Returns
/// Converted `Zval` or an error if conversion fails
pub(crate) fn json_into_zval(
    value: serde_json::Value,
    associative_arrays: bool,
) -> crate::error::Result<Zval> {
    match value {
        serde_json::Value::String(str) => {
            str.into_zval(false).map_err(|err| SqlxError::Conversion {
                message: format!("String conversion: {err:?}"),
            })
        }
        serde_json::Value::Number(number) => if let Some(i) = number.as_i64() {
            i.into_zval(false)
        } else if let Some(f) = number.as_f64() {
            f.into_zval(false)
        } else {
            number.to_string().into_zval(false)
        }
        .map_err(|err| SqlxError::Conversion {
            message: format!("Number conversion: {err:?}"),
        }),
        serde_json::Value::Bool(bool) => {
            bool.into_zval(false).map_err(|err| SqlxError::Conversion {
                message: format!("Bool conversion: {err:?}"),
            })
        }
        serde_json::Value::Null => Ok(Zval::null()),
        serde_json::Value::Array(array) => Ok(array
            .into_iter()
            .map(|x| json_into_zval(x, associative_arrays))
            .collect::<crate::error::Result<Vec<Zval>>>()?
            .into_zval(false)
            .map_err(|err| SqlxError::Conversion {
                message: format!("Array conversion: {err:?}"),
            })?),
        serde_json::Value::Object(object) => {
            if associative_arrays {
                Ok(object
                    .into_iter()
                    .map(|(key, value)| Ok((key, json_into_zval(value, associative_arrays)?)))
                    .collect::<crate::error::Result<HashMap<String, Zval>>>()?
                    .into_zval(false)
                    .map_err(|err| SqlxError::Conversion {
                        message: format!("Object conversion: {err:?}"),
                    })?)
            } else {
                Ok(object
                    .into_iter()
                    .try_fold(
                        zend_object::new_stdclass(),
                        |mut std_object, (key, value)| {
                            std_object
                                .set_property(&key, json_into_zval(value, associative_arrays))
                                .map(|()| std_object)
                                .map_err(|err| SqlxError::Conversion {
                                    message: format!("Object conversion: {err:?}"),
                                })
                        },
                    )?
                    .into_zval(false)
                    .map_err(|err| SqlxError::Conversion {
                        message: format!("Object conversion: {err:?}"),
                    })?)
            }
        }
    }
}
