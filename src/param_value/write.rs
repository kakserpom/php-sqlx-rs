use crate::ast::Settings;
use crate::param_value::ParameterValue;
use std::fmt::Write;

pub trait ParamVecWriteSqlTo {
    /// Trait for writing a vector of parameter values into a SQL string with placeholders,
    /// collecting the values into an output vector for binding later.
    ///
    /// This is used during SQL rendering to serialize parameter expressions like `IN (?, ?, ?)`
    /// while respecting engine-specific placeholder syntax and limits.
    fn write_sql_to(
        &self,
        sql: &mut String,
        out_vals: &mut Vec<ParameterValue>,
        settings: &Settings,
    ) -> anyhow::Result<()>;
}

impl ParameterValue {
    /// Internal helper that appends a single SQL placeholder (e.g. `$1`, `@p1`, or `?`) or literal.
    ///
    /// If `max_placeholders` is exceeded, falls back to inlined quoting via `quote()`.
    ///
    /// # Arguments
    /// - `sql`: SQL string buffer
    /// - `out_vals`: collected bindable values
    /// - `settings`: placeholder style and limits
    #[inline]
    fn write_placeholder_to(
        &self,
        sql: &mut String,
        out_vals: &mut Vec<Self>,
        settings: &Settings,
    ) -> anyhow::Result<()> {
        if out_vals.len() < settings.max_placeholders {
            out_vals.push(self.clone());
            if settings.placeholder_dollar_sign {
                write!(sql, "${}", out_vals.len())?;
            } else if settings.placeholder_at_sign {
                write!(sql, "@p{}", out_vals.len())?;
            } else {
                sql.push('?');
            }
        } else {
            sql.push_str(self.quote(settings)?.as_str());
        }
        Ok(())
    }

    /// Serializes a parameter value into the SQL stream, respecting placeholder style and clause rendering.
    ///
    /// Automatically handles:
    /// - Arrays: expanded into `?, ?, ...`
    /// - Rendered clauses: directly written into SQL
    /// - All other values: serialized via placeholder or literal
    ///
    /// # Arguments
    /// - `sql`: mutable SQL buffer to write into
    /// - `out_vals`: placeholder value collector
    /// - `settings`: SQL rendering and placeholder configuration
    #[inline]
    pub(crate) fn write_sql_to(
        &self,
        sql: &mut String,
        out_vals: &mut Vec<ParameterValue>,
        settings: &Settings,
    ) -> anyhow::Result<()> {
        match self {
            ParameterValue::SelectClauseRendered(scr) => {
                scr.write_sql_to(sql, settings)?;
            }
            ParameterValue::ByClauseRendered(by) => {
                by.write_sql_to(sql, settings)?;
            }
            ParameterValue::Array(arr) => {
                out_vals.reserve_exact(arr.len());
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        sql.push_str(", ");
                    }
                    item.write_placeholder_to(sql, out_vals, settings)?;
                }
            }
            _ => {
                self.write_placeholder_to(sql, out_vals, settings)?;
            }
        }
        Ok(())
    }
}

impl ParamVecWriteSqlTo for Vec<ParameterValue> {
    /// Writes a vector of parameter values to the SQL string with placeholders.
    ///
    /// Each value will be rendered as a separate placeholder or literal expression,
    /// joined by commas. This is commonly used for `IN (?)` expansion.
    #[inline]
    fn write_sql_to(
        &self,
        sql: &mut String,
        out_vals: &mut Vec<ParameterValue>,
        settings: &Settings,
    ) -> anyhow::Result<()> {
        out_vals.reserve_exact(self.len());
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            item.write_placeholder_to(sql, out_vals, settings)?;
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Settings;
    use crate::param_value::ParameterValue;

    #[test]
    fn write_sql_single_placeholder() {
        let mut settings = Settings::default();
        settings.max_placeholders = 65535;
        let mut sql = String::new();
        let mut out_vals = Vec::new();

        ParameterValue::Int(42)
            .write_sql_to(&mut sql, &mut out_vals, &settings)
            .unwrap();

        assert_eq!(sql, "?");
        assert_eq!(out_vals.len(), 1);
        assert_eq!(out_vals[0], ParameterValue::Int(42));
    }

    #[test]
    fn write_sql_array_placeholders() {
        let mut settings = Settings::default();
        settings.max_placeholders = 65535;
        let mut sql = String::new();
        let mut out_vals = Vec::new();

        ParameterValue::Array(vec![1.into(), 2.into(), 3.into()])
            .write_sql_to(&mut sql, &mut out_vals, &settings)
            .unwrap();

        assert_eq!(sql, "?, ?, ?");
        assert_eq!(out_vals.len(), 3);
    }

    #[test]
    fn write_sql_fallback_to_literal() {
        let mut settings = Settings::default();
        settings.max_placeholders = 0; // force fallback to literal

        let mut sql = String::new();
        let mut out_vals = Vec::new();

        ParameterValue::String("O'Reilly".into())
            .write_sql_to(&mut sql, &mut out_vals, &settings)
            .unwrap();

        assert_eq!(sql, "'O''Reilly'");
        assert!(out_vals.is_empty());
    }

    #[test]
    fn write_sql_mixed_values() {
        let mut settings = Settings::default();
        settings.max_placeholders = 2;
        let mut sql = String::new();
        let mut out_vals = Vec::new();

        let params = vec![
            ParameterValue::Int(1),
            ParameterValue::String("foo".into()),
            ParameterValue::Bool(true),
        ];

        params
            .write_sql_to(&mut sql, &mut out_vals, &settings)
            .unwrap();

        assert_eq!(sql, "?, ?, 1");
        assert_eq!(out_vals.len(), 2);
    }
}
