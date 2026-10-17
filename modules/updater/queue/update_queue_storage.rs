pub(crate) fn update_queue_path(root: &std::path::Path) -> std::path::PathBuf {
    shroudforge_loader_config::paths::updates_dir(root).join("update-queue.json")
}

pub(crate) fn read_update_queue_value(root: &std::path::Path) -> serde_json::Value {
    std::fs::read(update_queue_path(root))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .filter(|value: &serde_json::Value| value["items"].is_array())
        .unwrap_or_else(|| serde_json::json!({"schemaVersion":1,"items":[],"run":null}))
}

#[cfg(windows)]
pub(crate) fn write_update_queue_value(
    root: &std::path::Path,
    value: &serde_json::Value,
) -> Result<(), String> {
    shroudforge_loader_config::config::write_json(&update_queue_path(root), value)
        .map_err(|error| error.to_string())
}

#[cfg(windows)]
pub(crate) fn update_update_queue<T>(
    root: &std::path::Path,
    update: impl FnOnce(&mut serde_json::Value) -> Result<T, String>,
) -> Result<T, String> {
    let _lock = shroudforge_loader_config::config::installation_lock(root)?;
    let mut queue = read_update_queue_value(root);
    let result = update(&mut queue)?;
    write_update_queue_value(root, &queue)?;
    Ok(result)
}
