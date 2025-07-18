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
