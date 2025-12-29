// Allow separate match arms for different SQL types that map to the same Rust type
// This improves readability by showing all supported SQL types explicitly
#![allow(clippy::match_same_arms)]

use crate::conversion::{Conversion, json_into_zval};
use crate::error::Error as SqlxError;
#[cfg(feature = "lazy-row")]
use crate::lazy_row::LazyRowJson;
use ext_php_rs::binary::Binary;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::types::{ArrayKey, Zval};
use sqlx_oldapi::Column;
use sqlx_oldapi::Error::ColumnDecode;
use sqlx_oldapi::TypeInfo;
use sqlx_oldapi::error::UnexpectedNullError;
use sqlx_oldapi::postgres::{PgRow, PgValueRef};
use sqlx_oldapi::{Decode, Row, Type};

impl Conversion for PgRow {
    fn column_value_into_zval<PgColumn: Column, Postgres>(
        &self,
        column: &PgColumn,
        associative_arrays: bool,
    ) -> crate::error::Result<Zval> {
        fn try_cast_into_zval<'r, T>(row: &'r PgRow, column_ordinal: usize) -> crate::error::Result<Zval>
        where
            T: Decode<'r, <PgRow as Row>::Database> + Type<<PgRow as Row>::Database>,
            T: IntoZval,
        {
            match row.try_get::<'r, T, _>(column_ordinal) {
                Ok(value) => Ok(value.into_zval(false).map_err(|err| SqlxError::Conversion { message: format!("{err:?}") })?),
                Err(ColumnDecode { source, .. }) if source.is::<UnexpectedNullError>() => {
                    Ok(Zval::null())
                }
                Err(err) => Err(SqlxError::Conversion { message: format!("{err:?}") }),
            }
        }
        let column_ordinal = column.ordinal();
        Ok(match column.type_info().name() {
            "BOOL" => try_cast_into_zval::<bool>(self, column_ordinal)?,
            "BYTEA" | "BINARY" => self
                .try_get::<&[u8], _>(column_ordinal)
                .map_err(|err| SqlxError::Conversion { message: format!("{err:?}") })
                .map(|x| x.iter().copied().collect::<Binary<_>>())?
                .into_zval(false)
                .map_err(|err| SqlxError::Conversion { message: format!("{err:?}") })?,
            "CHAR" | "NAME" | "TEXT" | "BPCHAR" | "VARCHAR" => {
                try_cast_into_zval::<String>(self, column_ordinal)?
            }
            "INT2" => try_cast_into_zval::<i16>(self, column_ordinal)?,
            "INT4" | "INT" => try_cast_into_zval::<i32>(self, column_ordinal)?,
            "INT8" => try_cast_into_zval::<i64>(self, column_ordinal)?,
            "OID" => try_cast_into_zval::<i32>(self, column_ordinal)?,
            "FLOAT4" => try_cast_into_zval::<f32>(self, column_ordinal)?,
            "FLOAT8" | "F64" => try_cast_into_zval::<f64>(self, column_ordinal)?,
            "NUMERIC" | "MONEY" => try_cast_into_zval::<String>(self, column_ordinal)?,
            "UUID" => try_cast_into_zval::<String>(self, column_ordinal)?,
            "JSON" => self
                .try_get_raw::<_>(column_ordinal)
                .map(|val_ref: PgValueRef| {
                    let buf = val_ref.as_bytes().map_err(|err| SqlxError::Conversion { message: format!("{err:?}") })?;
                    if buf.is_empty() {
                        return Err(SqlxError::Conversion { message: format!("empty JSON raw value in {:?}", column.name()) });
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
                })??,
            "JSONB" => self
                .try_get_raw::<_>(column_ordinal)
                .map(|val_ref: PgValueRef| {
                    let buf = val_ref.as_bytes().map_err(|err| SqlxError::Conversion { message: format!("{err:?}") })?;
                    if buf.is_empty() {
                        return Err(SqlxError::Conversion { message: format!("empty JSONB raw value in {:?}", column.name()) });
                    }
                    if buf[0] != 1 {
                        return Err(SqlxError::Conversion {
                            message: format!("unsupported JSONB format version {}; please open an issue", buf[0]),
                        });
                    }
                    #[cfg(not(feature = "lazy-row"))]
                    {
                        #[cfg(feature = "simd-json")]
                        return json_into_zval(
                            simd_json::from_slice::<serde_json::Value>(&mut buf[1..].to_vec())?,
                            associative_arrays,
                        );
                        #[cfg(not(feature = "simd-json"))]
                        return serde_json::from_slice(&mut buf[1..].to_vec())?
                            .into_zval(associative_arrays)
                            .map_err(|err| SqlxError::Conversion { message: format!("{err:?}") });
                    }
                    #[cfg(feature = "lazy-row")]
                    LazyRowJson::new(&buf[1..], associative_arrays)
                        .into_zval(associative_arrays)
                        .map_err(|err| SqlxError::Conversion { message: format!("{err:?}") })
                })??,
            "_JSON" | "_JSONB" => self
                .try_get::<Vec<serde_json::Value>, _>(column_ordinal)
                .map_err(|err| SqlxError::Conversion { message: format!("{err:?}") })
                .map(|x| -> Vec<_> {
                    x.into_iter()
                        .map(|x| json_into_zval(x, associative_arrays))
                        .collect()
                })?
                .into_zval(false)
                .map_err(|err| SqlxError::Conversion { message: format!("{err:?}") })?,
            "DATE" | "TIME" | "TIMESTAMP" | "TIMESTAMPTZ" | "INTERVAL" | "TIMETZ" => {
                try_cast_into_zval::<String>(self, column_ordinal)?
            }
            "INET" | "CIDR" | "MACADDR" | "MACADDR8" => {
                try_cast_into_zval::<String>(self, column_ordinal)?
            }
            "BIT" | "VARBIT" => try_cast_into_zval::<String>(self, column_ordinal)?,
            "POINT" | "LSEG" | "PATH" | "BOX" | "POLYGON" | "LINE" | "CIRCLE" => {
                try_cast_into_zval::<String>(self, column_ordinal)?
            }
            "INT4RANGE" | "NUMRANGE" | "TSRANGE" | "TSTZRANGE" | "DATERANGE" | "INT8RANGE" => {
                try_cast_into_zval::<String>(self, column_ordinal)?
            }
            "RECORD" => try_cast_into_zval::<String>(self, column_ordinal)?,
            "JSONPATH" => try_cast_into_zval::<String>(self, column_ordinal)?,

            // массивы
            "_BOOL" => try_cast_into_zval::<Vec<bool>>(self, column_ordinal)?,
            "_BYTEA" => try_cast_into_zval::<Vec<Vec<u8>>>(self, column_ordinal)?,
            "_CHAR" | "_NAME" | "_TEXT" | "_BPCHAR" | "_VARCHAR" => {
                try_cast_into_zval::<Vec<String>>(self, column_ordinal)?
            }
            "_INT2" => try_cast_into_zval::<Vec<i16>>(self, column_ordinal)?,
            "_INT4" => try_cast_into_zval::<Vec<i32>>(self, column_ordinal)?,
            "_INT8" => try_cast_into_zval::<Vec<i64>>(self, column_ordinal)?,
            "_OID" => try_cast_into_zval::<Vec<i32>>(self, column_ordinal)?,
            "_FLOAT4" => try_cast_into_zval::<Vec<f32>>(self, column_ordinal)?,
            "_FLOAT8" => try_cast_into_zval::<Vec<f64>>(self, column_ordinal)?,
            "_NUMERIC" | "_MONEY" => try_cast_into_zval::<Vec<String>>(self, column_ordinal)?,
            "_UUID" => try_cast_into_zval::<Vec<String>>(self, column_ordinal)?,
            "_DATE" | "_TIME" | "_TIMESTAMP" | "_TIMESTAMPTZ" | "_INTERVAL" | "_TIMETZ" => {
                try_cast_into_zval::<Vec<String>>(self, column_ordinal)?
            }
            "_INET" | "_CIDR" | "_MACADDR" | "_MACADDR8" => {
                try_cast_into_zval::<Vec<String>>(self, column_ordinal)?
            }
            "_BIT" | "_VARBIT" => try_cast_into_zval::<Vec<String>>(self, column_ordinal)?,
            "_POINT" | "_LSEG" | "_PATH" | "_BOX" | "_POLYGON" | "_LINE" | "_CIRCLE" => {
                try_cast_into_zval::<Vec<String>>(self, column_ordinal)?
            }
            "_INT4RANGE" | "_NUMRANGE" | "_TSRANGE" | "_TSTZRANGE" | "_DATERANGE"
            | "_INT8RANGE" => try_cast_into_zval::<Vec<String>>(self, column_ordinal)?,
            "_RECORD" => try_cast_into_zval::<Vec<String>>(self, column_ordinal)?,
            "_JSONPATH" => try_cast_into_zval::<Vec<String>>(self, column_ordinal)?,

            other => return Err(SqlxError::Conversion { message: format!("unsupported type: {other}") }),
        })
    }

    fn column_value_into_array_key<'a, PgColumn: Column, Postgres>(
        &self,
        column: &PgColumn,
    ) -> crate::error::Result<ArrayKey<'a>> {
        let column_ordinal = column.ordinal();
        Ok(match column.type_info().name() {
            "BOOLEAN" => ArrayKey::Long(i64::from(self.try_get::<bool, _>(column_ordinal)?)),
            "BIT" => {
                let v: Vec<u8> = self.try_get(column_ordinal)?;
                if v.len() == 1 {
                    ArrayKey::Long(i64::from(v[0] != 0))
                } else {
                    ArrayKey::Long(0)
                }
            }
            "INT2" | "INT4" | "INT" => {
                ArrayKey::Long(self.try_get::<i32, _>(column_ordinal)?.into())
            }
            "INT8" | "OID" => ArrayKey::Long(self.try_get::<i64, _>(column_ordinal)?),
            "CHAR" | "NAME" | "TEXT" | "BPCHAR" | "VARCHAR" | "NUMERIC" | "MONEY" => {
                ArrayKey::String(self.try_get::<String, _>(column_ordinal)?)
            }
            "ENUM" | "SET" => ArrayKey::String(self.try_get::<String, _>(column_ordinal)?),
            "NULL" => ArrayKey::Str(""),
            other => return Err(SqlxError::Conversion { message: format!("unsupported type for array key: {other}") }),
        })
    }
}
