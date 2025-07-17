use anyhow::anyhow;
use ext_php_rs::ZvalConvert;
use ext_php_rs::boxed::ZBox;
use ext_php_rs::ffi::zend_array;
use ext_php_rs::types::{ArrayKey, ZendHashTable, Zval};

/// Trait providing case-insensitive stripping of word prefixes from a string.
///
/// This is useful for checking SQL keywords like `NOT IN (...)`
/// while ignoring case and ensuring prefix boundaries are respected.
pub trait StripPrefixWordIgnoreAsciiCase {
    /// Strips a sequence of words from the beginning of a string,
    /// ignoring ASCII case and ensuring each word is not a prefix of a larger identifier.
    ///
    /// Returns the remainder of the string if matched.
    fn strip_prefix_word_ignore_ascii_case(&self, prefix_words: &[&str]) -> Option<&str>;
}
impl<T: AsRef<str> + ?Sized> StripPrefixWordIgnoreAsciiCase for T {
    fn strip_prefix_word_ignore_ascii_case(&self, prefix_words: &[&str]) -> Option<&str> {
        let mut s = self.as_ref();
        for (i, prefix) in prefix_words.iter().enumerate() {
            if i > 0 {
                s = s.trim_start();
            }

            let prefix_len = prefix.len();
            if s.len() < prefix_len || !s[..prefix_len].eq_ignore_ascii_case(prefix) {
                return None;
            }
            if let Some(c) = &s[prefix_len..].chars().next() {
                if c.is_alphanumeric() {
                    return None;
                }
            }
            s = &s[prefix_len..];
        }
        Some(s)
    }
}

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
    let array_mut = &mut array;
    match key {
        ArrayKey::Long(_) | ArrayKey::Str(_) => {
            if let Some(entry) = array_mut.get_mut(key.clone()) {
                let entry_array = entry.array_mut().unwrap();
                entry_array.push(value).map_err(|err| anyhow!("{err:?}"))?;
            } else {
                let mut entry_array = zend_array::new();
                entry_array.push(value).map_err(|err| anyhow!("{err:?}"))?;
                array_mut
                    .insert(key, entry_array)
                    .map_err(|err| anyhow!("{err:?}"))?;
            }
        }
        ArrayKey::String(key) => {
            let key = key.as_str();
            if let Some(entry) = array_mut.get_mut(key) {
                let entry_array = entry.array_mut().unwrap();
                entry_array.push(value).map_err(|err| anyhow!("{err:?}"))?;
            } else {
                let mut entry_array = zend_array::new();
                entry_array.push(value).map_err(|err| anyhow!("{err:?}"))?;
                array_mut
                    .insert(key, entry_array)
                    .map_err(|err| anyhow!("{err:?}"))?;
            }
        }
    }
    Ok(array)
}

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

/// Represents a column reference, either by numeric index or string name.
///
/// Used to specify how to extract a value from a SQL row result.
#[derive(Debug, ZvalConvert)]
pub enum ColumnArgument<'a> {
    /// Column by numeric index (0-based).
    Index(usize),
    /// Column by name.
    Name(&'a str),
}
