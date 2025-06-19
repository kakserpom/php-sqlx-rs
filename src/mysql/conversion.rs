use crate::utils::ZvalNull;
use crate::conversion::{Conversion, json_into_zval};
use anyhow::{anyhow, bail};
use ext_php_rs::binary::Binary;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::types::Zval;
use sqlx::Column;
use sqlx::TypeInfo;
use sqlx::mysql::MySqlRow;
use sqlx::{Decode, Row, Type};

impl Conversion for MySqlRow {
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
        Ok(match column.type_info().name() {
            "BOOLEAN" => try_cast_into_zval::<bool>(self, column_name)?,
            "TINYINT" => try_cast_into_zval::<i8>(self, column_name)?,
            "TINYINT UNSIGNED" => try_cast_into_zval::<u8>(self, column_name)?,
            "SMALLINT" => try_cast_into_zval::<i16>(self, column_name)?,
            "SMALLINT UNSIGNED" => try_cast_into_zval::<u16>(self, column_name)?,
            "MEDIUMINT" => try_cast_into_zval::<i32>(self, column_name)?,
            "INTEGER" | "INT" => try_cast_into_zval::<i32>(self, column_name)?,
            "INT UNSIGNED" => try_cast_into_zval::<u32>(self, column_name)?,
            "BIGINT" => try_cast_into_zval::<i64>(self, column_name)?,
            "BIGINT UNSIGNED" => try_cast_into_zval::<u64>(self, column_name)?,
            "FLOAT" => try_cast_into_zval::<f32>(self, column_name)?,
            "DOUBLE" | "REAL" => try_cast_into_zval::<f64>(self, column_name)?,
            "DECIMAL" | "NUMERIC" => try_cast_into_zval::<String>(self, column_name)?,
            "DATE" => try_cast_into_zval::<String>(self, column_name)?,
            "TIME" => try_cast_into_zval::<String>(self, column_name)?,
            "DATETIME" | "TIMESTAMP" => try_cast_into_zval::<String>(self, column_name)?,
            "YEAR" => try_cast_into_zval::<i32>(self, column_name)?,
            "BLOB" | "TINYBLOB" | "MEDIUMBLOB" | "LONGBLOB" | "BINARY" | "VARBINARY" => self
                .try_get::<&[u8], _>(column_name)?
                .iter()
                .copied()
                .collect::<Binary<_>>()
                .into_zval(false)
                .map_err(|err| anyhow!("{err:?}"))?,
            "TEXT" | "TINYTEXT" | "MEDIUMTEXT" | "LONGTEXT" | "CHAR" | "VARCHAR" => {
                try_cast_into_zval::<String>(self, column_name)?
            }
            "ENUM" | "SET" => try_cast_into_zval::<String>(self, column_name)?,
            "BIT" => {
                let v: Vec<u8> = self.try_get(column_name)?;
                if v.len() == 1 {
                    (v[0] != 0)
                        .into_zval(false)
                        .map_err(|err| anyhow!("{err:?}"))?
                } else {
                    Binary::from(v)
                        .into_zval(false)
                        .map_err(|err| anyhow!("{err:?}"))?
                }
            }
            "NULL" => Zval::null(),
            "JSON" => self
                .try_get::<serde_json::Value, _>(column_name)
                .map_err(|err| anyhow!("{err:?}"))
                .map(|x| json_into_zval(x, associative_arrays))?
                .into_zval(false)
                .map_err(|err| anyhow!("{err:?}"))?,
            other => bail!("unsupported type: {other}"),
        })
    }
}
