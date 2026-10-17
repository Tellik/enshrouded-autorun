use crate::config::{validate_document, validate_settings};
use crate::read_mod_settings::parse_setting_definition;
use crate::{ModManifest, SettingGroup, validate_manifest};
use serde_json::{Value, json};
use std::path::Path;
pub fn parse_manifest(root: &Path, value: Value) -> Result<ModManifest, String> {
    parse_manifest_with_extension(root, value, None)
}

pub fn parse_manifest_with_extension(
    root: &Path,
    mut value: Value,
    external: Option<Value>,
) -> Result<ModManifest, String> {
    let had_extension = external.is_some();
    let mut extension = external;
    if let Some(extension) = extension.as_mut() {
        validate_document(root, crate::documents::EXTENDED_MOD, extension)?;
        if let Some(object) = extension.as_object_mut() {
            object.remove("$schema");
        }
    }
    validate_document(root, crate::documents::MOD, &value)?;
    let extension = extension.unwrap_or_else(|| json!({}));
    let enabled = extension
        .get("enabled")
        .and_then(Value::as_bool)
        // Mods remain disabled until activation is explicitly persisted. This
        // also keeps EML packages without an extension safe by default.
        .unwrap_or(false);
    let target = match extension.get("targets").and_then(Value::as_array) {
        Some(targets) => {
            let client = targets.iter().any(|value| value == "client");
            let server = targets.iter().any(|value| value == "server");
            match (client, server) {
                (true, true) => crate::ModTarget::ClientServer,
                (false, true) => crate::ModTarget::Server,
                _ => crate::ModTarget::Client,
            }
        }
        None if extension.get("launcher").and_then(Value::as_str) == Some("EML") => {
            crate::ModTarget::ClientServer
        }
        None if had_extension => crate::ModTarget::Client,
        None => crate::ModTarget::ClientServer,
    };
    let settings = extension
        .get("settings")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let settings_object = settings
        .as_object()
        .ok_or("extended.mod.json settings must be an object")?;
    let mut effective = serde_json::Map::new();
    let mut definitions = Vec::new();
    let empty_metadata = Value::Null;
    for (key, entry) in settings_object {
        let (value, metadata) = if entry.is_object() {
            (
                entry
                    .get("value")
                    .cloned()
                    .ok_or_else(|| format!("setting '{key}' requires value"))?,
                entry,
            )
        } else {
            (entry.clone(), &empty_metadata)
        };
        let definition = parse_setting_definition(key, &value, metadata)?;
        effective.insert(key.clone(), value);
        definitions.push(definition);
    }
    validate_settings(&definitions, &Value::Object(effective.clone()))?;
    let groups = extension
        .get("groups")
        .map(|value| {
            serde_json::from_value::<Vec<SettingGroup>>(value.clone())
                .map_err(|error| format!("extended.mod.json groups: {error}"))
        })
        .transpose()?
        .unwrap_or_default();
    let object = value.as_object_mut().ok_or("mod.json must be an object")?;
    object.remove("$schema");
    object.insert("enabled".into(), json!(enabled));
    object.insert(
        "target".into(),
        serde_json::to_value(target).map_err(|error| error.to_string())?,
    );
    object.insert("settingValues".into(), Value::Object(effective));
    object.insert("settings".into(), json!(definitions));
    object.insert("groups".into(), json!(groups));
    object.insert(
        "changelog".into(),
        extension
            .get("changelog")
            .cloned()
            .unwrap_or_else(|| json!([])),
    );
    object.insert(
        "links".into(),
        extension.get("links").cloned().unwrap_or_else(|| json!({})),
    );
    let manifest: ModManifest = serde_json::from_value(value).map_err(|error| error.to_string())?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}
