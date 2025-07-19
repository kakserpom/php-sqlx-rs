use crate::ast::Settings;
use crate::param_value::ParameterValue;
use anyhow::bail;

impl ParameterValue {
    /// Escapes `%`, `_`, and `\` characters in the input string by prefixing them with a backslash,
    /// making it safe to use in SQL `LIKE` patterns.
    ///
    /// # Returns
    /// A new `String` where all occurrences of `%`, `_`, and `\` are escaped.
    pub fn meta_quote_like(&self) -> anyhow::Result<String> {
        let Self::String(input) = self else {
            bail!("meta_quote_like called on non-string parameter");
        };

        let mut escaped = String::with_capacity(input.len());

        for c in input.chars() {
            if c == '%' || c == '_' || c == '\\' {
                escaped.push('\\');
            }
            escaped.push(c);
        }

        Ok(escaped)
    }

    /// Quotes the value as a SQL literal string, number, or boolean,
    /// escaping special characters where appropriate, based on the given `Settings`.
    ///
    /// This is used for generating safe inline SQL expressions (e.g. for debugging or logging),
    /// but **should not** be used for query execution — always use placeholders and bind values instead.
    ///
    /// # Errors
    /// Return Err if the value is a structured clause or an unsupported type.
    pub fn quote(&self, settings: &Settings) -> anyhow::Result<String> {
        fn escape_sql_string(input: &str, settings: &Settings) -> String {
            let mut out = String::with_capacity(input.len() + 8);
            if settings.strings_as_ntext {
                out.push_str("N'");
            } else {
                out.push('\'');
            }
            for c in input.chars() {
                match c {
                    '\'' => out.push_str("''"),
                    '\\' if settings.escape_backslash => out.push_str("\\\\"),
                    '\0' => out.push_str("\\0"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\x1A' => out.push_str("\\x1A"), // Ctrl+Z
                    _ => out.push(c),
                }
            }
            out.push('\'');
            out.shrink_to_fit();
            out
        }
        Ok(match self {
            Self::Null => "NULL".to_string(),

            Self::Int(i) => i.to_string(),
            Self::Float(f) => f.to_string(),
            Self::Json(pv) => escape_sql_string(pv.to_json()?.as_str(), settings),

            Self::Bool(b) => String::from(if settings.booleans_as_literals {
                if *b { "TRUE" } else { "FALSE" }
            } else if *b {
                "1"
            } else {
                "0"
            }),

            Self::String(s) => escape_sql_string(s, settings),

            Self::Array(values) => {
                let elements = values
                    .iter()
                    .map(|v| v.quote(settings))
                    .collect::<anyhow::Result<Vec<_>>>()?
                    .join(", ");
                format!("({elements})")
            }

            Self::Object(obj) => escape_sql_string(
                &serde_json::to_string(obj)
                    .map_err(|e| anyhow::anyhow!("JSON serialization error: {}", e))?,
                settings,
            ),

            Self::ByClauseRendered(_)
            | Self::SelectClauseRendered(_)
            | Self::PaginateClauseRendered(_)
            | Self::Builder(_) => {
                bail!("Cannot quote a clause as a value")
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::Settings;
    use crate::param_value::ParameterValue;
    use std::collections::HashMap;

    #[test]
    fn test_meta_quote_like() {
        let value = ParameterValue::String("%_\\abc".to_string());
        let escaped = value.meta_quote_like().unwrap();
        assert_eq!(escaped, "\\%\\_\\\\abc");

        let value = ParameterValue::String("no-special".to_string());
        let escaped = value.meta_quote_like().unwrap();
        assert_eq!(escaped, "no-special");

        let value = ParameterValue::Int(42);
        assert!(value.meta_quote_like().is_err());
    }

    #[test]
    fn test_quote_null() {
        let settings = Settings::default();
        let value = ParameterValue::Null;
        assert_eq!(value.quote(&settings).unwrap(), "NULL");
    }

    #[test]
    fn test_quote_numbers() {
        let settings = Settings::default();
        assert_eq!(ParameterValue::Int(123).quote(&settings).unwrap(), "123");
        assert_eq!(
            ParameterValue::Float(3.14).quote(&settings).unwrap(),
            "3.14"
        );
    }

    #[test]
    fn test_quote_bool() {
        let mut settings = Settings::default();

        settings.booleans_as_literals = false;
        assert_eq!(ParameterValue::Bool(true).quote(&settings).unwrap(), "1");
        assert_eq!(ParameterValue::Bool(false).quote(&settings).unwrap(), "0");

        settings.booleans_as_literals = true;
        assert_eq!(ParameterValue::Bool(true).quote(&settings).unwrap(), "TRUE");
        assert_eq!(
            ParameterValue::Bool(false).quote(&settings).unwrap(),
            "FALSE"
        );
    }

    #[test]
    fn test_quote_string() {
        let mut settings = Settings::default();

        settings.strings_as_ntext = false;
        let quoted = ParameterValue::String("O'Reilly".into())
            .quote(&settings)
            .unwrap();
        assert_eq!(quoted, "'O''Reilly'");

        settings.strings_as_ntext = true;
        let quoted = ParameterValue::String("line1\nline2".into())
            .quote(&settings)
            .unwrap();
        assert_eq!(quoted, "N'line1\\nline2'");
    }

    #[test]
    fn test_quote_array() {
        let settings = Settings::default();

        let value = ParameterValue::Array(vec![
            ParameterValue::Int(1),
            ParameterValue::String("x".into()),
        ]);
        let quoted = value.quote(&settings).unwrap();
        assert_eq!(quoted, "(1, 'x')");
    }

    #[test]
    fn test_quote_json_object() {
        let settings = Settings::default();

        let mut map = HashMap::new();
        map.insert("key".into(), ParameterValue::String("val".into()));
        let value = ParameterValue::Object(map);
        let quoted = value.quote(&settings).unwrap();
        assert_eq!(quoted, "'{\"key\":\"val\"}'");
    }
}
