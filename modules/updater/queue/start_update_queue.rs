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
    let selected_ids = update_update_queue(root, |queue| {
        if queue["run"]["state"] == "running" {
            return Err("the update queue is already running".into());
        }
        let items = queue["items"]
            .as_array_mut()
            .ok_or("update queue has an invalid items list")?;
        let selected_ids = if ids.is_empty() {
            items
                .iter()
                .filter(|item| item["selected"].as_bool() == Some(true))
                .filter_map(|item| item["id"].as_str().map(str::to_owned))
                .collect::<Vec<_>>()
        } else {
            ids.to_vec()
        };
        if selected_ids.is_empty() {
            return Err("select at least one update before starting the queue".into());
        }
        if selected_ids.iter().any(|id| id.starts_with("system:"))
            && !crate::update_check::system_updates_enabled(root)
        {
            return Err("System updates module is disabled".into());
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
        queue["run"] = serde_json::json!({"state":"running","waitForGame":wait_for_game,"showWindow":show_window,"itemIds":selected_ids,"startedAt":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()});
        Ok(selected_ids)
    })?;
    let _ = std::fs::remove_file(
        shroudforge_loader_config::paths::updates_dir(root).join("cancel-system-update"),
    );
    scheduled::write_status(root, "queued", "Selected updates are queued");
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

#[cfg(not(windows))]
pub fn start_update_queue(
    _: &std::path::Path,
    _: &[String],
    _: bool,
    _: bool,
) -> Result<(), String> {
    Err("update queue is available on Windows only".into())
}
