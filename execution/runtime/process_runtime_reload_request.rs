use super::*;

impl IngameRuntime {
    pub(super) fn process_runtime_reload_request(&mut self) {
        let request_path = shroudforge_loader_config::paths::runtime_dir(&self.root)
            .join("mod-reload-request.json");
        let Ok(bytes) = std::fs::read(&request_path) else {
            return;
        };
        let Ok(request) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            return;
        };
        let Some(request_id) = request["requestId"].as_str().map(str::to_owned) else {
            return;
        };
        if self.processed_reload_request.as_deref() == Some(&request_id) {
            return;
        }
        self.processed_reload_request = Some(request_id.clone());
        let ids = request["modIds"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|value| value.as_str().map(str::to_owned))
            .collect::<Vec<_>>();
        let result = match request["operation"].as_str().unwrap_or("reload") {
            "unload" => self.unload_runtime_packages(&ids),
            "reload" => self.reload_runtime_packages(&ids),
            operation => Err(format!("unknown runtime mod operation: {operation}")),
        };
        let status_path = shroudforge_loader_config::paths::runtime_dir(&self.root)
            .join("mod-reload-result.json");
        let value = match result {
            Ok(()) => {
                let status = if request["operation"] == "unload" {
                    "unloaded"
                } else {
                    "reloaded"
                };
                serde_json::json!({"requestId":request_id,"status":status,"message":"Runtime mod packages updated"})
            }
            Err(error) => {
                serde_json::json!({"requestId":request_id,"status":"error","message":error})
            }
        };
        let _ = shroudforge_loader_config::config::write_json(&status_path, &value);
    }
}
