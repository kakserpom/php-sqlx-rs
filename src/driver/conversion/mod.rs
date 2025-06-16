use anyhow::anyhow;
use ext_php_rs::boxed::ZBox;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::ffi::zend_array;
use ext_php_rs::ffi::zend_object;
use ext_php_rs::types::Zval;
use sqlx::Column;
use sqlx::Row;
use std::collections::HashMap;

#[feature(mysql)]
mod mysql;
#[feature(postgres)]
mod postgres;

/// Trait to convert a row into a PHP value.
pub trait Conversion: Row {
    /// Convert the row into a PHP associative array.
    fn into_zval(self, associative_arrays: bool) -> anyhow::Result<Zval>
    where
        Self: Sized,
    {
        if associative_arrays {
            Ok(self
                .columns()
                .iter()
                .try_fold(
                    zend_array::new(),
                    |mut array, column| -> anyhow::Result<ZBox<zend_array>> {
                        array
                            .insert(
                                column.name(),
                                self.column_value_into_zval(column, associative_arrays)?,
                            )
                            .map_err(|err| anyhow!("{err:?}"))?;
                        Ok(array)
                    },
                )?
                .into_zval(false)
                .map_err(|err| anyhow!("{err:?}"))?)
        } else {
            Ok(self
                .columns()
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
}

/// Converts a JSON value into a PHP value, respecting associative array settings.
///
/// # Arguments
/// - `value`: A `serde_json::Value` to convert
/// - `associative_arrays`: Whether to convert objects into PHP associative arrays or `stdClass`
///
/// # Returns
/// Converted `Zval` or an error if conversion fails
fn json_into_zval(value: serde_json::Value, associative_arrays: bool) -> anyhow::Result<Zval> {
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
        serde_json::Value::Null => {
            let mut null = Zval::new();
            null.set_null();
            Ok(null)
        }
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
