use crate::SettingDefinition;
use serde_json::{Value, json};
pub fn validate_settings(definitions: &[SettingDefinition], values: &Value) -> Result<(), String> {
    let object = values.as_object().ok_or("settings must be an object")?;
    for (key, value) in object {
        let definition = definitions
            .iter()
            .find(|d| d.key == *key)
            .ok_or_else(|| format!("unknown setting: {key}"))?;
        if matches!(definition.control, Some(crate::SettingControl::Keybind)) {
            shroudforge_hotkeys::parse_value(value).map_err(|error| format!("{key}: {error}"))?;
        }
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

/// Follow the same enabledWhen chain as the settings editor. Missing
/// controllers and cycles disable a setting instead of admitting its action.
pub fn setting_is_enabled(key: &str, definitions: &[SettingDefinition], values: &Value) -> bool {
    let mut current = key;
    let mut seen = std::collections::BTreeSet::new();
    loop {
        let Some(definition) = definitions.iter().find(|setting| setting.key == current) else {
            return false;
        };
        let Some(condition) = &definition.enabled_when else {
            return true;
        };
        if !seen.insert(current) {
            return false;
        }
        let Some(value) = values.get(&condition.setting) else {
            return false;
        };
        // JavaScript has one numeric type. Preserve the editor's equality for
        // JSON integer and floating representations of the same number.
        let matches = if value.is_number() && condition.equals.is_number() {
            value.as_f64() == condition.equals.as_f64()
        } else {
            value == &condition.equals
        };
        if !matches {
            return false;
        }
        current = &condition.setting;
    }
}
