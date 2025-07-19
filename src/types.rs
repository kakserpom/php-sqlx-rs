use crate::param_value::ParameterValue;
use ext_php_rs::prelude::*;

pub fn build(module: ModuleBuilder) -> ModuleBuilder {
    module.class::<JsonWrapper>().function(wrap_function!(json))
}

#[php_class]
#[php(name = "Sqlx\\JsonWrapper")]
pub struct JsonWrapper {
    pub pv: ParameterValue,
}

#[php_function]
#[php(name = "Sqlx\\JSON")]
pub fn json(pv: ParameterValue) -> JsonWrapper {
    JsonWrapper { pv }
}
