// Allow separate match arms for different SQL types that map to the same Rust type
// This improves readability by showing all supported SQL types explicitly
#![allow(clippy::match_same_arms)]

use crate::LazyRowJson;
use crate::conversion::{Conversion, json_into_zval};
use crate::error::Error as SqlxError;
use ext_php_rs::binary::Binary;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::types::{ArrayKey, Zval};
use sqlx_oldapi::Column;
use sqlx_oldapi::TypeInfo;
use sqlx_oldapi::mysql::{MySqlRow, MySqlValueRef};
use sqlx_oldapi::{Decode, Row, Type};

impl Conversion for MySqlRow {
    fn column_value_into_zval<MySqlColumn: Column, MySql>(
        &self,
        column: &MySqlColumn,
        associative_arrays: bool,
    ) -> crate::error::Result<Zval> {
        fn try_cast_into_zval<'r, T>(row: &'r MySqlRow, column_ordinal: usize) -> crate::error::Result<Zval>
        where
            T: Decode<'r, <MySqlRow as Row>::Database> + Type<<MySqlRow as Row>::Database>,
            T: IntoZval,
        {
            row.try_get::<'r, T, _>(column_ordinal)
                .map_err(|err| SqlxError::Conversion { message: format!("{err:?}") })?
                .into_zval(false)
                .map_err(|err| SqlxError::Conversion { message: format!("{err:?}") })
        }

        let column_ordinal = column.ordinal();
        Ok(match column.type_info().name() {
            "BOOLEAN" => try_cast_into_zval::<bool>(self, column_ordinal)?,
            "TINYINT" => try_cast_into_zval::<i8>(self, column_ordinal)?,
            "TINYINT UNSIGNED" => try_cast_into_zval::<u8>(self, column_ordinal)?,
            "SMALLINT" => try_cast_into_zval::<i16>(self, column_ordinal)?,
            "SMALLINT UNSIGNED" => try_cast_into_zval::<u16>(self, column_ordinal)?,
            "MEDIUMINT" => try_cast_into_zval::<i32>(self, column_ordinal)?,
            "INTEGER" | "INT" => try_cast_into_zval::<i32>(self, column_ordinal)?,
            "INT UNSIGNED" => try_cast_into_zval::<u32>(self, column_ordinal)?,
            "BIGINT" => try_cast_into_zval::<i64>(self, column_ordinal)?,
            "BIGINT UNSIGNED" => try_cast_into_zval::<u64>(self, column_ordinal)?,
            "FLOAT" => try_cast_into_zval::<f32>(self, column_ordinal)?,
            "DOUBLE" | "REAL" => try_cast_into_zval::<f64>(self, column_ordinal)?,
            "DECIMAL" | "NUMERIC" => try_cast_into_zval::<String>(self, column_ordinal)?,
            "DATE" => try_cast_into_zval::<String>(self, column_ordinal)?,
            "TIME" => try_cast_into_zval::<String>(self, column_ordinal)?,
            "DATETIME" | "TIMESTAMP" => try_cast_into_zval::<String>(self, column_ordinal)?,
            "YEAR" => try_cast_into_zval::<i32>(self, column_ordinal)?,
            "BLOB" | "TINYBLOB" | "MEDIUMBLOB" | "LONGBLOB" | "BINARY" | "VARBINARY" => self
                .try_get::<&[u8], _>(column_ordinal)?
                .iter()
                .copied()
                .collect::<Binary<_>>()
                .into_zval(false)
                .map_err(|err| SqlxError::Conversion { message: format!("{err:?}") })?,
            "TEXT" | "TINYTEXT" | "MEDIUMTEXT" | "LONGTEXT" | "CHAR" | "VARCHAR" => {
                try_cast_into_zval::<String>(self, column_ordinal)?
            }
            "ENUM" | "SET" => try_cast_into_zval::<String>(self, column_ordinal)?,
            "BIT" => {
                let v: Vec<u8> = self.try_get(column_ordinal)?;
                if v.len() == 1 {
                    (v[0] != 0)
                        .into_zval(false)
                        .map_err(|err| SqlxError::Conversion { message: format!("{err:?}") })?
                } else {
                    Binary::from(v)
                        .into_zval(false)
                        .map_err(|err| SqlxError::Conversion { message: format!("{err:?}") })?
                }
            }
            "NULL" => Zval::null(),
            "JSON" => {
                self.try_get_raw::<_>(column_ordinal)
                    .map(|value_ref: MySqlValueRef<'_>| {
                        let buf = value_ref.as_bytes().map_err(|err| SqlxError::Conversion { message: format!("{err:?}") })?;
                        if buf.is_empty() {
                            return Err(SqlxError::Conversion { message: format!("empty JSON raw value in {column_ordinal}") });
                        }
                        #[cfg(feature = "lazy-row")]
                        if buf.len() > crate::lazy_row::LAZY_ROW_JSON_SIZE_THRESHOLD {
                            return LazyRowJson::new(buf, associative_arrays)
                                .into_zval(associative_arrays)
                                .map_err(|err| SqlxError::Conversion { message: format!("{err:?}") });
                        }

                        #[cfg(feature = "simd-json")]
                        return json_into_zval(
                            simd_json::from_slice::<serde_json::Value>(&mut buf.to_vec())?,
                            associative_arrays,
                        );
                        #[cfg(not(feature = "simd-json"))]
                        return json_into_zval(
                            serde_json::from_slice::<serde_json::Value>(&mut buf.to_vec())?,
                            associative_arrays,
                        );
                    })??
            }
            other => return Err(SqlxError::Conversion { message: format!("unsupported type: {other}") }),
        })
    }

    fn column_value_into_array_key<'a, MySqlColumn: Column, MySql>(
        &self,
        column: &MySqlColumn,
    ) -> crate::error::Result<ArrayKey<'a>> {
        let column_name = column.name();
        Ok(match column.type_info().name() {
            "BOOLEAN" => ArrayKey::Long(i64::from(self.try_get::<bool, _>(column_name)?)),
            "BIT" => {
                let v: Vec<u8> = self.try_get(column_name)?;
                if v.len() == 1 {
                    ArrayKey::Long(i64::from(v[0] != 0))
                } else {
                    ArrayKey::Long(0)
                }
            }
            "TINYINT UNSIGNED" | "TINYINT" | "SMALLINT" | "SMALLINT UNSIGNED" | "MEDIUMINT"
            | "INTEGER" | "INT" | "BIGINT" | "BIGINT UNSIGNED" | "YEAR" => {
                ArrayKey::Long(self.try_get::<i64, _>(column_name)?)
            }
            "TEXT" | "TINYTEXT" | "MEDIUMTEXT" | "LONGTEXT" | "CHAR" | "VARCHAR" => {
                ArrayKey::String(self.try_get::<String, _>(column_name)?)
            }
            "ENUM" | "SET" => ArrayKey::String(self.try_get::<String, _>(column_name)?),
            "NULL" => ArrayKey::Str(""),
            other => return Err(SqlxError::Conversion { message: format!("unsupported type for array key: {other}") }),
        })
    }
}
