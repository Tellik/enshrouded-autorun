use serde_json::json;
use std::path::Path;

pub fn request(root: &Path, action: &str) -> Result<(), String> {
    request_job(root, action, None)
}

pub fn request_job(root: &Path, action: &str, job_id: Option<&str>) -> Result<(), String> {
    if !["start", "stop", "snapshot"].contains(&action) {
        return Err("invalid diagnostic action".into());
    }
    let config = shroudforge_loader_config::config::read_loader(root)?;
    let settings = &config["modules"]["runtimeDiagnostics"];
    if action != "stop" && settings["enabled"] == false {
        return Err("Runtime Diagnostics module is disabled in Settings. The diagnostics service is unavailable until it is enabled.".into());
    }
    let job = if action == "stop" {
        None
    } else {
        Some(crate::job_catalog::resolve(root, settings, job_id)?)
    };
    shroudforge_loader_config::config::update_loader(root, |value| {
        let settings = &mut value["modules"]["runtimeDiagnostics"];
        settings["sessionActive"] = json!(action != "stop");
        if let Some(job) = job {
            settings["jobId"] = json!(job.id);
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
