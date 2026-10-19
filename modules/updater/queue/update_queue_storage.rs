pub(crate) fn update_queue_path(root: &std::path::Path) -> std::path::PathBuf {
    shroudforge_loader_config::paths::updates_dir(root).join("update-queue.json")
}

pub(crate) fn new_update_queue_run_id() -> String {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}-{stamp}", std::process::id())
}

pub(crate) fn update_queue_run_token(run: &serde_json::Value) -> Option<String> {
    if let Some(run_id) = run["runId"].as_str().filter(|value| !value.is_empty()) {
        return Some(run_id.to_owned());
    }
    let started_at = run["startedAt"].as_u64()?;
    let mut item_ids = run["itemIds"]
        .as_array()?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    item_ids.sort();
    serde_json::to_string(&(started_at, item_ids))
        .ok()
        .map(|value| format!("legacy:{value}"))
}

pub(crate) fn read_update_queue_value(root: &std::path::Path) -> serde_json::Value {
    try_read_update_queue_value(root).unwrap_or_else(|error| {
        serde_json::json!({
            "schemaVersion": 1,
            "items": [],
            "run": null,
            "error": format!("The update queue could not be read and was preserved: {error}")
        })
    })
}

pub(crate) fn try_read_update_queue_value(
    root: &std::path::Path,
) -> Result<serde_json::Value, String> {
    let path = update_queue_path(root);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(serde_json::json!({"schemaVersion":1,"items":[],"run":null}));
        }
        Err(error) => {
            return Err(format!("could not read {}: {error}", path.display()));
        }
    };
    let mut value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{} contains invalid JSON: {error}", path.display()))?;
    if !value["items"].is_array() {
        return Err(format!(
            "{} does not contain a valid item list",
            path.display()
        ));
    }
    if value.get("run").is_none() {
        value["run"] = serde_json::Value::Null;
    }
    Ok(value)
}

pub(crate) fn write_update_queue_value(
    root: &std::path::Path,
    value: &serde_json::Value,
) -> Result<(), String> {
    shroudforge_loader_config::config::write_json(&update_queue_path(root), value)
        .map_err(|error| error.to_string())
}

pub(crate) fn update_update_queue<T>(
    root: &std::path::Path,
    update: impl FnOnce(&mut serde_json::Value) -> Result<T, String>,
) -> Result<T, String> {
    let _lock = shroudforge_loader_config::config::installation_lock(root)?;
    let mut queue = try_read_update_queue_value(root)?;
    let result = update(&mut queue)?;
    write_update_queue_value(root, &queue)?;
    Ok(result)
}
