#[cfg(feature = "lazy-row")]
use crate::LazyRow;
use crate::utils::ZvalNull;
use anyhow::anyhow;
use ext_php_rs::boxed::ZBox;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::ffi::zend_array;
use ext_php_rs::ffi::zend_object;
use ext_php_rs::types::{ArrayKey, Zval};
use sqlx::Column;
use sqlx::Row;
use std::collections::HashMap;

/// Trait to convert a row into a PHP value.
pub trait Conversion: Row {
    /// Convert the row into a PHP associative array.
    fn into_zval(self, associative_arrays: bool) -> anyhow::Result<Zval>
    where
        Self: Sized,
    {
        let columns = self.columns();
        if associative_arrays {
            let mut lazy = false;
            let array = columns.iter().try_fold(
                zend_array::with_capacity(u32::try_from(columns.len())?),
                |mut array, column| -> anyhow::Result<ZBox<zend_array>> {
                    lazy = true;
                    array
                        .insert(
                            column.name(),
                            self.column_value_into_zval(column, associative_arrays)?,
                        )
                        .map_err(|err| anyhow!("{err:?}"))?;
                    Ok(array)
                },
            )?;
            #[cfg(feature = "lazy-row")]
            if lazy {
                return LazyRow::new(array)
                    .into_zval(false)
                    .map_err(|err| anyhow!("{err:?}"));
            }
            Ok(array.into_zval(false).map_err(|err| anyhow!("{err:?}"))?)
        } else {
            Ok(columns
                .iter()
                .try_fold(zend_object::new_stdclass(), |mut object, column| {
                    object
                        .set_property(
                            column.name(),
                            self.column_value_into_zval(column, associative_arrays)?,
                        )
                        .map(|()| object)
                        .map_err(|err| anyhow!("{:?}", err))
                })?
                .into_zval(false)
                .map_err(|err| anyhow!("{:?}", err))?)
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
    ) -> anyhow::Result<Zval>
    where
        C: Column<Database = D>;

    fn column_value_into_array_key<'a, C, D>(
        &self,
        column: &C,
    ) -> anyhow::Result<ArrayKey<'a>>
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
) -> anyhow::Result<Zval> {
    match value {
        serde_json::Value::String(str) => str
            .into_zval(false)
            .map_err(|err| anyhow!("String conversion: {err:?}")),
        serde_json::Value::Number(number) => number
            .to_string()
            .into_zval(false)
            .map_err(|err| anyhow!("Number conversion: {err:?}")),
        serde_json::Value::Bool(bool) => bool
            .into_zval(false)
            .map_err(|err| anyhow!("Bool conversion: {err:?}")),
        serde_json::Value::Null => Ok(Zval::null()),
        serde_json::Value::Array(array) => Ok(array
            .into_iter()
            .map(|x| json_into_zval(x, associative_arrays))
            .collect::<anyhow::Result<Vec<Zval>>>()?
            .into_zval(false)
            .map_err(|err| anyhow!("Array conversion: {err:?}"))?),
        serde_json::Value::Object(object) => {
            if associative_arrays {
                Ok(object
                    .into_iter()
                    .map(|(key, value)| Ok((key, json_into_zval(value, associative_arrays)?)))
                    .collect::<anyhow::Result<HashMap<String, Zval>>>()?
                    .into_zval(false)
                    .map_err(|err| anyhow!("Object conversion: {err:?}"))?)
            } else {
                Ok(object
                    .into_iter()
                    .try_fold(
                        zend_object::new_stdclass(),
                        |mut std_object, (key, value)| {
                            std_object
                                .set_property(&key, json_into_zval(value, associative_arrays))
                                .map(|()| std_object)
                                .map_err(|err| anyhow!("Object conversion: {:?}", err))
                        },
                    )?
                    .into_zval(false)
                    .map_err(|err| anyhow!("Object conversion: {err:?}"))?)
            }
        }
    }
}
