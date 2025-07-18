use crate::conversion::Conversion;
use anyhow::{anyhow, bail};
use ext_php_rs::binary::Binary;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::types::{ArrayKey, Zval};
use sqlx_oldapi::Column;
use sqlx_oldapi::Error::ColumnDecode;
use sqlx_oldapi::TypeInfo;
use sqlx_oldapi::error::UnexpectedNullError;
use sqlx_oldapi::mssql::MssqlRow;
use sqlx_oldapi::{Decode, Row, Type};

impl Conversion for MssqlRow {
    fn column_value_into_zval<MssqlColumn: Column, Mssql>(
        &self,
        column: &MssqlColumn,
        _associative_arrays: bool,
    ) -> anyhow::Result<Zval> {
        fn try_cast_into_zval<'r, T>(
            row: &'r MssqlRow,
            column_ordinal: usize,
        ) -> anyhow::Result<Zval>
        where
            T: Decode<'r, <MssqlRow as Row>::Database> + Type<<MssqlRow as Row>::Database>,
            T: IntoZval,
        {
            match row.try_get::<'r, T, _>(column_ordinal) {
                Ok(value) => Ok(value.into_zval(false).map_err(|err| anyhow!("{err:?}"))?),
                Err(ColumnDecode { source, .. }) if source.is::<UnexpectedNullError>() => {
                    Ok(Zval::null())
                }
                Err(err) => Err(anyhow!("{err:?}")),
            }
        }

        let column_ordinal = column.ordinal();
        Ok(match column.type_info().name() {
            "BIT" => try_cast_into_zval::<bool>(self, column_ordinal)?,

            "TINYINT" => try_cast_into_zval::<u8>(self, column_ordinal)?,
            "SMALLINT" => try_cast_into_zval::<i16>(self, column_ordinal)?,
            "INT" | "INTEGER" => try_cast_into_zval::<i32>(self, column_ordinal)?,
            "BIGINT" => try_cast_into_zval::<i64>(self, column_ordinal)?,

            "REAL" => try_cast_into_zval::<f32>(self, column_ordinal)?,
            "FLOAT" => try_cast_into_zval::<f64>(self, column_ordinal)?,

            "DECIMAL" | "NUMERIC" | "MONEY" | "SMALLMONEY" => {
                try_cast_into_zval::<String>(self, column_ordinal)?
            }

            "CHAR" | "VARCHAR" | "TEXT" | "NCHAR" | "NVARCHAR" | "NTEXT" | "XML"
            | "UNIQUEIDENTIFIER" => try_cast_into_zval::<String>(self, column_ordinal)?,

            "DATE" | "TIME" | "DATETIME" | "DATETIME2" | "SMALLDATETIME" | "DATETIMEOFFSET" => {
                try_cast_into_zval::<String>(self, column_ordinal)?
            }

            "BINARY" | "VARBINARY" | "IMAGE" | "TIMESTAMP" | "ROWVERSION" => self
                .try_get::<&[u8], _>(column_ordinal)?
                .iter()
                .copied()
                .collect::<Binary<_>>()
                .into_zval(false)
                .map_err(|err| anyhow!("{err:?}"))?,

            other => bail!("unsupported type: {}", other),
        })
    }

    fn column_value_into_array_key<'a, MssqlColumn: Column, Mssql>(
        &self,
        column: &MssqlColumn,
    ) -> anyhow::Result<ArrayKey<'a>> {
        let column_ordinal = column.name();
        Ok(match column.type_info().name() {
            "BIT" => {
                let v: bool = self.try_get(column_ordinal)?;
                ArrayKey::Long(i64::from(v))
            }

            "TINYINT" => ArrayKey::Long(i64::from(self.try_get::<u8, _>(column_ordinal)?)),
            "SMALLINT" => ArrayKey::Long(i64::from(self.try_get::<i16, _>(column_ordinal)?)),
            "INT" | "INTEGER" => ArrayKey::Long(i64::from(self.try_get::<i32, _>(column_ordinal)?)),
            "BIGINT" => ArrayKey::Long(self.try_get::<i64, _>(column_ordinal)?),

            "DECIMAL" | "NUMERIC" | "MONEY" | "SMALLMONEY" => {
                ArrayKey::String(self.try_get::<String, _>(column_ordinal)?)
            }

            "CHAR" | "VARCHAR" | "TEXT" | "NCHAR" | "NVARCHAR" | "NTEXT" | "XML"
            | "UNIQUEIDENTIFIER" => ArrayKey::String(self.try_get::<String, _>(column_ordinal)?),

            "DATE" | "TIME" | "DATETIME" | "DATETIME2" | "SMALLDATETIME" | "DATETIMEOFFSET" => {
                ArrayKey::String(self.try_get::<String, _>(column_ordinal)?)
            }

            other => bail!("unsupported type for array key: {}", other),
        })
    }
}
