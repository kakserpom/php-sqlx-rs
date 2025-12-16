//! SQL indentation utilities for php-sqlx.
//!
//! This module provides the [`IndentSql`] trait for adding consistent indentation
//! to SQL queries, primarily used when formatting subqueries within query builders.

/// Trait for adding indentation to strings.
///
/// This is useful for formatting SQL queries or other text output in a readable way.
pub trait IndentSql {
    /// Adds the specified number of spaces at the beginning of each line in the string.
    ///
    /// # Arguments
    ///
    /// * `indent` - The number of spaces to add at the beginning of each line.
    ///
    /// # Returns
    ///
    /// A new string with the specified indentation applied to each line.
    fn indent_sql(&self, query: bool) -> String;
}

impl<T: AsRef<str>> IndentSql for T {
    fn indent_sql(&self, query: bool) -> String {
        let indent = 3;
        let s = self.as_ref();
        // If indent is 0 or string is empty, return the original string
        if s.is_empty() {
            return String::new();
        }

        // Count the number of lines to calculate capacity
        let line_count = s.lines().take(2).count();

        // If there's only one line, optimize for the common case
        if line_count == 1 {
            if query {
                return format!("\n{}{s}", " ".repeat(indent));
            }
            return s.to_string();
        }

        // Calculate capacity: original length + (indent * number of non-empty lines) + potential newlines
        let non_empty_lines = s.lines().filter(|line| !line.is_empty()).count();
        let capacity = s.len() + (indent * non_empty_lines);

        // Pre-allocate the result string with the calculated capacity
        let mut result = String::with_capacity(capacity);

        // Process each line
        for (i, line) in s.lines().enumerate() {
            // Add newline before lines (except the first one)
            if i > 0 || query {
                result.push('\n');
            }

            // Only add indentation to non-empty lines
            let line = line.trim_end();
            if !line.is_empty() && (i > 0 || query) {
                result.extend(std::iter::repeat_n(' ', indent));
            }

            // Add the line content
            result.push_str(line);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indent_string() {
        // Test with single line
        let s = String::from("SELECT * FROM users");
        assert_eq!(s.indent_sql(false), "SELECT * FROM users");

        // Test with multiple lines
        let s = String::from("SELECT *\nFROM users\nWHERE id = 1");
        assert_eq!(
            s.indent_sql(false),
            "SELECT *\n   FROM users\n   WHERE id = 1"
        );

        // Test with empty lines
        let s = String::from("SELECT *\n\nFROM users");
        assert_eq!(s.indent_sql(false), "SELECT *\n\n   FROM users");

        // Test with zero indent
        let s = String::from("SELECT * FROM users");
        assert_eq!(s.indent_sql(false), "SELECT * FROM users");
    }
}
