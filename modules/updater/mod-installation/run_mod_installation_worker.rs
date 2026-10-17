use crate::install_catalog_mod::install_mod_from_catalog;
use crate::provider_config::ProviderConfig;
use shroudforge_logging::write_activity;
use shroudforge_package::package_lookup::valid_identifier;
use std::{fs, path::PathBuf};

pub fn run_catalog_install_worker() -> Result<(), String> {
    let values: Vec<String> = std::env::args().collect();
    let root = values
        .windows(2)
        .find(|pair| pair[0] == "--root")
        .map(|pair| PathBuf::from(&pair[1]))
        .ok_or("missing --root")?;
    let request_path =
        shroudforge_loader_config::paths::updates_dir(&root).join("mod-install-queue.json");
    let result: Result<String, String> = (|| {
        let bytes =
            fs::read(&request_path).map_err(|e| format!("cannot read mod install request: {e}"))?;
        let request: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| format!("invalid mod install request: {e}"))?;
        let queue_item_id = request["queueItemId"]
            .as_str()
            .ok_or("request has no queueItemId")?;
        let project_id = request["projectId"]
            .as_str()
            .ok_or("request has no projectId")?;
        let provider: ProviderConfig = serde_json::from_value(request["provider"].clone())
            .map_err(|e| format!("invalid provider settings: {e}"))?;
        if !valid_identifier(project_id) {
            return Err("invalid mod project ID".into());
        }
        let operation = request["operation"].as_str().unwrap_or("install");
        let update_mod_id = if operation == "update" {
            Some(
                request["modId"]
                    .as_str()
                    .ok_or("update request has no modId")?,
            )
        } else {
            None
        };
        if operation != "install" && operation != "update" {
            return Err(format!("unsupported catalog operation: {operation}"));
        }
        let installed =
            install_mod_from_catalog(&root, &provider, project_id, update_mod_id, queue_item_id)?;
        Ok(format!(
            "{} {} package(s) for {project_id}",
            if update_mod_id.is_some() {
                "Updated"
            } else {
                "Installed"
            },
            installed.len()
        ))
    })();
    match &result {
        Ok(detail) => write_activity(
            &root,
            "ShroudEdit",
            "Install mod",
            "Succeeded",
            Some(detail),
            "success",
        ),
        Err(error) => write_activity(
            &root,
            "ShroudEdit",
            "Install mod",
            "Failed",
            Some(error),
            "error",
        ),
    }
    let _ = fs::remove_file(request_path);
    result.map(|_| ())
}
