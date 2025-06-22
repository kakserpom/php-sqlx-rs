#![allow(clippy::needless_pass_by_value)]

use crate::conversion::Conversion;
use crate::paramvalue::{ParameterValue, bind_values};
use crate::utils::{fold_into_zend_hashmap, fold_into_zend_hashmap_grouped};
use crate::{RUNTIME, utils::{ColumnArgument, ZvalNull}, php_sqlx_impl_driver_inner};
use anyhow::{anyhow, bail};
use ext_php_rs::convert::IntoZval;
use ext_php_rs::ffi::zend_array;
use ext_php_rs::types::Zval;
use itertools::Itertools;
use sqlx::Error;
use sqlx::Row;
use sqlx::mysql::MySqlPoolOptions as PoolOptions;
use crate::mysql::ast::Ast;
use sqlx::pool::Pool;
use sqlx::{Column, MySql};
use std::collections::HashMap;
use threadsafe_lru::LruCache;
use crate::options::DriverInnerOptions;

pub struct MySqlDriverInner {
    pub pool: Pool<MySql>,
    pub ast_cache: LruCache<String, Ast>,
    pub options: DriverInnerOptions,
}

php_sqlx_impl_driver_inner!(MySqlDriverInner);
