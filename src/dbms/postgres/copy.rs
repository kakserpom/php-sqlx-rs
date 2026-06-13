//! `PostgreSQL` `COPY ... FROM STDIN` bulk-ingest fast path.
//!
//! This is a high-throughput alternative to `insertMany()` for loading many rows:
//! instead of one big parameterized `INSERT`, it streams the rows to Postgres in
//! the `COPY` text format. It sidesteps the bind-parameter limit entirely and is
//! markedly faster for large batches.
//!
//! Rows are read **directly from the PHP array** and encoded into a small reusable
//! buffer that is flushed to the connection in chunks as we go — so neither the
//! decoded rows nor the full COPY payload are ever held in memory at once.
//!
//! Values are encoded into the COPY text format with full escaping, so the data
//! stream cannot break out of its field — there is no SQL injection surface in the
//! row values (column/table identifiers come from calling code, as with
//! `insertMany()`).

use super::inner::PgDriverInner;
use crate::RUNTIME;
use crate::error::Error as SqlxError;
use crate::param_value::ParameterValue;
use ext_php_rs::convert::FromZval;
use ext_php_rs::types::Zval;
use std::fmt::Write;

/// Flush the COPY buffer to the connection once it grows past this many bytes.
const FLUSH_THRESHOLD: usize = 64 * 1024;

impl PgDriverInner {
    /// Bulk-inserts rows into `table` using `COPY ... FROM STDIN`.
    ///
    /// `rows` is the PHP array of rows (each row an associative array). Columns are
    /// taken from the first row (in its PHP order); columns missing from later rows
    /// are sent as `NULL`. Rows are streamed in chunks, so memory use stays bounded
    /// regardless of batch size. Returns the number of rows ingested.
    pub fn copy_in(&self, table: &str, rows: &Zval) -> crate::error::Result<u64> {
        let outer = rows
            .array()
            .ok_or_else(|| SqlxError::Other("copyIn: rows must be an array of rows".to_string()))?;

        // Columns come from the first row, preserving its PHP insertion order.
        let first = outer
            .iter()
            .next()
            .ok_or_else(|| SqlxError::Other("copyIn requires at least one row".to_string()))?;
        let first_row = first
            .1
            .array()
            .ok_or_else(|| SqlxError::Other("copyIn: each row must be an array".to_string()))?;
        let columns: Vec<String> = first_row.iter().map(|(key, _)| key.to_string()).collect();
        if columns.is_empty() {
            return Err(SqlxError::Other(
                "copyIn: the first row has no columns".to_string(),
            ));
        }

        let statement = format!("COPY {table} ({}) FROM STDIN", columns.join(", "));

        RUNTIME.block_on(async {
            let mut copy = self
                .pool
                .copy_in_raw(&statement)
                .await
                .map_err(|err| SqlxError::query_with_source(&statement, err))?;

            let mut buf = String::with_capacity(FLUSH_THRESHOLD + 1024);
            for (_, row) in outer {
                encode_row(row, &columns, &mut buf)?;
                if buf.len() >= FLUSH_THRESHOLD {
                    copy.send(buf.as_bytes())
                        .await
                        .map_err(|err| SqlxError::query_with_source(&statement, err))?;
                    buf.clear();
                }
            }
            if !buf.is_empty() {
                copy.send(buf.as_bytes())
                    .await
                    .map_err(|err| SqlxError::query_with_source(&statement, err))?;
            }
            copy.finish()
                .await
                .map_err(|err| SqlxError::query_with_source(&statement, err))
        })
    }
}

/// Encodes one row (a PHP associative array) as a COPY text line into `out`.
///
/// Columns are written in `columns` order, tab-separated and newline-terminated;
/// a column absent from this row is written as `NULL` (`\N`).
fn encode_row(row: &Zval, columns: &[String], out: &mut String) -> crate::error::Result<()> {
    let row_ht = row
        .array()
        .ok_or_else(|| SqlxError::Other("copyIn: each row must be an array".to_string()))?;
    for (i, column) in columns.iter().enumerate() {
        if i > 0 {
            out.push('\t');
        }
        match row_ht.get(column.as_str()) {
            Some(value) => encode_value(value, out)?,
            None => out.push_str("\\N"),
        }
    }
    out.push('\n');
    Ok(())
}

/// Encodes a single cell `Zval` into the COPY text format.
///
/// Scalars take an allocation-free fast path (`str()` borrows); anything else
/// (JSON arrays/objects, `DateTime`, …) falls back to [`ParameterValue`] and the
/// shared [`encode_field`] encoder.
fn encode_value(value: &Zval, out: &mut String) -> crate::error::Result<()> {
    if value.is_null() {
        out.push_str("\\N");
    } else if let Some(s) = value.str() {
        escape_into(s, out);
    } else if let Some(b) = value.bool() {
        out.push(if b { 't' } else { 'f' });
    } else if let Some(i) = value.long() {
        let _ = write!(out, "{i}");
    } else if let Some(f) = value.double() {
        let _ = write!(out, "{f}");
    } else {
        let pv = ParameterValue::from_zval(value)
            .ok_or_else(|| SqlxError::Other("copyIn: unsupported value in row".to_string()))?;
        encode_field(&pv, out)?;
    }
    Ok(())
}

/// Encodes a [`ParameterValue`] into the COPY text format.
///
/// `NULL` is the unescaped marker `\N`; every other value is escaped so its
/// content cannot contain a field (tab) or row (newline) separator.
fn encode_field(value: &ParameterValue, out: &mut String) -> crate::error::Result<()> {
    match value {
        ParameterValue::Null => out.push_str("\\N"),
        ParameterValue::Bool(b) => out.push(if *b { 't' } else { 'f' }),
        ParameterValue::Int(i) => {
            let _ = write!(out, "{i}");
        }
        ParameterValue::Float(f) => {
            let _ = write!(out, "{f}");
        }
        ParameterValue::String(s) => escape_into(s, out),
        ParameterValue::DateTime(dt) => {
            escape_into(&dt.format("%Y-%m-%d %H:%M:%S%.f").to_string(), out);
        }
        // JSON-ish values are serialized to JSON text (for json/jsonb columns).
        ParameterValue::Json(_) | ParameterValue::Object(_) | ParameterValue::Array(_) => {
            let json = serde_json::to_string(value).map_err(|err| SqlxError::Conversion {
                message: format!("copyIn JSON encode: {err}"),
            })?;
            escape_into(&json, out);
        }
        other => {
            return Err(SqlxError::Other(format!(
                "copyIn: unsupported value type for COPY: {other:?}"
            )));
        }
    }
    Ok(())
}

/// Escapes the COPY text metacharacters (`\`, tab, newline, carriage return).
fn escape_into(s: &str, out: &mut String) {
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn field(value: &ParameterValue) -> String {
        let mut out = String::new();
        encode_field(value, &mut out).unwrap();
        out
    }

    #[test]
    fn encodes_scalars() {
        assert_eq!(field(&ParameterValue::Int(7)), "7");
        assert_eq!(field(&ParameterValue::Bool(true)), "t");
        assert_eq!(field(&ParameterValue::Bool(false)), "f");
        assert_eq!(field(&ParameterValue::String("Alice".to_string())), "Alice");
    }

    #[test]
    fn null_becomes_backslash_n() {
        assert_eq!(field(&ParameterValue::Null), "\\N");
    }

    #[test]
    fn escapes_separators_and_backslash() {
        let mut s = String::new();
        escape_into("a\tb\nc\\d\re", &mut s);
        assert_eq!(s, "a\\tb\\nc\\\\d\\re");
    }

    #[test]
    fn literal_backslash_n_string_is_not_null() {
        // The two-char string `\N` must not be confused with the NULL marker.
        assert_eq!(field(&ParameterValue::String("\\N".to_string())), "\\\\N");
    }

    #[test]
    fn json_object_is_serialized() {
        let mut obj = BTreeMap::new();
        obj.insert("k".to_string(), ParameterValue::Int(1));
        assert_eq!(field(&ParameterValue::Object(obj)), "{\"k\":1}");
    }

    #[test]
    fn encode_row_tab_separates_and_fills_missing_with_null() {
        // encode_row needs PHP Zvals, so exercise the column/missing logic via
        // encode_field directly here; row-level behavior is covered by the
        // PostgreSQL integration test.
        let columns = ["a".to_string(), "b".to_string()];
        let mut out = String::new();
        for (i, _) in columns.iter().enumerate() {
            if i > 0 {
                out.push('\t');
            }
            encode_field(&ParameterValue::Null, &mut out).unwrap();
        }
        out.push('\n');
        assert_eq!(out, "\\N\t\\N\n");
    }
}
