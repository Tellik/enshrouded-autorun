use std::path::Path;

pub fn set_mod_enabled(root: &Path, id: &str, enabled: bool, revision: &str) -> Result<(), String> {
    if enabled {
        let package =
            crate::package_lookup::find_mod_package(root, id).ok_or("mod package not found")?;
        let manifest = crate::read_manifest_path(root, &package)?;
        if !manifest.enabled
            && manifest
                .settings
                .iter()
                .any(|setting| matches!(setting.control, Some(crate::SettingControl::Keybind)))
        {
            let config = crate::config::read_loader(root)?;
            crate::validate_hotkey_configuration(
                root,
                &config,
                Some((id, &serde_json::json!({}), Some(true))),
            )?;
        }
    }
    crate::update_mod_configuration::update_mod_configuration(root, id, revision, |extension| {
        extension["enabled"] = enabled.into();
        Ok(())
    })
}
