use crate::conversion::json_into_zval;

use ext_php_rs::boxed::ZBox;
use ext_php_rs::convert::FromZval;
use ext_php_rs::ffi::zend_array;
use ext_php_rs::prelude::*;
use ext_php_rs::types::{ArrayKey, ZendClassObject, Zval};
use ext_php_rs::zend::ce;
use ext_php_rs::{php_class, php_impl};
use std::cell::{Cell, RefCell};

/// Threshold above which JSON is parsed lazily (in bytes).
pub const LAZY_ROW_JSON_SIZE_THRESHOLD: usize = 4096;

#[php_class]
#[php(name = "Sqlx\\LazyRow")]
#[php(extends(ce = ce::stdclass, stub = "\\stdClass"))]
#[php(implements(ce = ce::arrayaccess, stub = "\\ArrayAccess"))]
#[php(implements(ce = ce::iterator, stub = "\\Iterator"))]
#[php(implements("\\JsonSerializable"))]
/// A PHP-accessible wrapper around a `zend_array` that lazily decodes JSON values.
///
/// Extends `stdClass` for compatibility with code expecting objects.
/// Implements `ArrayAccess` so that columns can be accessed as array entries.
/// Implements `Iterator` so that columns can be iterated with `foreach`.
/// Implements `JsonSerializable` so that `json_encode()` works correctly.
pub struct LazyRow {
    pub(crate) array: RefCell<ZBox<zend_array>>,
    /// Current iterator position (index into the array keys)
    iter_pos: Cell<usize>,
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
            iter_pos: Cell::new(0),
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

    /// Magic isset for checking if a property exists (`isset($row->column)`).
    ///
    /// # Arguments
    ///
    /// * `name` – The column name.
    ///
    /// # Returns
    ///
    /// `true` if the column exists, `false` otherwise.
    pub fn __isset(&self, name: &str) -> bool {
        self.array.borrow().get(name).is_some()
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
                let zval = LazyRowJson::decode_json(lazy_row_json)?;
                let clone = zval.shallow_clone();
                ht.insert(name, zval)?;
                Ok(clone)
            } else {
                Ok(value.shallow_clone())
            }
        } else {
            Ok(value.shallow_clone())
        }
    }

    /// `ArrayAccess` getter (`$row[$column]`).
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
                let zval = LazyRowJson::decode_json(lazy_row_json)?;
                let clone = zval.shallow_clone();
                ht.insert(key, zval)?;
                Ok(clone)
            } else {
                Ok(value.shallow_clone())
            }
        } else {
            Ok(value.shallow_clone())
        }
    }

    /// `ArrayAccess` setter (`$row[$key] = $value`).
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

    /// `ArrayAccess` unsetter (`unset($row[$key])`).
    ///
    /// Unsetting values is not supported and always returns an error.
    pub fn offset_unset(&mut self, offset: &'_ Zval) -> PhpResult {
        let key = ArrayKey::from_zval(offset).ok_or("Illegal offset")?;
        let _ = self.array.borrow_mut().remove(key);
        Ok(())
    }

    /// Implementation of `JsonSerializable::jsonSerialize()`.
    ///
    /// Returns the underlying array with all `LazyRowJson` values decoded for `json_encode()`.
    #[php(name = "jsonSerialize")]
    pub fn json_serialize(&self) -> PhpResult<Zval> {
        use ext_php_rs::convert::IntoZval;
        use ext_php_rs::ffi::zend_array;

        let ht = self.array.borrow();
        let mut result = zend_array::with_capacity(
            u32::try_from(ht.len())
                .map_err(|e| PhpException::from(format!("u32 conversion error: {e}")))?,
        );

        for (key, value) in ht.iter() {
            let decoded_value = if let Some(obj) = value.object() {
                if let Some(lazy_json) = ZendClassObject::<LazyRowJson>::from_zend_obj(obj) {
                    LazyRowJson::decode_json(lazy_json)
                        .map_err(|e| PhpException::from(format!("JSON decode error: {e}")))?
                } else {
                    value.shallow_clone()
                }
            } else {
                value.shallow_clone()
            };
            result
                .insert(key, decoded_value)
                .map_err(|_| PhpException::from("unable to insert into array"))?;
        }

        result
            .into_zval(false)
            .map_err(|e| PhpException::from(format!("conversion error: {e:?}")))
    }

    // =========================================================================
    // Iterator interface implementation
    // =========================================================================

    /// Rewinds the iterator to the first element.
    pub fn rewind(&self) {
        self.iter_pos.set(0);
    }

    /// Returns the current element value (with lazy JSON decoding).
    pub fn current(&self) -> PhpResult<Zval> {
        let pos = self.iter_pos.get();

        // First pass: check if we need to decode and get the key as owned String
        let decode_info: Option<(String, Zval)> = {
            let ht = self.array.borrow();
            if let Some((key, value)) = ht.iter().nth(pos) {
                if let Some(obj) = value.object() {
                    if let Some(lazy_json) = ZendClassObject::<LazyRowJson>::from_zend_obj(obj) {
                        let decoded = LazyRowJson::decode_json(lazy_json)
                            .map_err(|e| PhpException::from(format!("JSON decode error: {e}")))?;
                        // Convert key to owned String
                        let key_string = match key {
                            ArrayKey::Long(n) => n.to_string(),
                            ArrayKey::String(s) => s,
                            ArrayKey::Str(s) => s.to_string(),
                        };
                        Some((key_string, decoded))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                return Ok(Zval::new());
            }
        };

        // Second pass: update the array if needed, or just return the value
        let mut ht = self.array.borrow_mut();
        if let Some((key, decoded)) = decode_info {
            ht.insert(&key as &str, decoded.shallow_clone())
                .map_err(|_| PhpException::from("unable to update array"))?;
            Ok(decoded)
        } else if let Some((_, value)) = ht.iter().nth(pos) {
            Ok(value.shallow_clone())
        } else {
            Ok(Zval::new())
        }
    }

    /// Returns the key of the current element.
    pub fn key(&self) -> PhpResult<Zval> {
        use ext_php_rs::convert::IntoZval;

        let pos = self.iter_pos.get();
        let ht = self.array.borrow();

        if let Some((key, _)) = ht.iter().nth(pos) {
            match key {
                ArrayKey::Long(n) => n
                    .into_zval(false)
                    .map_err(|e| PhpException::from(format!("key conversion error: {e:?}"))),
                ArrayKey::String(s) => s
                    .into_zval(false)
                    .map_err(|e| PhpException::from(format!("key conversion error: {e:?}"))),
                ArrayKey::Str(s) => s
                    .to_string()
                    .into_zval(false)
                    .map_err(|e| PhpException::from(format!("key conversion error: {e:?}"))),
            }
        } else {
            Ok(Zval::new())
        }
    }

    /// Moves the iterator to the next element.
    pub fn next(&self) {
        self.iter_pos.set(self.iter_pos.get() + 1);
    }

    /// Checks if the current position is valid.
    pub fn valid(&self) -> bool {
        let pos = self.iter_pos.get();
        let ht = self.array.borrow();
        pos < ht.len()
    }
}

#[php_class]
#[php(name = "Sqlx\\LazyRowJson")]
#[php(implements("\\JsonSerializable"))]
/// A helper PHP class that holds raw JSON bytes for lazy decoding.
///
/// When accessed, it will be parsed into a PHP value on demand.
/// Implements `JsonSerializable` so that `json_encode()` works correctly.
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

    /// Decode the stored JSON into a PHP `Zval`.
    ///
    /// Uses either `simd-json` or `serde_json` depending on build features.
    ///
    /// # Errors
    ///
    /// Propagates JSON parsing exceptions.
    pub(crate) fn decode_json(self_: &ZendClassObject<LazyRowJson>) -> crate::error::Result<Zval> {
        #[cfg(feature = "simd-json")]
        return json_into_zval(
            simd_json::from_slice::<serde_json::Value>(self_.raw.borrow_mut().as_mut_slice())?,
            self_.assoc,
        );
        #[cfg(not(feature = "simd-json"))]
        json_into_zval(serde_json::from_slice(&self_.raw.borrow())?, self_.assoc)
    }
}

#[php_impl]
impl LazyRowJson {
    /// Implementation of `JsonSerializable::jsonSerialize()`.
    ///
    /// Decodes and returns the stored JSON data for use with `json_encode()`.
    #[php(name = "jsonSerialize")]
    pub fn json_serialize(self_: &ZendClassObject<LazyRowJson>) -> PhpResult<Zval> {
        Self::decode_json(self_).map_err(|e| PhpException::from(format!("JSON decode error: {e}")))
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
