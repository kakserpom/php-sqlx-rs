use crate::by_clause::ByClauseRendered;
use crate::paginate_clause::PaginateClauseRendered;
use crate::param_value::ParameterValue;
use crate::select_clause::SelectClauseRendered;
use ext_php_rs::convert::{FromZval, IntoZval};
use ext_php_rs::flags::DataType;
use ext_php_rs::types::{ZendClassObject, ZendHashTable, Zval};
use itertools::Itertools;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};

impl From<&str> for ParameterValue {
    /// Converts a `&str` into a `ParameterValue::Str`.
    fn from(s: &str) -> Self {
        ParameterValue::String(s.to_string())
    }
}
impl From<String> for ParameterValue {
    /// Converts a `String` into a `ParameterValue::Str`.
    fn from(s: String) -> Self {
        ParameterValue::String(s)
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
            Self::String(str) => zv.set_string(str.as_str(), persistent)?,
            Self::Int(i64) => zv.set_long(i64),
            Self::Float(f64) => zv.set_double(f64),
            Self::Bool(bool) => zv.set_bool(bool),
            Self::Array(array) => {
                let mut ht = ZendHashTable::new();
                for val in array {
                    ht.push(val)?;
                }
                zv.set_hashtable(ht);
            }
            Self::Object(hash_map) => {
                let mut ht = ZendHashTable::new();
                for (k, v) in hash_map {
                    ht.insert(k, v)?;
                }
                zv.set_hashtable(ht);
            }
            Self::Null
            | Self::ByClauseRendered(_)
            | Self::SelectClauseRendered(_)
            | Self::PaginateClauseRendered(_)
            | Self::Builder(_) => zv.set_null(),
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
            DataType::String => Some(Self::String(zval.string()?)),
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

impl Serialize for ParameterValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Null => serializer.serialize_none(),
            Self::String(s) => serializer.serialize_str(s),
            Self::Int(i) => serializer.serialize_i64(*i),
            Self::Float(f) => serializer.serialize_f64(*f),
            Self::Bool(b) => serializer.serialize_bool(*b),

            Self::Array(arr) => {
                let mut seq = serializer.serialize_seq(Some(arr.len()))?;
                for elem in arr {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }

            Self::Object(map) => {
                let mut m = serializer.serialize_map(Some(map.len()))?;
                for (k, v) in map {
                    m.serialize_entry(k, v)?;
                }
                m.end()
            }

            Self::ByClauseRendered(val) => serializer.serialize_str(&format!("{val:?}")),

            Self::SelectClauseRendered(val) => serializer.serialize_str(&format!("{val:?}")),

            Self::PaginateClauseRendered(val) => serializer.serialize_str(&format!("{val:?}")),
            Self::Builder(_) => Err(serde::ser::Error::custom("Builder cannot be serialized")),
        }
    }
}
