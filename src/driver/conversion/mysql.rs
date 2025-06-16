use crate::driver::conversion::{Conversion, json_into_zval};
use anyhow::{anyhow, bail};
use ext_php_rs::binary::Binary;
use ext_php_rs::boxed::ZBox;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::ffi::{zend_array, zend_object};
use ext_php_rs::types::Zval;
use sqlx::Column;
use sqlx::TypeInfo;
use sqlx::mysql::MySqlRow;
use sqlx::{Decode, Row, Type};

impl Conversion for MySqlRow {
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
    fn column_value_into_zval<MySqlColumn: Column, MySql>(
        &self,
        column: &MySqlColumn,
        associative_arrays: bool,
    ) -> anyhow::Result<Zval> {
        fn try_cast_into_zval<'r, T>(row: &'r MySqlRow, name: &str) -> anyhow::Result<Zval>
        where
            T: Decode<'r, <MySqlRow as Row>::Database> + Type<<MySqlRow as Row>::Database>,
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
            _ => bail!("unsupported type: {column_type}"),
        })
    }
}
