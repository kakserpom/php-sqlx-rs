use crate::param_value::ParameterValue;

impl ParameterValue {
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}
