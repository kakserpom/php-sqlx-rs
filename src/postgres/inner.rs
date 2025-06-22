#![allow(clippy::needless_pass_by_value)]
use crate::conversion::Conversion;
use crate::options::DriverInnerOptions;
use crate::paramvalue::{ParameterValue, bind_values};
use crate::postgres::ast::Ast;
use crate::utils::ZvalNull;
use crate::utils::{ColumnArgument, fold_into_zend_hashmap, fold_into_zend_hashmap_grouped};
use crate::{RUNTIME, php_sqlx_impl_driver_inner};
use anyhow::{anyhow, bail};
use ext_php_rs::convert::IntoZval;
use ext_php_rs::ffi::zend_array;
use ext_php_rs::types::Zval;
use itertools::Itertools;
use sqlx::Error;
use sqlx::Row;
use sqlx::pool::Pool;
use sqlx::postgres::PgPoolOptions as PoolOptions;
use sqlx::{Column, Postgres};
use std::collections::HashMap;
use threadsafe_lru::LruCache;

pub struct PgDriverInner {
    pub pool: Pool<Postgres>,
    pub ast_cache: LruCache<String, Ast>,
    pub options: DriverInnerOptions,
}

php_sqlx_impl_driver_inner!(PgDriverInner);
