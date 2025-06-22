use crate::byclause::ByClauseRendered;
use crate::paginateclause::PaginateClauseRendered;
use crate::selectclause::SelectClauseRendered;
use ext_php_rs::convert::{FromZval, IntoZval};
use ext_php_rs::flags::DataType;
use ext_php_rs::types::{ZendClassObject, ZendHashTable, Zval};
use itertools::Itertools;
use sqlx_oldapi::database::HasArguments;
use sqlx_oldapi::query::Query;
use sqlx_oldapi::{Database, Encode, Type};
use std::collections::{BTreeMap, HashMap};

pub type Placeholder = String;
pub type ParamsMap = BTreeMap<Placeholder, ParameterValue>;

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
    #[must_use]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::ByClauseRendered(x) => x.is_empty(),
            Self::SelectClauseRendered(x) => x.is_empty(),
            Self::Array(array) => array.is_empty(),
            Self::Str(_) | Self::Int(_) | Self::Float(_) | Self::Bool(_) | Self::Object(_) => false,
            Self::PaginateClauseRendered(_) => false,
            Self::Null => true,
        }
    }
}

impl From<&str> for ParameterValue {
    fn from(s: &str) -> Self {
        ParameterValue::Str(s.to_string())
    }
}
impl From<String> for ParameterValue {
    fn from(s: String) -> Self {
        ParameterValue::Str(s)
    }
}

impl From<i64> for ParameterValue {
    fn from(s: i64) -> Self {
        ParameterValue::Int(s)
    }
}

impl From<bool> for ParameterValue {
    fn from(s: bool) -> Self {
        ParameterValue::Bool(s)
    }
}

impl IntoZval for ParameterValue {
    const TYPE: DataType = DataType::Mixed;
    const NULLABLE: bool = true;

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

    fn from_zval(zval: &Zval) -> Option<Self> {
        match zval.get_type() {
            DataType::Mixed => None,
            DataType::Undef => Some(Self::Null),
            DataType::Null => Some(Self::Null),
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
            DataType::Iterable => None,
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
            DataType::Resource => None,
            DataType::Reference => None,
            DataType::Callable => None,
            DataType::ConstantExpression => None,
            DataType::Void => Some(Self::Null),
            DataType::Bool => Some(Self::Bool(zval.bool()?)),
            DataType::Ptr => None,
            DataType::Indirect => None,
        }
    }
}

/// Binds a list of `Value` arguments to an `SQLx` query.
pub fn bind_values<'a, D: Database>(
    query: Query<'a, D, <D as HasArguments<'a>>::Arguments>,
    values: &'a [ParameterValue],
) -> Query<'a, D, <D as HasArguments<'a>>::Arguments>
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
    ) -> Query<'a, D, <D as HasArguments<'a>>::Arguments>
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
        match value {
            ParameterValue::Str(s) => q.bind(s),
            ParameterValue::Int(s) => q.bind(s),
            ParameterValue::Bool(s) => q.bind(s),
            ParameterValue::Float(s) => q.bind(s),
            ParameterValue::Array(s) => s.iter().fold(q, walker),
            // @TODO: values()?
            ParameterValue::Object(s) => s.values().fold(q, walker),
            ParameterValue::ByClauseRendered(_)
            | ParameterValue::SelectClauseRendered(_)
            | ParameterValue::PaginateClauseRendered(_)
            | ParameterValue::Null => unimplemented!(),
        }
    }

    values.iter().fold(query, walker)
}
