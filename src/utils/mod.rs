//! Utility functions and types for php-sqlx.
//!
//! This module provides various helper functions used throughout the crate:
//!
//! - [`ident`]: SQL identifier validation
//! - [`hashmap_fold`]: Helpers for folding query results into PHP hash maps
//! - [`indent_sql`]: SQL formatting with indentation
//! - [`strip_prefix`]: Case-insensitive prefix stripping for SQL parsing
//! - [`types`]: Common type definitions

pub mod hashmap_fold;
pub mod ident;
pub mod indent_sql;
pub mod strip_prefix;
pub mod types;

/// Dynamically registers a PHP class as implementing an interface.
///
/// This is used at runtime to make driver classes implement `DriverInterface`
/// without requiring compile-time knowledge of all implementations.
///
/// # Arguments
/// - `class`: The fully qualified PHP class name (e.g., `"Sqlx\\PgDriver"`)
/// - `interface`: The fully qualified interface name (e.g., `"Sqlx\\DriverInterface"`)
///
/// # Returns
/// `Some(())` if successful, `None` if the class or interface couldn't be found.
pub fn adhoc_php_class_implements(class: &str, interface: &str) -> Option<()> {
    use ext_php_rs::ffi::zend_do_implement_interface;
    use ext_php_rs::types::ZendStr;
    use ext_php_rs::zend::ClassEntry;
    use ext_php_rs::zend::ExecutorGlobals;
    use std::ptr;
    ExecutorGlobals::get().class_table()?;
    let mut class = ZendStr::new(class, false);
    let interface = ClassEntry::try_find(interface)?;
    unsafe {
        zend_do_implement_interface(
            ext_php_rs::ffi::zend_lookup_class_ex(&raw mut *class, ptr::null_mut(), 0).as_mut()?,
            ptr::from_ref(interface).cast_mut(),
        );
    };
    Some(())
}
