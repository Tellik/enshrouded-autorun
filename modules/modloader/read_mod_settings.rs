use crate::{SettingDefinition, SettingOption, SettingValueType};
use serde_json::{Value, json};
pub(crate) fn parse_setting_definition(
    key: &str,
    value: &Value,
    metadata: &Value,
) -> Result<SettingDefinition, String> {
    let value_type = match value {
        Value::Bool(_) => SettingValueType::Boolean,
        Value::String(_) => SettingValueType::String,
        Value::Number(number) if number.as_i64().is_some() => SettingValueType::Integer,
        Value::Number(_) => SettingValueType::Number,
        Value::Array(_) => SettingValueType::Array,
        _ => {
            return Err(format!(
                "setting '{key}' value must be boolean, string, number, or array"
            ));
        }
    };
    let options = metadata
        .get("options")
        .and_then(Value::as_object)
        .map(|entries| {
            entries
                .iter()
                .map(|(option, label)| {
                    let option_value = match value_type {
                        SettingValueType::Boolean => Value::Bool(option == "true"),
                        SettingValueType::Integer => Value::Number(
                            option
                                .parse::<i64>()
                                .map_err(|_| format!("invalid option for '{key}'"))?
                                .into(),
                        ),
                        SettingValueType::Number => json!(
                            option
                                .parse::<f64>()
                                .map_err(|_| format!("invalid option for '{key}'"))?
                        ),
                        SettingValueType::Array => {
                            match value.as_array().and_then(|items| items.first()) {
                                Some(Value::Bool(_)) => Value::Bool(option == "true"),
                                Some(Value::Number(number)) if number.as_i64().is_some() => {
                                    Value::Number(
                                        option
                                            .parse::<i64>()
                                            .map_err(|_| format!("invalid option for '{key}'"))?
                                            .into(),
                                    )
                                }
                                Some(Value::Number(_)) => json!(
                                    option
                                        .parse::<f64>()
                                        .map_err(|_| format!("invalid option for '{key}'"))?
                                ),
                                _ => Value::String(option.clone()),
                            }
                        }
                        _ => Value::String(option.clone()),
                    };
                    Ok(SettingOption {
                        value: option_value,
                        label: label
                            .as_str()
                            .ok_or_else(|| format!("option label for '{key}' must be a string"))?
                            .to_owned(),
                    })
                })
                .collect::<Result<Vec<_>, String>>()
        })
        .transpose()?
        .unwrap_or_default();
    let mut display_key = String::with_capacity(key.len());
    for (index, character) in key.chars().enumerate() {
        if index > 0 && character.is_uppercase() {
            display_key.push(' ');
        }
        display_key.push(character);
    }
    let mut control = metadata
        .get("control")
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .map_err(|error| format!("setting '{key}' control: {error}"))?;
    if value_type == SettingValueType::Array && options.is_empty() {
        return Err(format!("array setting '{key}' must declare options"));
    }
    if control.is_none() && !options.is_empty() {
        control = Some(if value_type == SettingValueType::Array {
            crate::SettingControl::Multiselect
        } else {
            crate::SettingControl::Select
        });
    }
    Ok(SettingDefinition {
        key: key.into(),
        value_type,
        control,
        label: metadata
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or(&display_key)
            .into(),
        description: metadata
            .get("description")
            .and_then(Value::as_str)
            .map(str::to_owned),
        default: value.clone(),
        group: None,
        minimum: metadata.get("min").and_then(Value::as_f64),
        maximum: metadata.get("max").and_then(Value::as_f64),
        step: metadata.get("step").and_then(Value::as_f64),
        minimum_length: metadata
            .get("minLength")
            .and_then(Value::as_u64)
            .map(|value| value as usize),
        maximum_length: metadata
            .get("maxLength")
            .and_then(Value::as_u64)
            .map(|value| value as usize),
        options,
        restart_required: true,
        apply_at: "restart".into(),
    })
}
