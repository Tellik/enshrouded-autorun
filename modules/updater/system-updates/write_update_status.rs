use std::sync::atomic::{AtomicBool, Ordering};
use std::{fs, path::Path};

static DOWNLOAD_PROGRESS_WRITE_FAILED: AtomicBool = AtomicBool::new(false);
static FILE_PROGRESS_WRITE_FAILED: AtomicBool = AtomicBool::new(false);

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
    if let Err(error) = shroudforge_loader_config::config::write_json(
        &shroudforge_loader_config::paths::updates_dir(root).join("updater-status.json"),
        &value,
    ) {
        let _ = shroudforge_logging::append(
            root,
            'W',
            "updater",
            &format!("Could not save updater status '{status}': {error}"),
        );
    }
}

pub(crate) fn write_download_progress(
    root: &Path,
    version: &str,
    downloaded: u64,
    total: Option<u64>,
    speed: Option<u64>,
) {
    let value = serde_json::json!({"schemaVersion":1,"operation":"systemStage","status":"downloading","step":"download","message":"Downloading ShroudForge update","version":version,"downloadedBytes":downloaded,"totalBytes":total,"bytesPerSecond":speed,"updatedAt":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()});
    match shroudforge_loader_config::config::write_json(
        &shroudforge_loader_config::paths::updates_dir(root).join("updater-status.json"),
        &value,
    ) {
        Ok(()) => DOWNLOAD_PROGRESS_WRITE_FAILED.store(false, Ordering::Release),
        Err(error) => {
            if !DOWNLOAD_PROGRESS_WRITE_FAILED.swap(true, Ordering::AcqRel) {
                let _ = shroudforge_logging::append(
                    root,
                    'W',
                    "updater",
                    &format!(
                        "Could not save download progress status. The download continues without visible byte progress: {error}"
                    ),
                );
            }
        }
    }
}

pub(crate) fn write_file_progress(
    root: &Path,
    status: &str,
    phase: &str,
    completed_files: usize,
    total_files: usize,
    current_file: &str,
    current_file_bytes: u64,
    current_file_total_bytes: u64,
) {
    let path = shroudforge_loader_config::paths::updates_dir(root).join("updater-status.json");
    let mut value = fs::read(&path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .unwrap_or_else(|| serde_json::json!({"schemaVersion":1,"operation":"systemStage"}));
    value["status"] = serde_json::Value::String(status.to_owned());
    value["step"] = serde_json::Value::String(status.to_owned());
    value["message"] =
        serde_json::Value::String(format!("{phase}: {completed_files}/{total_files} files"));
    value["completedFiles"] = serde_json::json!(completed_files);
    value["totalFiles"] = serde_json::json!(total_files);
    value["currentFile"] = serde_json::Value::String(current_file.to_owned());
    value["currentFileBytes"] = serde_json::json!(current_file_bytes);
    value["currentFileTotalBytes"] = serde_json::json!(current_file_total_bytes);
    value["updatedAt"] = serde_json::json!(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    );
    match shroudforge_loader_config::config::write_json(&path, &value) {
        Ok(()) => FILE_PROGRESS_WRITE_FAILED.store(false, Ordering::Release),
        Err(error) => {
            if !FILE_PROGRESS_WRITE_FAILED.swap(true, Ordering::AcqRel) {
                let _ = shroudforge_logging::append(
                    root,
                    'W',
                    "updater",
                    &format!(
                        "Could not save updater file progress. The operation continues without visible file progress: {error}"
                    ),
                );
            }
        }
    }
}
