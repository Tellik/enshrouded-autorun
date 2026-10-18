use std::path::Path;

pub fn reset_window_geometry(root: &Path) -> Result<(), String> {
    shroudforge_loader_config::config::update_loader(root, |config| {
        for module in ["modloaderUi", "updates", "debugConsole"] {
            config["modules"][module]["window"]["position"] = serde_json::Value::Null;
        }
        Ok(())
    })
}
