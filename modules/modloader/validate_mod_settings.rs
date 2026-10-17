use crate::SettingDefinition;
use serde_json::{Value, json};
pub fn validate_settings(definitions: &[SettingDefinition], values: &Value) -> Result<(), String> {
    let object = values.as_object().ok_or("settings must be an object")?;
    for (key, value) in object {
        let definition = definitions
            .iter()
            .find(|d| d.key == *key)
            .ok_or_else(|| format!("unknown setting: {key}"))?;
        let mut schema = json!({"type": definition.value_type});
        if let Some(v) = definition.minimum {
            schema["minimum"] = json!(v);
        }
        if let Some(v) = definition.maximum {
            schema["maximum"] = json!(v);
        }
        if let Some(v) = definition.minimum_length {
            schema["minLength"] = json!(v);
        }
        if let Some(v) = definition.maximum_length {
            schema["maxLength"] = json!(v);
        }
        if !definition.options.is_empty() {
            let options: Vec<_> = definition.options.iter().map(|v| v.value.clone()).collect();
            if value.is_array() {
                schema["items"] = json!({"enum": options});
            } else {
                schema["enum"] = json!(options);
            }
        }
        let validator = jsonschema::validator_for(&schema).map_err(|e| e.to_string())?;
        validator
            .validate(value)
            .map_err(|e| format!("{key}: {e}"))?;
    }
    Ok(())
}
