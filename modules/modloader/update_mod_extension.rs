use crate::config_revision;
use crate::package_files::{read_package_file, write_package_json};
use crate::package_lookup::find_mod_package;
use std::path::Path;

pub fn update_mod_extension(
    root: &Path,
    id: &str,
    expected_revision: &str,
    update: impl FnOnce(&mut serde_json::Value) -> Result<(), String>,
) -> Result<(), String> {
    let package = find_mod_package(root, id).ok_or("mod package not found")?;
    let manifest = crate::config::read_manifest_path(root, &package)?;
    let current_state =
        serde_json::json!({"enabled":manifest.enabled,"settings":manifest.setting_values});
    if config_revision(&current_state) != expected_revision {
        return Err("Mod settings changed since they were displayed. Reload before saving".into());
    }
    let mut extension = crate::ensure_eml_extension::ensure_eml_extension(&package, &manifest)?;
    update(&mut extension)?;
    crate::config::validate_document(root, crate::documents::EXTENDED_MOD, &extension)?;
    let wire_manifest = serde_json::from_slice(&read_package_file(&package, "mod.json")?)
        .map_err(|error| error.to_string())?;
    crate::config::parse_manifest_with_extension(root, wire_manifest, Some(extension.clone()))?;
    write_package_json(&package, "extended.mod.json", &extension)
}
