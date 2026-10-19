pub(crate) fn choose_folder(key: &str) -> Result<serde_json::Value, String> {
    let title = match key {
        "mods" => "Choose the mods folder",
        "state" => "Choose the status folder",
        "logs" => "Choose the logs folder",
        "cache" => "Choose the cache folder",
        "exports" => "Choose the exports folder",
        "updates" => "Choose the updates folder",
        "ui" => "Choose the UI data folder",
        "runtime" => "Choose the runtime data folder",
        _ => return Err("unsupported storage folder".into()),
    };
    let selected = shroudforge_window_management::choose_folder(title)?
        .ok_or_else(|| "folder selection was cancelled".to_owned())?;
    Ok(serde_json::Value::String(
        selected.to_string_lossy().into_owned(),
    ))
}
