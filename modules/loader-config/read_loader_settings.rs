use crate::config::*;
use serde_json::Value;
use std::{fs, path::Path};

pub fn read_loader(root: &Path) -> Result<Value, String> {
    let path = document_path(root, crate::documents::LOADER);
    let defaults: Value = serde_json::from_str(include_str!("loader.default.json"))
        .map_err(|error| error.to_string())?;
    let mut result = defaults;
    let mut loaded = match fs::read(&path) {
        Ok(bytes) => {
            Some(serde_json::from_slice::<Value>(&bytes).map_err(|error| error.to_string())?)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.to_string()),
    };
    if let Some(loaded) = loaded.as_mut() {
        if let Some(modules) = loaded
            .pointer_mut("/modules")
            .and_then(Value::as_object_mut)
        {
            for name in ["worldEditor", "modloaderUi"] {
                let settings = modules
                    .entry(name)
                    .or_insert_with(|| Value::Object(Default::default()));
                if let Some(settings) = settings.as_object_mut() {
                    settings.entry("enabled").or_insert(Value::Bool(true));
                }
            }
            let network = modules
                .entry("network")
                .or_insert_with(|| Value::Object(Default::default()));
            if let Some(network) = network.as_object_mut() {
                network.entry("enabled").or_insert(Value::Bool(true));
            }
            let updates = modules
                .entry("updates")
                .or_insert_with(|| Value::Object(Default::default()));
            if let Some(updates) = updates.as_object_mut() {
                updates.entry("enabled").or_insert(Value::Bool(true));
            }
        }
        let legacy_system = loaded
            .pointer("/modules/updates/system")
            .cloned()
            .unwrap_or_default();
        let legacy_mods = loaded
            .pointer("/modules/updates/mods")
            .cloned()
            .unwrap_or_default();
        if let Some(updates) = loaded
            .pointer_mut("/modules/updates")
            .and_then(Value::as_object_mut)
        {
            let automatic = updates
                .entry("automatic")
                .or_insert_with(|| Value::Object(Default::default()));
            if let Some(automatic) = automatic.as_object_mut() {
                automatic.entry("enabled").or_insert_with(|| {
                    Value::Bool(
                        legacy_system
                            .get("enabled")
                            .and_then(Value::as_bool)
                            .unwrap_or(true)
                            && legacy_mods
                                .get("enabled")
                                .and_then(Value::as_bool)
                                .unwrap_or(true),
                    )
                });
                automatic.entry("checkMinutes").or_insert_with(|| {
                    let minutes = legacy_system
                        .get("checkMinutes")
                        .or_else(|| legacy_mods.get("checkMinutes"))
                        .and_then(Value::as_u64)
                        .unwrap_or(60)
                        .clamp(5, 1440);
                    Value::from(minutes)
                });
                automatic.entry("queueUpdates").or_insert(Value::Bool(true));
            }
            for name in ["system", "mods"] {
                if let Some(settings) = updates.get_mut(name).and_then(Value::as_object_mut) {
                    settings.remove("enabled");
                    settings.remove("checkMinutes");
                }
            }
        }
        if let Some(diagnostics) = loaded
            .pointer_mut("/modules/runtimeDiagnostics")
            .and_then(Value::as_object_mut)
        {
            if !diagnostics.contains_key("sessionActive") {
                if let Some(active) = diagnostics.get("enabled").cloned() {
                    diagnostics.insert("sessionActive".into(), active);
                }
            }
        }
        if let Some(provider) = loaded
            .pointer_mut("/catalog/provider")
            .and_then(Value::as_object_mut)
        {
            for key in [
                "enabled",
                "checkMinutes",
                "includePrereleases",
                "lastCheckAt",
                "latestRelease",
            ] {
                provider.remove(key);
            }
        }
        if let Some(system) = loaded
            .pointer_mut("/modules/updates/system")
            .and_then(Value::as_object_mut)
        {
            system.remove("channel");
        }
    }
    fn merge(target: &mut Value, source: Value) {
        match (target, source) {
            (Value::Object(target), Value::Object(source)) => {
                for (key, value) in source {
                    merge(target.entry(key).or_insert(Value::Null), value);
                }
            }
            (target, source) => *target = source,
        }
    }
    if let Some(loaded) = loaded {
        merge(&mut result, loaded);
    }
    if let Some(settings) = result
        .pointer_mut("/modules/runtimeDiagnostics")
        .and_then(Value::as_object_mut)
    {
        settings.remove("areas");
        if settings.get("jobId").and_then(Value::as_str) == Some("legacy.custom-areas") {
            settings.insert("jobId".into(), "runtime.overview".into());
        }
    }
    validate_document(root, crate::documents::LOADER, &result)?;
    Ok(result)
}
