use crate::utils::ZvalNull;
use crate::conversion::{Conversion, json_into_zval};
use anyhow::{anyhow, bail};
use ext_php_rs::binary::Binary;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::types::Zval;
use sqlx::Column;
use sqlx::Error::ColumnDecode;
use sqlx::TypeInfo;
use sqlx::error::UnexpectedNullError;
use sqlx::postgres::PgRow;
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

            other => bail!("unsupported type: {other}"),
        })
    }
}
