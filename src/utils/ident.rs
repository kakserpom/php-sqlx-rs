/// Validates whether a string is a valid SQL identifier.
///
/// A valid identifier must:
/// - Be non-empty
/// - Start with an alphabetic character or underscore
/// - Contain only alphanumeric characters or underscores
#[must_use]
pub fn is_valid_ident(name: &str) -> bool {
    !name.is_empty()
        && name.starts_with(|c: char| c.is_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_identifiers() {
        assert!(is_valid_ident("users"));
        assert!(is_valid_ident("Users"));
        assert!(is_valid_ident("_private"));
        assert!(is_valid_ident("user_table"));
        assert!(is_valid_ident("table123"));
        assert!(is_valid_ident("a"));
        assert!(is_valid_ident("_"));
        assert!(is_valid_ident("__double_underscore__"));
    }

    #[test]
    fn test_invalid_identifiers() {
        // Empty string
        assert!(!is_valid_ident(""));

        // Starts with number
        assert!(!is_valid_ident("123table"));
        assert!(!is_valid_ident("1"));

        // Contains special characters
        assert!(!is_valid_ident("user-table"));
        assert!(!is_valid_ident("user.table"));
        assert!(!is_valid_ident("user table"));
        assert!(!is_valid_ident("user;DROP TABLE"));

        // SQL injection attempts
        assert!(!is_valid_ident("users; DROP TABLE users;--"));
        assert!(!is_valid_ident("users'--"));
        assert!(!is_valid_ident("users\""));
        assert!(!is_valid_ident("users`"));
    }
}
