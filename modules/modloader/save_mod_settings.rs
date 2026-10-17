use crate::package_lookup::find_mod_package;
use std::path::Path;

pub fn save_mod_settings(
    root: &Path,
    id: &str,
    values: &serde_json::Value,
    revision: &str,
) -> Result<(), String> {
    let package = find_mod_package(root, id).ok_or("mod package not found")?;
    let manifest = crate::config::read_manifest_path(root, &package)?;
    crate::config::validate_settings(&manifest.settings, values)?;
    crate::update_mod_configuration::update_mod_configuration(root, id, revision, |extension| {
        let next = values.as_object().ok_or("settings must be an object")?;
        let settings = extension
            .get_mut("settings")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or("extended.mod.json settings must be an object")?;
        for (key, value) in next {
            let current = settings
                .get_mut(key)
                .ok_or_else(|| format!("unknown setting '{key}'"))?;
            if current.is_object() {
                current["value"] = value.clone();
            } else {
                *current = value.clone();
            }
        }
        Ok(())
    })
}
