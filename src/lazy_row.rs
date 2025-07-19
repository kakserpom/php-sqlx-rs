use crate::conversion::json_into_zval;

use ext_php_rs::boxed::ZBox;
use ext_php_rs::convert::FromZval;
use ext_php_rs::ffi::zend_array;
use ext_php_rs::prelude::*;
use ext_php_rs::types::{ArrayKey, ZendClassObject, Zval};
use ext_php_rs::zend::ce;
use ext_php_rs::{php_class, php_impl};
use std::cell::RefCell;

/// Threshold above which JSON is parsed lazily (in bytes).
pub const LAZY_ROW_JSON_SIZE_THRESHOLD: usize = 4096;

#[php_class]
#[php(name = "Sqlx\\LazyRow")]
#[php(implements(ce = ce::arrayaccess, stub = "\\ArrayAccess"))]
/// A PHP-accessible wrapper around a `zend_array` that lazily decodes JSON values.
///
/// Implements `ArrayAccess` so that columns can be accessed as array entries.
pub struct LazyRow {
    pub(crate) array: RefCell<ZBox<zend_array>>,
}

impl LazyRow {
    /// Create a new `LazyRow` from an existing Zend array.
    ///
    /// # Arguments
    ///
    /// * `row` – A boxed `zend_array` representing the row data.
    pub fn new(row: ZBox<zend_array>) -> Self {
        Self {
            array: RefCell::new(row),
        }
    }
}

#[php_impl]
impl LazyRow {
    /// Checks whether a column exists in the row.
    ///
    /// # Arguments
    ///
    /// * `offset` – The column name as a `Zval` (expected to be a string).
    ///
    /// # Returns
    ///
    /// `Ok(true)` if the column exists, `Ok(false)` otherwise, or an error if the offset isn't a string.
    pub fn offset_exists(&self, offset: &'_ Zval) -> PhpResult<bool> {
        Ok(self
            .array
            .borrow()
            .get(offset.str().ok_or("Expected string offset")?)
            .is_some())
    }

    /// Magic getter for property access in PHP (`$row->column`).
    ///
    /// Lazily decodes JSON-wrapped values if needed and replaces the placeholder object
    /// with the actual decoded `Zval`.
    ///
    /// # Arguments
    ///
    /// * `name` – The column name.
    ///
    /// # Errors
    ///
    /// Returns a `PhpException` if the column is not found or offset is not a string.
    pub fn __get(&self, name: &str) -> PhpResult<Zval> {
        let mut ht = self.array.borrow_mut();
        let value = ht
            .get_mut(name)
            .ok_or_else(|| PhpException::from("column not found"))?;

        if let Some(obj) = value.object() {
            if let Some(lazy_row_json) = ZendClassObject::<LazyRowJson>::from_zend_obj(obj) {
                // Take the raw JSON, decode it, and replace the stored placeholder
                let zval = LazyRowJson::take_zval(lazy_row_json)?;
                let clone = value.shallow_clone();
                ht.insert(name, zval)?;
                Ok(clone)
            } else {
                Ok(value.shallow_clone())
            }
        } else {
            Ok(value.shallow_clone())
        }
    }

    /// ArrayAccess getter (`$row[$column]`).
    ///
    /// Performs the same lazy JSON decoding logic as `__get`.
    pub fn offset_get(&self, offset: &'_ Zval) -> PhpResult<Zval> {
        let mut ht = self.array.borrow_mut();
        let key = ArrayKey::from_zval(offset).ok_or("Illegal offset")?;
        let value = ht
            .get_mut(key.clone())
            .ok_or_else(|| PhpException::from("column not found"))?;

        if let Some(obj) = value.object() {
            if let Some(lazy_row_json) = ZendClassObject::<LazyRowJson>::from_zend_obj(obj) {
                let zval = LazyRowJson::take_zval(lazy_row_json)?;
                let clone = value.shallow_clone();
                ht.insert(key, zval)?;
                Ok(clone)
            } else {
                Ok(value.shallow_clone())
            }
        } else {
            Ok(value.shallow_clone())
        }
    }

    /// ArrayAccess setter (`$row[$key] = $value`).
    ///
    /// Inserts or updates the given key with the provided `Zval`.
    ///
    /// # Exceptions
    ///
    /// Throws an exception if insertion fails or if the offset isn't a string.
    pub fn offset_set(&mut self, offset: &'_ Zval, value: &'_ Zval) -> PhpResult {
        let key = ArrayKey::from_zval(offset).ok_or("Illegal offset")?;
        self.array
            .borrow_mut()
            .insert(key, value.shallow_clone())
            .map_err(|_| PhpException::from("unable to set"))
    }

    /// ArrayAccess unsetter (`unset($row[$key])`).
    ///
    /// Unsetting values is not supported and always returns an error.
    pub fn offset_unset(&mut self, offset: &'_ Zval) -> PhpResult {
        let key = ArrayKey::from_zval(offset).ok_or("Illegal offset")?;
        let _ = self.array.borrow_mut().remove(key);
        Ok(())
    }
}

#[php_class]
#[php(name = "Sqlx\\LazyRowJson")]
/// A helper PHP class that holds raw JSON bytes for lazy decoding.
///
/// When accessed, it will be parsed into a PHP value on demand.
pub struct LazyRowJson {
    pub(crate) raw: RefCell<Vec<u8>>,
    pub(crate) assoc: bool,
}

impl LazyRowJson {
    /// Construct a new `LazyRowJson` from a byte slice.
    ///
    /// # Arguments
    ///
    /// * `slice` – Raw JSON bytes.
    /// * `assoc` – Whether to decode objects as associative arrays.
    pub(crate) fn new(slice: &[u8], assoc: bool) -> Self {
        Self {
            raw: RefCell::new(slice.to_vec()),
            assoc,
        }
    }
}

#[php_impl]
impl LazyRowJson {
    /// Decode the stored JSON into a PHP `Zval`.
    ///
    /// Uses either `simd-json` or `serde_json` depending on build features.
    ///
    /// # Errors
    ///
    /// Propagates JSON parsing exceptions.
    pub fn take_zval(self_: &ZendClassObject<LazyRowJson>) -> anyhow::Result<Zval> {
        #[cfg(feature = "simd-json")]
        return json_into_zval(
            simd_json::from_slice::<serde_json::Value>(self_.raw.borrow_mut().as_mut_slice())?,
            self_.assoc,
        );
        #[cfg(not(feature = "simd-json"))]
        json_into_zval(serde_json::from_slice(&self_.raw.borrow())?, self_.assoc)
    }
}

/// Register the `LazyRow` and `LazyRowJson` classes with the PHP extension.
///
/// # Returns
///
/// The updated `ModuleBuilder`.
pub fn build(module: ModuleBuilder) -> ModuleBuilder {
    module.class::<LazyRow>().class::<LazyRowJson>()
}
