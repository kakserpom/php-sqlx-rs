use crate::ast::RenderingSettings;
use crate::byclause::ByClauseRendered;
use crate::paginateclause::PaginateClauseRendered;
use crate::selectclause::SelectClauseRendered;
use anyhow::bail;
use ext_php_rs::convert::{FromZval, IntoZval};
use ext_php_rs::flags::DataType;
use ext_php_rs::types::{ZendClassObject, ZendHashTable, Zval};
use itertools::Itertools;
use sqlx_oldapi::database::HasArguments;
use sqlx_oldapi::query::Query;
use sqlx_oldapi::{Database, Encode, Type};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;

/// A type alias representing the name of a placeholder in SQL templates.
pub type Placeholder = String;

/// A type alias for a parameter map used during query rendering and execution.
/// Keys are placeholders (e.g., `id`), and values are user-supplied input.
pub type ParamsMap = BTreeMap<Placeholder, ParameterValue>;

/// Represents a parameter value for use in SQL queries, supporting both primitive and complex structures.
///
/// Includes built-in types (string, int, float, bool), composite values (arrays, objects),
/// and pre-rendered clauses like `ORDER BY`, `SELECT`, and pagination fragments.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    Null,
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<ParameterValue>),
    Object(HashMap<String, ParameterValue>),
    ByClauseRendered(ByClauseRendered),
    SelectClauseRendered(SelectClauseRendered),
    PaginateClauseRendered(PaginateClauseRendered),
}

impl ParameterValue {
    /// Checks whether the value is considered "empty".
    ///
    /// - For `ByClauseRendered`, `SelectClauseRendered`, and `Array`, returns true if empty.
    /// - For `Null`, always returns true.
    /// - Other variants return false.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::ByClauseRendered(x) => x.is_empty(),
            Self::SelectClauseRendered(x) => x.is_empty(),
            Self::Array(array) => array.is_empty(),
            Self::Str(_)
            | Self::Int(_)
            | Self::Float(_)
            | Self::Bool(_)
            | Self::Object(_)
            | Self::PaginateClauseRendered(_) => false,
            Self::Null => true,
        }
    }
}

pub trait ParamVecWriteSqlTo {
    fn write_sql_to(
        &self,
        sql: &mut String,
        out_vals: &mut Vec<ParameterValue>,
        rendering_settings: &RenderingSettings,
    ) -> anyhow::Result<()>;
}

impl ParamVecWriteSqlTo for Vec<ParameterValue> {
    #[inline]
    fn write_sql_to(
        &self,
        sql: &mut String,
        out_vals: &mut Vec<ParameterValue>,
        rendering_settings: &RenderingSettings,
    ) -> anyhow::Result<()> {
        out_vals.reserve_exact(self.len());
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            out_vals.push(item.clone());
            if rendering_settings.placeholder_dollar_sign {
                write!(sql, "${}", out_vals.len())?;
            } else if rendering_settings.placeholder_at_sign {
                write!(sql, "@p{}", out_vals.len())?;
            } else {
                sql.push('?');
            }
        }
        Ok(())
    }
}

impl From<&str> for ParameterValue {
    /// Converts a `&str` into a `ParameterValue::Str`.
    fn from(s: &str) -> Self {
        ParameterValue::Str(s.to_string())
    }
}
impl From<String> for ParameterValue {
    /// Converts a `String` into a `ParameterValue::Str`.
    fn from(s: String) -> Self {
        ParameterValue::Str(s)
    }
}

impl From<i64> for ParameterValue {
    /// Converts an `i64` into a `ParameterValue::Int`.
    fn from(s: i64) -> Self {
        ParameterValue::Int(s)
    }
}

impl From<bool> for ParameterValue {
    /// Converts a `bool` into a `ParameterValue::Bool`.
    fn from(s: bool) -> Self {
        ParameterValue::Bool(s)
    }
}

impl IntoZval for ParameterValue {
    const TYPE: DataType = DataType::Mixed;
    const NULLABLE: bool = true;

    /// Converts a `ParameterValue` into a PHP `Zval`.
    ///
    /// - `Str`, `Int`, `Float`, `Bool` map to PHP scalars.
    /// - `Array` and `Object` become PHP arrays.
    /// - `Null` and clause values render as `null`.
    ///
    /// # Errors
    /// Returns an error if value insertion fails.
    fn set_zval(self, zv: &mut Zval, persistent: bool) -> ext_php_rs::error::Result<()> {
        match self {
            ParameterValue::Str(str) => zv.set_string(str.as_str(), persistent)?,
            ParameterValue::Int(i64) => zv.set_long(i64),
            ParameterValue::Float(f64) => zv.set_double(f64),
            ParameterValue::Bool(bool) => zv.set_bool(bool),
            ParameterValue::Array(array) => {
                let mut ht = ZendHashTable::new();
                for val in array {
                    ht.push(val)?;
                }
                zv.set_hashtable(ht);
            }
            ParameterValue::Object(hash_map) => {
                let mut ht = ZendHashTable::new();
                for (k, v) in hash_map {
                    ht.insert(k, v)?;
                }
                zv.set_hashtable(ht);
            }
            ParameterValue::Null
            | ParameterValue::ByClauseRendered(_)
            | ParameterValue::SelectClauseRendered(_)
            | ParameterValue::PaginateClauseRendered(_) => zv.set_null(),
        }
        Ok(())
    }
}

impl FromZval<'_> for ParameterValue {
    const TYPE: DataType = DataType::Mixed;

    /// Attempts to convert a PHP `Zval` into a `ParameterValue`.
    ///
    /// - Arrays are parsed into either `Array` or `Object` depending on their keys.
    /// - `stdClass` maps to `Object`.
    /// - Instances of known clause types are wrapped appropriately.
    fn from_zval(zval: &Zval) -> Option<Self> {
        match zval.get_type() {
            DataType::Undef | DataType::Null | DataType::Void => Some(Self::Null),
            DataType::False => Some(Self::Bool(false)),
            DataType::True => Some(Self::Bool(true)),
            DataType::Long => Some(Self::Int(zval.long()?)),
            DataType::Double => Some(Self::Float(zval.double()?)),
            DataType::String => Some(Self::Str(zval.string()?)),
            DataType::Array => {
                let array = zval.array()?;
                if array.has_sequential_keys() {
                    Some(Self::Array(
                        array
                            .iter()
                            .map(|(_, value)| Self::from_zval(value).ok_or(()))
                            .try_collect()
                            .ok()?,
                    ))
                } else {
                    Some(Self::Object(
                        array
                            .iter()
                            .map(|(key, value)| {
                                Self::from_zval(value)
                                    .map(|value| (key.to_string(), value))
                                    .ok_or(())
                            })
                            .try_collect()
                            .ok()?,
                    ))
                }
            }
            DataType::Object(_) => {
                let obj = zval.object()?;
                match obj.get_class_name().ok()?.as_str() {
                    "Sqlx\\ByClauseRendered" => Some(Self::ByClauseRendered(
                        ZendClassObject::<ByClauseRendered>::from_zend_obj(obj)
                            .and_then(|x| x.obj.as_ref())?
                            .to_owned(),
                    )),
                    "Sqlx\\SelectClauseRendered" => Some(Self::SelectClauseRendered(
                        ZendClassObject::<SelectClauseRendered>::from_zend_obj(obj)
                            .and_then(|x| x.obj.as_ref())?
                            .to_owned(),
                    )),
                    "Sqlx\\PaginateClauseRendered" => Some(Self::PaginateClauseRendered(
                        ZendClassObject::<PaginateClauseRendered>::from_zend_obj(obj)
                            .and_then(|x| x.obj.as_ref())?
                            .to_owned(),
                    )),
                    "stdClass" => Some(Self::Object(
                        obj.get_properties()
                            .ok()?
                            .iter()
                            .map(|(key, value)| {
                                Self::from_zval(value)
                                    .map(|value| (key.to_string(), value))
                                    .ok_or(())
                            })
                            .try_collect()
                            .ok()?,
                    )),
                    _ => None,
                }
            }
            DataType::Iterable
            | DataType::Mixed
            | DataType::Resource
            | DataType::Reference
            | DataType::Callable
            | DataType::ConstantExpression
            | DataType::Ptr
            | DataType::Indirect => None,
            DataType::Bool => Some(Self::Bool(zval.bool()?)),
        }
    }
}

/// Binds a list of `ParameterValue` items to an `SQLx` query.
///
/// This function recursively traverses and binds all primitive values from the input slice,
/// supporting nested arrays and objects. Each primitive is passed to `query.bind()`, in left-to-right order.
///
/// # Supported types
/// - `Str`, `Int`, `Float`, `Bool` — bound directly
/// - `Array`, `Object` — recursively expanded and flattened into positional bindings
///
/// # Unsupported types
/// - `ByClauseRendered`, `SelectClauseRendered`, `PaginateClauseRendered`, and `Null` are not bindable and will
///   result in an error
///
/// # Errors
/// Returns an `anyhow::Error` if an unsupported value is encountered or if recursive binding fails.
///
/// # Example
/// ```rust
///  use sqlx_oldapi::Postgres;
///  use php_sqlx::paramvalue::{bind_values, ParameterValue};
///  let query = sqlx_oldapi::query::<Postgres>("SELECT * FROM users WHERE id = $1 AND active = $2");
///  let values = &[ParameterValue::Int(1), ParameterValue::Bool(true)];
///  let query = bind_values(query, values).expect("Cannot bind values");
/// ```
pub fn bind_values<'a, D: Database>(
    query: Query<'a, D, <D as HasArguments<'a>>::Arguments>,
    values: &'a [ParameterValue],
) -> anyhow::Result<Query<'a, D, <D as HasArguments<'a>>::Arguments>>
where
    f64: Type<D>,
    f64: Encode<'a, D>,
    i64: Type<D>,
    i64: Encode<'a, D>,
    bool: Type<D>,
    bool: Encode<'a, D>,
    String: Type<D>,
    String: Encode<'a, D>,
{
    fn walker<'a, D: Database>(
        q: Query<'a, D, <D as HasArguments<'a>>::Arguments>,
        value: &'a ParameterValue,
    ) -> anyhow::Result<Query<'a, D, <D as HasArguments<'a>>::Arguments>>
    where
        f64: Type<D>,
        f64: Encode<'a, D>,
        i64: Type<D>,
        i64: Encode<'a, D>,
        bool: Type<D>,
        bool: Encode<'a, D>,
        String: Type<D>,
        String: Encode<'a, D>,
    {
        Ok(match value {
            ParameterValue::Str(s) => q.bind(s),
            ParameterValue::Int(s) => q.bind(s),
            ParameterValue::Bool(s) => q.bind(s),
            ParameterValue::Float(s) => q.bind(s),
            ParameterValue::Array(s) => s.iter().try_fold(q, walker)?,
            // @TODO: values()?
            ParameterValue::Object(s) => s.values().try_fold(q, walker)?,
            ParameterValue::ByClauseRendered(_)
            | ParameterValue::SelectClauseRendered(_)
            | ParameterValue::PaginateClauseRendered(_)
            | ParameterValue::Null => bail!("Internal error: cannot bind parameter of this type"),
        })
    }

    values.iter().try_fold(query, walker)
}
