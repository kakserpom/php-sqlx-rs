//! Helpers for folding database results into PHP hash maps.
//!
//! This module provides fold functions used when building PHP associative arrays
//! from database query results, supporting both simple key-value maps and grouped
//! results where multiple values share the same key.

use anyhow::anyhow;
use ext_php_rs::boxed::ZBox;
use ext_php_rs::ffi::zend_array;
use ext_php_rs::types::{ArrayKey, ZendHashTable, Zval};

/// Folds a key-value pair into a PHP associative array represented by `ZendHashTable`.
///
/// Used to accumulate results from database queries into a PHP-compatible hashmap.
///
/// # Errors
/// Returns an error if the insertion into the array fails.
pub fn fold_into_zend_hashmap(
    mut array: ZBox<ZendHashTable>,
    item: anyhow::Result<(ArrayKey, Zval)>,
) -> anyhow::Result<ZBox<ZendHashTable>> {
    let (key, value) = item?;
    array.insert(key, value).map_err(|err| anyhow!("{err:?}"))?;
    Ok(array)
}

/// Folds a key-value pair into a grouped PHP associative array represented by `ZendHashTable`.
///
/// If the key already exists, appends the value to an array at that key.
/// Otherwise, creates a new array for the key and inserts the value.
///
/// # Errors
/// Returns an error if any insertion fails or array conversion fails.
pub fn fold_into_zend_hashmap_grouped(
    mut array: ZBox<ZendHashTable>,
    item: anyhow::Result<(ArrayKey, Zval)>,
) -> anyhow::Result<ZBox<ZendHashTable>> {
    let (key, value) = item?;
    match key {
        ArrayKey::Long(_) | ArrayKey::Str(_) => {
            if let Some(entry) = array.get_mut(key.clone()).and_then(Zval::array_mut) {
                entry.push(value).map_err(|err| anyhow!("{err:?}"))?;
            } else {
                let mut entry_array = zend_array::new();
                entry_array.push(value).map_err(|err| anyhow!("{err:?}"))?;
                array
                    .insert(key, entry_array)
                    .map_err(|err| anyhow!("{err:?}"))?;
            }
        }
        ArrayKey::String(key) => {
            let key = key.as_str();
            if let Some(entry) = array.get_mut(key).and_then(Zval::array_mut) {
                entry.push(value).map_err(|err| anyhow!("{err:?}"))?;
            } else {
                let mut entry_array = zend_array::new();
                entry_array.push(value).map_err(|err| anyhow!("{err:?}"))?;
                array
                    .insert(key, entry_array)
                    .map_err(|err| anyhow!("{err:?}"))?;
            }
        }
    }
    Ok(array)
}
