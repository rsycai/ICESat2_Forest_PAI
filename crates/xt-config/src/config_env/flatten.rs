use crate::config_env::error::configuration_error;
use crate::config_env::naming::normalize;
use serde_json::Value;
use std::collections::BTreeMap;
use xt_error::Result;

pub(super) fn flatten(
    value: &Value,
    path: &str,
    output: &mut BTreeMap<String, String>,
) -> Result<()> {
    if let Value::Object(table) = value {
        for (key, value) in table {
            flatten(value, &format!("{path}__{}", normalize(key)?), output)?;
        }
        return Ok(());
    }
    let text = match value {
        Value::String(value) => value.clone(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::Array(_) => serde_json::to_string(value)
            .map_err(|e| configuration_error(format!("Failed to encode array: {path}")).with_source(e))?,
        Value::Null => {
            return Err(configuration_error(format!(
                "Environment variables cannot represent null: {path}"
            )));
        }
        Value::Object(_) => unreachable!(),
    };
    if text.contains('\0') {
        return Err(configuration_error(format!("Configuration value contains a null character: {path}")));
    }
    if output.insert(path.to_owned(), text).is_some() {
        return Err(configuration_error(format!("Environment variable key collision: {path}")));
    }
    Ok(())
}
