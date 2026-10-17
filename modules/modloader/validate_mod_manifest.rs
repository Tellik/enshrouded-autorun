use crate::package_lookup::valid_identifier;
use crate::{ModManifest, SettingControl, SettingValueType};

pub fn validate_manifest(manifest: &ModManifest) -> Result<(), String> {
    if !crate::config::valid_id(&manifest.id) {
        return Err("mod id is invalid".into());
    }
    let configured_link_ids: Vec<String> =
        serde_json::from_str(include_str!("mod_link_order.json"))
            .map_err(|error| format!("invalid configured mod link registry: {error}"))?;
    for (name, url) in &manifest.links.0 {
        if !configured_link_ids.iter().any(|id| id == name) {
            return Err(format!("unsupported mod link '{name}'"));
        }
        if !url.starts_with("https://") {
            return Err(format!("mod link '{name}' must use HTTPS"));
        }
    }
    let mut setting_keys = std::collections::HashSet::new();
    for setting in &manifest.settings {
        if !valid_identifier(&setting.key) || !setting_keys.insert(setting.key.as_str()) {
            return Err(format!(
                "setting key '{}' is invalid or duplicated",
                setting.key
            ));
        }
        let valid_default = match setting.value_type {
            SettingValueType::Boolean => setting.default.is_boolean(),
            SettingValueType::String => setting.default.is_string(),
            SettingValueType::Integer => setting.default.as_i64().is_some(),
            SettingValueType::Number => setting.default.is_number(),
            SettingValueType::Array => setting.default.is_array(),
        };
        if !valid_default {
            return Err(format!(
                "setting '{}' has an invalid default value",
                setting.key
            ));
        }
        if let (Some(minimum), Some(maximum)) = (setting.minimum, setting.maximum)
            && minimum > maximum
        {
            return Err(format!(
                "setting '{}' has minimum greater than maximum",
                setting.key
            ));
        }
        if setting
            .step
            .is_some_and(|step| !step.is_finite() || step <= 0.0)
        {
            return Err(format!("setting '{}' has an invalid step", setting.key));
        }
        if setting.value_type == SettingValueType::Array && setting.options.is_empty() {
            return Err(format!("array setting '{}' requires options", setting.key));
        }
        if setting.control.is_some_and(|control| {
            matches!(
                control,
                SettingControl::Select
                    | SettingControl::Radio
                    | SettingControl::Segmented
                    | SettingControl::Multiselect
            )
        }) && setting.options.is_empty()
        {
            return Err(format!(
                "setting '{}' uses a choice control without options",
                setting.key
            ));
        }
    }
    let mut placed_settings = std::collections::HashSet::new();
    for group in &manifest.groups {
        if group.label.trim().is_empty() {
            return Err("setting group label must not be empty".into());
        }
        for key in &group.settings {
            if !setting_keys.contains(key.as_str()) {
                return Err(format!(
                    "group '{}' references unknown setting '{key}'",
                    group.label
                ));
            }
            if !placed_settings.insert(key.as_str()) {
                return Err(format!("setting '{key}' appears in more than one group"));
            }
        }
        for action in &group.actions {
            if !valid_identifier(&action.id) || action.label.trim().is_empty() {
                return Err(format!(
                    "group '{}' contains an invalid action",
                    group.label
                ));
            }
            if action
                .style
                .as_deref()
                .is_some_and(|style| !matches!(style, "primary" | "secondary" | "danger"))
            {
                return Err(format!("action '{}' has an invalid style", action.id));
            }
        }
    }
    Ok(())
}
