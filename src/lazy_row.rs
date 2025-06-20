use crate::conversion::json_into_zval;

use ext_php_rs::boxed::ZBox;
use ext_php_rs::ffi::zend_array;
use ext_php_rs::prelude::*;
use ext_php_rs::types::{ZendClassObject, Zval};
use ext_php_rs::zend::ce;
use ext_php_rs::{php_class, php_impl};
use std::cell::RefCell;
#[php_class]
#[php(name = "Sqlx\\LazyRow")]
#[php(rename = "none")]
#[php(implements(ce = ce::arrayaccess, stub = "\\ArrayAccess"))]
pub struct LazyRow {
    pub(crate) array: RefCell<ZBox<zend_array>>,
}

impl LazyRow {
    pub fn new(row: ZBox<zend_array>) -> Self {
        Self {
            array: RefCell::new(row),
        }
    }
}
#[php_impl]
impl LazyRow {
    pub fn offset_exists(&self, offset: &'_ Zval) -> PhpResult<bool> {
        Ok(self
            .array
            .borrow()
            .get(offset.str().ok_or("Expected string offset")?)
            .is_some())
    }
    pub fn __get(&self, name: &str) -> PhpResult<Zval> {
        let mut ht = self.array.borrow_mut();
        let value = ht
            .get_mut(name)
            .ok_or_else(|| PhpException::from("column not found"))?;

        if let Some(obj) = value.object_mut() {
            if let Some(lazy_row_json) = ZendClassObject::<LazyRowJson>::from_zend_obj(obj) {
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

    pub fn offset_get(&self, offset: &'_ Zval) -> PhpResult<Zval> {
        let mut ht = self.array.borrow_mut();
        let value = ht
            .get_mut(offset.str().ok_or("Expected string offset")?)
            .ok_or_else(|| PhpException::from("column not found"))?;

        if let Some(obj) = value.object_mut() {
            if let Some(lazy_row_json) = ZendClassObject::<LazyRowJson>::from_zend_obj(obj) {
                let zval = LazyRowJson::take_zval(lazy_row_json)?;
                let clone = value.shallow_clone();
                ht.insert(offset.str().ok_or("Expected string offset")?, zval)?;
                Ok(clone)
            } else {
                Ok(value.shallow_clone())
            }
        } else {
            Ok(value.shallow_clone())
        }
    }
    pub fn offset_set(&mut self, offset: &'_ Zval, value: &'_ Zval) -> PhpResult {
        self.array
            .borrow_mut()
            .insert(
                offset.str().ok_or("Expected string offset")?,
                value.shallow_clone(),
            )
            .map_err(|_| PhpException::from("unable to set"))
    }
    pub fn offset_unset(&mut self, _offset: &'_ Zval) -> PhpResult {
        Err("Setting values is not supported".into())
    }
}

#[php_class]
#[php(name = "Sqlx\\LazyRowJson")]
#[php(rename = "none")]
pub struct LazyRowJson {
    pub(crate) raw: RefCell<Vec<u8>>,
    pub(crate) assoc: bool,
}

impl LazyRowJson {
    pub(crate) fn new(slice: &[u8], assoc: bool) -> Self {
        Self {
            raw: RefCell::new(slice.to_vec()),
            assoc,
        }
    }
}
#[php_impl]
impl LazyRowJson {
    pub fn take_zval(self_: &ZendClassObject<LazyRowJson>) -> anyhow::Result<Zval> {
        let mut buf = self_.raw.borrow().to_owned();

        #[cfg(feature = "simd-json")]
        return json_into_zval(
            simd_json::from_slice::<serde_json::Value>(&mut buf)?,
            self_.assoc,
        );
        #[cfg(not(feature = "simd-json"))]
        return json_into_zval(serde_json::Value::from_slice(&mut buf)?, self.assoc);
    }
}

pub fn build(module: ModuleBuilder) -> ModuleBuilder {
    module.class::<LazyRow>().class::<LazyRowJson>()
}
