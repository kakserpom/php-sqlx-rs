use anyhow::{anyhow, bail};
use ext_php_rs::ZvalConvert;
use ext_php_rs::boxed::ZBox;
use ext_php_rs::ffi::zend_array;
use ext_php_rs::types::{ZendHashTable, Zval};

pub trait StripPrefixIgnoreAsciiCase {
    fn strip_prefix_ignore_ascii_case(&self, prefix: &str) -> Option<&str>;
}
impl<T: AsRef<str> + ?Sized> StripPrefixIgnoreAsciiCase for T {
    fn strip_prefix_ignore_ascii_case(&self, prefix: &str) -> Option<&str> {
        let s = self.as_ref();
        let prefix_len = prefix.len();
        if s.len() >= prefix_len && s[..prefix_len].eq_ignore_ascii_case(prefix) {
            Some(&s[prefix_len..])
        } else {
            None
        }
    }
}

pub fn fold_into_zend_hashmap(
    mut array: ZBox<ZendHashTable>,
    item: anyhow::Result<(Zval, Zval)>,
) -> anyhow::Result<ZBox<ZendHashTable>> {
    let (key, value) = item?;
    if let Some(index) = key.long() {
        if let Ok(non_negative_index) = u64::try_from(index) {
            array
                .insert_at_index(non_negative_index, value)
                .map_err(|err| anyhow!("{err:?}"))?;
        } else {
            let key = index.to_string();
            array
                .insert(&key, value)
                .map_err(|err| anyhow!("{err:?}"))?;
        }
    } else if let Some(key) = key.string() {
        array
            .insert(&key, value)
            .map_err(|err| anyhow!("{err:?}"))?;
    } else {
        bail!("First column must be scalar")
    }
    Ok(array)
}

pub fn fold_into_zend_hashmap_grouped(
    mut array: ZBox<ZendHashTable>,
    item: anyhow::Result<(Zval, Zval)>,
) -> anyhow::Result<ZBox<ZendHashTable>> {
    let (key, value) = item?;
    let array_mut = &mut array;
    if let Some(index) = key.long() {
        if let Ok(non_negative_index) = u64::try_from(index) {
            if let Some(entry) = array_mut.get_index_mut(non_negative_index) {
                let entry_array = entry.array_mut().unwrap();
                entry_array.push(value).map_err(|err| anyhow!("{err:?}"))?;
            } else {
                let mut entry_array = zend_array::new();
                entry_array.push(value).map_err(|err| anyhow!("{err:?}"))?;
                array_mut
                    .insert_at_index(non_negative_index, entry_array)
                    .map_err(|err| anyhow!("{err:?}"))?;
            }
        } else {
            let key = index.to_string();
            if let Some(entry) = array_mut.get_mut(&key) {
                let entry_array = entry.array_mut().unwrap();
                entry_array.push(value).map_err(|err| anyhow!("{err:?}"))?;
            } else {
                let mut entry_array = zend_array::new();
                entry_array.push(value).map_err(|err| anyhow!("{err:?}"))?;
                array_mut
                    .insert(&key, entry_array)
                    .map_err(|err| anyhow!("{err:?}"))?;
            }
        }
    } else if let Some(key) = key.string() {
        if let Some(entry) = array_mut.get_mut(&key) {
            let entry_array = entry.array_mut().unwrap();
            entry_array.push(value).map_err(|err| anyhow!("{err:?}"))?;
        } else {
            let mut entry_array = zend_array::new();
            entry_array.push(value).map_err(|err| anyhow!("{err:?}"))?;
            array_mut
                .insert(&key, entry_array)
                .map_err(|err| anyhow!("{err:?}"))?;
        }
    } else {
        bail!("First column must be scalar")
    }
    Ok(array)
}

#[must_use]
pub fn is_valid_ident(name: &str) -> bool {
    !name.is_empty()
        && name.starts_with(|c: char| c.is_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_alphanumeric() || c == '_')
}
pub trait ZvalNull {
    fn null() -> Zval;
}
impl ZvalNull for Zval {
    fn null() -> Zval {
        let mut zval = Zval::new();
        zval.set_null();
        zval
    }
}

#[derive(Debug, ZvalConvert)]
pub enum ColumnArgument<'a> {
    Index(usize),
    Name(&'a str),
}
