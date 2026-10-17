use serde_json::json;
use std::path::Path;

pub fn request(root: &Path, action: &str) -> Result<(), String> {
    if !["start", "stop", "snapshot"].contains(&action) {
        return Err("invalid diagnostic action".into());
    }
    shroudforge_loader_config::config::update_loader(root, |value| {
        let settings = &mut value["modules"]["runtimeDiagnostics"];
        settings["enabled"] = json!(action != "stop");
        if action != "stop" {
            settings["continuous"] = json!(action == "start");
            settings["requestId"] = json!(
                settings["requestId"]
                    .as_u64()
                    .unwrap_or(0)
                    .saturating_add(1)
            );
        }
        Ok(())
    })
}
