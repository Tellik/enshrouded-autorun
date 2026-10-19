#[cfg(windows)]
use crate::scheduled;
#[cfg(windows)]
use crate::update_queue_storage::update_update_queue;

#[cfg(windows)]
pub fn start_update_queue(
    root: &std::path::Path,
    ids: &[String],
    wait_for_game: bool,
    show_window: bool,
) -> Result<(), String> {
    let _start_lock = crate::acquire_queue_worker_mutex(root)?
        .ok_or("an update or recovery action is already running")?;
    ensure_gamefiles_restore_idle(root)?;
    crate::maintenance_jobs::enqueue_before_update(root)?;
    let selected_ids = update_update_queue(root, |queue| {
        if queue["run"]["state"] == "running" {
            return Err("the update queue is already running".into());
        }
        let items = queue["items"]
            .as_array_mut()
            .ok_or("update queue has an invalid items list")?;
        let mut selected_ids = if ids.is_empty() {
            items
                .iter()
                .filter(|item| item["selected"].as_bool() == Some(true))
                .filter_map(|item| item["id"].as_str().map(str::to_owned))
                .collect::<Vec<_>>()
        } else {
            ids.to_vec()
        };
        let mut unique_ids = std::collections::HashSet::new();
        if selected_ids.iter().any(|id| !unique_ids.insert(id)) {
            return Err("the update selection contains duplicate item IDs".into());
        }
        if selected_ids.is_empty()
            && !items
                .iter()
                .any(|item| item["kind"] == "maintenance" && item["state"] == "queued")
        {
            return Err("select at least one update before starting the queue".into());
        }
        let maintenance_ids = items
            .iter()
            .filter(|item| item["kind"] == "maintenance" && item["state"] == "queued")
            .filter_map(|item| item["id"].as_str().map(str::to_owned))
            .collect::<Vec<_>>();
        for id in maintenance_ids.into_iter().rev() {
            if !selected_ids.contains(&id) {
                selected_ids.insert(0, id);
            }
        }
        if selected_ids
            .iter()
            .any(|id| id.starts_with("system:") || id.starts_with("mod-"))
            && !crate::update_check::updates_enabled(root)
        {
            return Err("Updates module is disabled".into());
        }
        for id in &selected_ids {
            let item = items
                .iter_mut()
                .find(|item| item["id"].as_str() == Some(id.as_str()))
                .ok_or_else(|| format!("update queue item '{id}' was not found"))?;
            if matches!(item["state"].as_str(), Some("downloading" | "installing")) {
                return Err(format!("update queue item '{id}' is already active"));
            }
            item["state"] = serde_json::Value::String("queued".into());
            item["selected"] = serde_json::Value::Bool(true);
            item["waitForGame"] = serde_json::Value::Bool(wait_for_game);
        }
        let wait_pid = selected_ids
            .iter()
            .find_map(|id| {
                items
                    .iter()
                    .find(|item| item["id"].as_str() == Some(id.as_str()))
                    .and_then(|item| item["payload"]["waitPid"].as_u64())
            })
            .unwrap_or_else(|| std::process::id() as u64);
        let cancel_path = shroudforge_loader_config::paths::updates_dir(root)
            .join("cancel-system-update");
        match std::fs::remove_file(cancel_path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!("could not clear an old update cancellation request: {error}"));
            }
        }
        queue["run"] = serde_json::json!({"state":"running","waitForGame":wait_for_game,"waitPid":wait_pid,"showWindow":show_window,"itemIds":selected_ids,"startedAt":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()});
        scheduled::write_status(root, "queued", "Selected updates are queued");
        Ok(selected_ids)
    })?;
    drop(_start_lock);
    if let Err(error) = scheduled::start_system_waiter(root) {
        let _ = update_update_queue(root, |queue| {
            queue["run"] = serde_json::Value::Null;
            if let Some(items) = queue["items"].as_array_mut() {
                for item in items {
                    if selected_ids
                        .iter()
                        .any(|id| item["id"].as_str() == Some(id.as_str()))
                    {
                        item["state"] = serde_json::Value::String("error".into());
                        item["message"] = serde_json::Value::String(error.clone());
                    }
                }
            }
            Ok(())
        });
        return Err(error);
    }
    Ok(())
}

#[cfg(windows)]
pub(crate) fn ensure_gamefiles_restore_idle(root: &std::path::Path) -> Result<(), String> {
    use std::io::ErrorKind;

    let path = shroudforge_loader_config::paths::backups_dir(root)
        .join("gamefiles-restore-status.json");
    let metadata = match std::fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "could not inspect game file restoration status {}: {error}",
                path.display()
            ));
        }
    };
    if !metadata.file_type().is_file() {
        return Err(format!(
            "game file restoration status {} is not a regular file",
            path.display()
        ));
    }
    let bytes = std::fs::read(&path).map_err(|error| {
        format!(
            "could not read game file restoration status {}: {error}",
            path.display()
        )
    })?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "game file restoration status {} contains invalid JSON: {error}",
            path.display()
        )
    })?;
    let status = value["status"].as_str().ok_or_else(|| {
        format!(
            "game file restoration status {} has no valid status",
            path.display()
        )
    })?;
    if matches!(status, "queued" | "waitingForGame" | "restoring") {
        return Err(
            "game files are being restored. Wait for restoration to finish before starting updates"
                .into(),
        );
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn start_update_queue(
    _: &std::path::Path,
    _: &[String],
    _: bool,
    _: bool,
) -> Result<(), String> {
    Err("update queue is available on Windows only".into())
}
