use crate::error::Error as SqlxError;
#[cfg(feature = "lazy-row")]
use crate::lazy_row::{LazyRow, LazyRowJson};
use ext_php_rs::boxed::ZBox;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::ffi::zend_array;
use ext_php_rs::ffi::zend_object;
#[cfg(feature = "lazy-row")]
use ext_php_rs::types::ZendClassObject;
use ext_php_rs::types::{ArrayKey, Zval};
use sqlx_oldapi::Column;
use sqlx_oldapi::Row;
use std::collections::HashMap;

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
