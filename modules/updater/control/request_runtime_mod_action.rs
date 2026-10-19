#[cfg(windows)]
pub fn request_runtime_mod_reload(
    root: &std::path::Path,
    mod_ids: &[String],
) -> Result<(), String> {
    request_runtime_mod_action(root, "reload", mod_ids)
}

#[cfg(not(windows))]
pub fn request_runtime_mod_reload(_: &std::path::Path, _: &[String]) -> Result<(), String> {
    Err("live runtime mod reload is available on Windows only".into())
}

#[cfg(windows)]
pub fn request_runtime_mod_unload(
    root: &std::path::Path,
    mod_ids: &[String],
) -> Result<(), String> {
    request_runtime_mod_action(root, "unload", mod_ids)
}

#[cfg(not(windows))]
pub fn request_runtime_mod_unload(_: &std::path::Path, _: &[String]) -> Result<(), String> {
    Err("live runtime mod reload is available on Windows only".into())
}

/// Start a small in-process controller for commands written to the loader
/// configuration. The game process owns this thread, while actual downloads
/// and file replacement continue to run in the existing updater worker.
#[cfg(windows)]
fn acquire_runtime_mod_action_lock(
    root: &std::path::Path,
    deadline: std::time::Instant,
) -> Result<std::fs::File, String> {
    use fs2::FileExt;
    use std::io::ErrorKind;

    let runtime_dir = shroudforge_loader_config::paths::runtime_dir(root);
    std::fs::create_dir_all(&runtime_dir)
        .map_err(|error| format!("could not prepare the runtime action directory: {error}"))?;
    let path = runtime_dir.join("mod-reload-action.lock");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)
        .map_err(|error| format!("could not open the runtime action lock: {error}"))?;

    loop {
        match file.try_lock_exclusive() {
            Ok(()) => return Ok(file),
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                if remaining.is_zero() {
                    return Err("another runtime mod action did not finish in time".into());
                }
                std::thread::sleep(remaining.min(std::time::Duration::from_millis(100)));
            }
            Err(error) => {
                return Err(format!(
                    "could not acquire the runtime action lock: {error}"
                ));
            }
        }
    }
}

#[cfg(windows)]
pub(crate) fn request_runtime_mod_action(
    root: &std::path::Path,
    operation: &str,
    mod_ids: &[String],
) -> Result<(), String> {
    let lock_deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let _action_lock = acquire_runtime_mod_action_lock(root, lock_deadline)?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let request_id = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let request_path =
        shroudforge_loader_config::paths::runtime_dir(root).join("mod-reload-request.json");
    let result_path =
        shroudforge_loader_config::paths::runtime_dir(root).join("mod-reload-result.json");
    let request =
        serde_json::json!({"requestId":request_id,"operation":operation,"modIds":mod_ids});
    shroudforge_loader_config::config::write_json(&request_path, &request)
        .map_err(|error| format!("could not request live mod reload: {error}"))?;
    loop {
        if let Ok(bytes) = std::fs::read(&result_path)
            && let Ok(result) = serde_json::from_slice::<serde_json::Value>(&bytes)
            && result["requestId"].as_str() == Some(&request_id)
        {
            let _ = std::fs::remove_file(&request_path);
            let expected_status = if operation == "unload" {
                "unloaded"
            } else {
                "reloaded"
            };
            return if result["status"] == expected_status {
                Ok(())
            } else {
                Err(result["message"]
                    .as_str()
                    .unwrap_or("live runtime mod reload failed")
                    .to_owned())
            };
        }
        if std::time::Instant::now() >= deadline {
            let _ = std::fs::remove_file(&request_path);
            return Err("timed out waiting for the running game to reload its mod runtime".into());
        }
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        std::thread::sleep(remaining.min(std::time::Duration::from_millis(100)));
    }
}
