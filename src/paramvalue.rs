use crate::byclause::ByClauseRendered;
use crate::paginateclause::PaginateClauseRendered;
use crate::selectclause::SelectClauseRendered;
use ext_php_rs::convert::{FromZval, IntoZval};
use ext_php_rs::flags::DataType;
use ext_php_rs::types::{ZendClassObject, Zval};
use itertools::Itertools;
use std::collections::HashMap;

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

impl IntoZval for ParameterValue {
    const TYPE: DataType = DataType::Void;
    const NULLABLE: bool = false;

    fn set_zval(self, zv: &mut Zval, persistent: bool) -> ext_php_rs::error::Result<()> {
        zv.set_null();
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
            DataType::Object(opt_class_name) => {
                let obj = zval.object()?;
                match opt_class_name {
                    Some("Sqlx\\ByClauseRendered") => Some(Self::ByClauseRendered(
                        ZendClassObject::<ByClauseRendered>::from_zend_obj(obj)
                            .and_then(|x| x.obj.as_ref())?
                            .to_owned(),
                    )),
                    Some("Sqlx\\SelectClauseRendered") => Some(Self::SelectClauseRendered(
                        ZendClassObject::<SelectClauseRendered>::from_zend_obj(obj)
                            .and_then(|x| x.obj.as_ref())?
                            .to_owned(),
                    )),
                    Some("Sqlx\\PaginateClauseRendered") => Some(Self::PaginateClauseRendered(
                        ZendClassObject::<PaginateClauseRendered>::from_zend_obj(obj)
                            .and_then(|x| x.obj.as_ref())?
                            .to_owned(),
                    )),
                    _ => Some(Self::Object(
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
