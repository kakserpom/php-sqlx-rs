//! Validated SQL identifier value type.
//!
//! `Sqlx\Identifier` is a low-level, validated value type for table/column names
//! in dynamic SQL. Construction fails unless the value is a safe identifier
//! (starts with a letter or underscore, then only alphanumerics/underscores) and,
//! when an allowlist is supplied, is one of its members.
//!
//! It is the single primitive that replaces hand-rolled
//! `preg_replace('/[^\w."]+/', ...)` column sanitizers: bind an `Identifier` to a
//! placeholder and it renders as a properly quoted identifier for the driver's
//! dialect (`"col"`, `` `col` ``, or `[col]`), never as a bound string literal.
//!
//! # PHP Usage
//!
//! ```php
//! // Charset validation — throws on anything unsafe:
//! $col = Sqlx\Identifier::from($_GET['sort']);            // e.g. "created_at"
//!
//! // Allowlist validation — throws unless the value is permitted:
//! $col = Sqlx\Identifier::from($_GET['sort'], ['created_at', 'name']);
//!
//! $rows = $driver->queryAll("SELECT * FROM users ORDER BY :col", ['col' => $col]);
//! // renders: SELECT * FROM users ORDER BY "created_at"
//! ```

use crate::error::Error as SqlxError;
use crate::utils::ident::is_valid_ident;
use ext_php_rs::prelude::ModuleBuilder;
use ext_php_rs::{php_class, php_impl};

/// Registers the `Identifier` class with the PHP module builder.
pub fn build(module: ModuleBuilder) -> ModuleBuilder {
    module.class::<Identifier>()
}

/// A validated SQL identifier (table or column name).
#[php_class]
#[php(name = "Sqlx\\Identifier")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identifier {
    /// The validated identifier (always a safe, unquoted name).
    pub(crate) name: String,
}

impl Identifier {
    /// Validates `name` and, if `allowed` is given, checks membership.
    ///
    /// # Errors
    /// Returns [`SqlxError::InvalidIdentifier`] if `name` is not a safe SQL
    /// identifier or is absent from a non-empty allowlist.
    pub fn _new(name: String, allowed: Option<Vec<String>>) -> crate::error::Result<Self> {
        if !is_valid_ident(&name) {
            return Err(SqlxError::InvalidIdentifier { value: name });
        }
        if let Some(allowed) = allowed
            && !allowed.iter().any(|candidate| candidate == &name)
        {
            return Err(SqlxError::InvalidIdentifier { value: name });
        }
        Ok(Self { name })
    }
}

#[php_impl]
impl Identifier {
    /// Constructs a validated identifier.
    ///
    /// # Arguments
    /// - `name`: The identifier to validate.
    /// - `allowed`: Optional allowlist; when given, `name` must be a member.
    ///
    /// # Exceptions
    /// Throws if `name` is not a safe identifier or is not in `allowed`.
    pub fn __construct(name: String, allowed: Option<Vec<String>>) -> crate::error::Result<Self> {
        Self::_new(name, allowed)
    }

    /// Static factory mirroring the constructor: `Identifier::from($name)`.
    ///
    /// # Exceptions
    /// Throws if `name` is not a safe identifier or is not in `allowed`.
    pub fn from(name: String, allowed: Option<Vec<String>>) -> crate::error::Result<Self> {
        Self::_new(name, allowed)
    }

    /// Returns the validated identifier string (unquoted).
    #[must_use]
    pub fn value(&self) -> String {
        self.name.clone()
    }

    /// Returns the validated identifier string (unquoted).
    #[must_use]
    pub fn __to_string(&self) -> String {
        self.name.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_identifiers() {
        assert_eq!(
            Identifier::_new("created_at".to_string(), None)
                .unwrap()
                .name,
            "created_at"
        );
        assert!(Identifier::_new("_private".to_string(), None).is_ok());
    }

    #[test]
    fn rejects_unsafe_identifiers() {
        assert!(Identifier::_new("users; DROP TABLE users".to_string(), None).is_err());
        assert!(Identifier::_new("1col".to_string(), None).is_err());
        assert!(Identifier::_new(String::new(), None).is_err());
        assert!(Identifier::_new("a.b".to_string(), None).is_err());
    }

    #[test]
    fn enforces_allowlist() {
        let allowed = vec!["name".to_string(), "created_at".to_string()];
        assert!(Identifier::_new("name".to_string(), Some(allowed.clone())).is_ok());
        // Valid charset but not in the allowlist.
        assert!(Identifier::_new("email".to_string(), Some(allowed)).is_err());
    }
}
