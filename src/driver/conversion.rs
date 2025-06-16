use std::collections::HashMap;
use anyhow::{anyhow, bail};
use ext_php_rs::binary::Binary;
use ext_php_rs::boxed::ZBox;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::ffi::{zend_array, zend_object};
use ext_php_rs::types::Zval;
use sqlx::postgres::{PgColumn, PgRow};
use sqlx::{Decode, Row, Type};
use sqlx::TypeInfo;
use sqlx::Column;


/// Trait to convert a row into a PHP value.
pub trait RowToZval: Row {
    /// Convert the row into a PHP associative array.
    fn into_zval(self, associative_arrays: bool) -> anyhow::Result<Zval>;
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
/// Trait to convert a column value into a PHP value.
pub trait ColumnToZval {
    /// Converts a specific column from a row to a PHP value.
    ///
    /// # Arguments
    /// - `column`: Reference to the column in the row.
    /// - `associative_arrays`: Whether to render complex types as associative arrays
    ///
    /// # Returns
    /// A PHP-compatible `Zval` value
    fn column_value_into_zval(
        &self,
        column: &PgColumn,
        associative_arrays: bool,
    ) -> anyhow::Result<Zval>;
}
impl ColumnToZval for &PgRow {
    fn column_value_into_zval(
        &self,
        column: &PgColumn,
        associative_arrays: bool,
    ) -> anyhow::Result<Zval> {
        fn try_cast_into_zval<'r, T>(row: &'r PgRow, name: &str) -> anyhow::Result<Zval>
        where
            T: Decode<'r, <PgRow as Row>::Database> + Type<<PgRow as Row>::Database>,
            T: IntoZval,
        {
            row.try_get::<'r, T, _>(name)
                .map_err(|err| anyhow!("{err:?}"))?
                .into_zval(false)
                .map_err(|err| anyhow!("{err:?}"))
        }

        let column_name = column.name();
        let column_type = column.type_info().name();
        Ok(match column_type {
            "BOOL" => try_cast_into_zval::<bool>(self, column_name)?,
            "BYTEA" | "BINARY" => (self)
                .try_get::<&[u8], _>(column_name)
                .map_err(|err| anyhow!("{err:?}"))
                .map(|x| x.iter().copied().collect::<Binary<_>>())?
                .into_zval(false)
                .map_err(|err| anyhow!("{err:?}"))?,
            "CHAR" | "NAME" | "TEXT" | "BPCHAR" | "VARCHAR" => {
                try_cast_into_zval::<String>(self, column_name)?
            }
            "INT2" => try_cast_into_zval::<i16>(self, column_name)?,
            "INT4" | "INT" => try_cast_into_zval::<i32>(self, column_name)?,
            "INT8" => try_cast_into_zval::<i64>(self, column_name)?,
            "OID" => try_cast_into_zval::<i32>(self, column_name)?,
            "FLOAT4" => try_cast_into_zval::<f32>(self, column_name)?,
            "FLOAT8" | "F64" => try_cast_into_zval::<f64>(self, column_name)?,
            "NUMERIC" | "MONEY" => try_cast_into_zval::<String>(self, column_name)?,
            "UUID" => try_cast_into_zval::<String>(self, column_name)?,
            "JSON" | "JSONB" => self
                .try_get::<serde_json::Value, _>(column_name)
                .map_err(|err| anyhow!("{err:?}"))
                .map(|x| json_into_zval(x, associative_arrays))?
                .into_zval(false)
                .map_err(|err| anyhow!("{err:?}"))?,
            "_JSON" | "_JSONB" => self
                .try_get::<Vec<serde_json::Value>, _>(column_name)
                .map_err(|err| anyhow!("{err:?}"))
                .map(|x| {
                    x.into_iter()
                        .map(|x| json_into_zval(x, associative_arrays))
                        .collect::<Vec<_>>()
                })?
                .into_zval(false)
                .map_err(|err| anyhow!("{err:?}"))?,
            "DATE" | "TIME" | "TIMESTAMP" | "TIMESTAMPTZ" | "INTERVAL" | "TIMETZ" => {
                try_cast_into_zval::<String>(self, column_name)?
            }
            "INET" | "CIDR" | "MACADDR" | "MACADDR8" => {
                try_cast_into_zval::<String>(self, column_name)?
            }
            "BIT" | "VARBIT" => try_cast_into_zval::<String>(self, column_name)?,
            "POINT" | "LSEG" | "PATH" | "BOX" | "POLYGON" | "LINE" | "CIRCLE" => {
                try_cast_into_zval::<String>(self, column_name)?
            }
            "INT4RANGE" | "NUMRANGE" | "TSRANGE" | "TSTZRANGE" | "DATERANGE" | "INT8RANGE" => {
                try_cast_into_zval::<String>(self, column_name)?
            }
            "RECORD" => try_cast_into_zval::<String>(self, column_name)?,
            "JSONPATH" => try_cast_into_zval::<String>(self, column_name)?,

            // массивы
            "_BOOL" => try_cast_into_zval::<Vec<bool>>(self, column_name)?,
            "_BYTEA" => try_cast_into_zval::<Vec<Vec<u8>>>(self, column_name)?,
            "_CHAR" | "_NAME" | "_TEXT" | "_BPCHAR" | "_VARCHAR" => {
                try_cast_into_zval::<Vec<String>>(self, column_name)?
            }
            "_INT2" => try_cast_into_zval::<Vec<i16>>(self, column_name)?,
            "_INT4" => try_cast_into_zval::<Vec<i32>>(self, column_name)?,
            "_INT8" => try_cast_into_zval::<Vec<i64>>(self, column_name)?,
            "_OID" => try_cast_into_zval::<Vec<i32>>(self, column_name)?,
            "_FLOAT4" => try_cast_into_zval::<Vec<f32>>(self, column_name)?,
            "_FLOAT8" => try_cast_into_zval::<Vec<f64>>(self, column_name)?,
            "_NUMERIC" | "_MONEY" => try_cast_into_zval::<Vec<String>>(self, column_name)?,
            "_UUID" => try_cast_into_zval::<Vec<String>>(self, column_name)?,
            "_DATE" | "_TIME" | "_TIMESTAMP" | "_TIMESTAMPTZ" | "_INTERVAL" | "_TIMETZ" => {
                try_cast_into_zval::<Vec<String>>(self, column_name)?
            }
            "_INET" | "_CIDR" | "_MACADDR" | "_MACADDR8" => {
                try_cast_into_zval::<Vec<String>>(self, column_name)?
            }
            "_BIT" | "_VARBIT" => try_cast_into_zval::<Vec<String>>(self, column_name)?,
            "_POINT" | "_LSEG" | "_PATH" | "_BOX" | "_POLYGON" | "_LINE" | "_CIRCLE" => {
                try_cast_into_zval::<Vec<String>>(self, column_name)?
            }
            "_INT4RANGE" | "_NUMRANGE" | "_TSRANGE" | "_TSTZRANGE" | "_DATERANGE"
            | "_INT8RANGE" => try_cast_into_zval::<Vec<String>>(self, column_name)?,
            "_RECORD" => try_cast_into_zval::<Vec<String>>(self, column_name)?,
            "_JSONPATH" => try_cast_into_zval::<Vec<String>>(self, column_name)?,

            _ => bail!("unsupported type: {column_type}"),
        })
    }
}
impl RowToZval for PgRow {
    fn into_zval(self, associative_arrays: bool) -> anyhow::Result<Zval> {
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
                                (&self).column_value_into_zval(column, associative_arrays)?,
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
                            (&self).column_value_into_zval(column, associative_arrays)?,
                        )
                        .map(|()| object)
                        .map_err(|err| anyhow!("{:?}", err))
                })?
                .into_zval(false)
                .map_err(|err| anyhow!("{:?}", err))?)
        }
    }
}
