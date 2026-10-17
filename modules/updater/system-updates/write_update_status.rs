use std::{fs, path::Path};

pub(crate) fn write_status(root: &Path, status: &str, message: &str) {
    write_status_with_version(root, status, message, None);
}

pub(crate) fn write_status_with_version(
    root: &Path,
    status: &str,
    message: &str,
    version: Option<&str>,
) {
    let previous_path =
        shroudforge_loader_config::paths::updates_dir(root).join("updater-status.json");
    let previous = fs::read(&previous_path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
    let keep_progress = status != "queued";
    let version = version.map(str::to_owned).or_else(|| {
        previous
            .as_ref()
            .and_then(|value| value["version"].as_str())
            .map(str::to_owned)
    });
    let downloaded = if keep_progress {
        previous
            .as_ref()
            .and_then(|value| value["downloadedBytes"].as_u64())
            .unwrap_or(0)
    } else {
        0
    };
    let total = if keep_progress {
        previous
            .as_ref()
            .and_then(|value| value.get("totalBytes"))
            .cloned()
            .unwrap_or(serde_json::Value::Null)
    } else {
        serde_json::Value::Null
    };
    let speed = if keep_progress {
        previous
            .as_ref()
            .and_then(|value| value.get("bytesPerSecond"))
            .cloned()
            .unwrap_or(serde_json::Value::Null)
    } else {
        serde_json::Value::Null
    };
    let value = serde_json::json!({"schemaVersion":1,"operation":"systemStage","status":status,"step":status,"message":message,"version":version,"downloadedBytes":downloaded,"totalBytes":total,"bytesPerSecond":speed,"updatedAt":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()});
    let _ = shroudforge_loader_config::config::write_json(
        &shroudforge_loader_config::paths::updates_dir(root).join("updater-status.json"),
        &value,
    );
}

pub(crate) fn write_download_progress(
    root: &Path,
    version: &str,
    downloaded: u64,
    total: Option<u64>,
    speed: Option<u64>,
) {
    let value = serde_json::json!({"schemaVersion":1,"operation":"systemStage","status":"downloading","step":"download","message":"Downloading ShroudForge update","version":version,"downloadedBytes":downloaded,"totalBytes":total,"bytesPerSecond":speed,"updatedAt":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()});
    let _ = shroudforge_loader_config::config::write_json(
        &shroudforge_loader_config::paths::updates_dir(root).join("updater-status.json"),
        &value,
    );
}
