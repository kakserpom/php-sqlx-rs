use crate::conversion::{Conversion, json_into_zval};
#[cfg(feature = "lazy-row")]
use crate::lazy_row::LazyRowJson;
use crate::utils::ZvalNull;
use anyhow::{anyhow, bail};
use ext_php_rs::binary::Binary;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::types::{ArrayKey, Zval};
use sqlx::Column;
use sqlx::Error::ColumnDecode;
use sqlx::TypeInfo;
use sqlx::error::UnexpectedNullError;
use sqlx::postgres::{PgRow, PgValueRef};
use sqlx::{Decode, Row, Type};

impl Conversion for PgRow {
    fn column_value_into_zval<PgColumn: Column, Postgres>(
        &self,
        column: &PgColumn,
        associative_arrays: bool,
    ) -> anyhow::Result<Zval> {
        fn try_cast_into_zval<'r, T>(row: &'r PgRow, name: &str) -> anyhow::Result<Zval>
        where
            T: Decode<'r, <PgRow as Row>::Database> + Type<<PgRow as Row>::Database>,
            T: IntoZval,
        {
            match row.try_get::<'r, T, _>(name) {
                Ok(value) => Ok(value.into_zval(false).map_err(|err| anyhow!("{err:?}"))?),
                Err(ColumnDecode { source, .. }) if source.is::<UnexpectedNullError>() => {
                    Ok(Zval::null())
                }
                Err(err) => Err(anyhow!("{err:?}")),
            }
        }

        let column_name = column.name();
        Ok(match column.type_info().name() {
            "BOOL" => try_cast_into_zval::<bool>(self, column_name)?,
            "BYTEA" | "BINARY" => self
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
            "JSON" => self
                .try_get_raw::<_>(column_name)
                .map(|val_ref: PgValueRef| {
                    let buf = val_ref.as_bytes().map_err(|err| anyhow!("{err:?}"))?;
                    if buf.is_empty() {
                        bail!("empty JSON raw value in {column_name}");
                    }
                    #[cfg(feature = "lazy-row")]
                    if buf.len() > 4096 {
                        return LazyRowJson::new(&buf, associative_arrays)
                            .into_zval(associative_arrays)
                            .map_err(|err| anyhow!("{err:?}"));
                    }

                    #[cfg(feature = "simd-json")]
                    return json_into_zval(
                        simd_json::from_slice::<serde_json::Value>(&mut buf.to_vec())?,
                        associative_arrays,
                    );
                    #[cfg(not(feature = "simd-json"))]
                    return crate::conversion::json_into_zval(
                        serde_json::from_slice(&mut buf.to_vec())?
                            .into_zval(associative_arrays)
                            .map_err(|err| anyhow!("{err:?}")),
                    );
                })??,
            "JSONB" => self
                .try_get_raw::<_>(column_name)
                .map(|val_ref: PgValueRef| {
                    let buf = val_ref.as_bytes().map_err(|err| anyhow!("{err:?}"))?;
                    if buf.is_empty() {
                        bail!("empty JSONB raw value in {column_name}");
                    }
                    if buf[0] != 1 {
                        bail!(
                            "unsupported JSONB format version {}; please open an issue",
                            buf[0]
                        );
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
                            .map_err(|err| anyhow!("{err:?}"));
                    }
                    #[cfg(feature = "lazy-row")]
                    LazyRowJson::new(&buf[1..], associative_arrays)
                        .into_zval(associative_arrays)
                        .map_err(|err| anyhow!("{err:?}"))
                })??,
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

            other => bail!("unsupported type: {other}"),
        })
    }

    fn column_value_into_array_key<'a, 'b, PgColumn: Column, Postgres>(
        &'a self,
        column: &PgColumn,
    ) -> anyhow::Result<ArrayKey<'b>> {
        let column_name = column.name();
        Ok(match column.type_info().name() {
            "BOOLEAN" => ArrayKey::Long(if self.try_get::<bool, _>(column_name)? {
                1
            } else {
                0
            }),
            "BIT" => {
                let v: Vec<u8> = self.try_get(column_name)?;
                if v.len() == 1 {
                    ArrayKey::Long(if v[0] != 0 { 1 } else { 0 })
                } else {
                    ArrayKey::Long(0)
                }
            }
            "INT2" | "INT4" | "INT" | "INT8" | "OID" => {
                ArrayKey::Long(self.try_get::<i64, _>(column_name)?)
            }
            "CHAR" | "NAME" | "TEXT" | "BPCHAR" | "VARCHAR" | "NUMERIC" | "MONEY" => {
                ArrayKey::String(self.try_get::<String, _>(column_name)?)
            }
            "ENUM" | "SET" => ArrayKey::String(self.try_get::<String, _>(column_name)?),
            "NULL" => ArrayKey::Str(""),
            other => bail!("unsupported type for array key: {other}"),
        })
    }
}
