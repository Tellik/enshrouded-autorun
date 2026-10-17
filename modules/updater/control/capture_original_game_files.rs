
#[cfg(windows)]
pub(crate) fn capture_original_game_files(
    root: &std::path::Path,
    _parameters: &serde_json::Value,
) -> Result<String, String> {
    let target = if root.join("enshrouded_server.exe").is_file() {
        "server"
    } else if root.join("enshrouded.exe").is_file() {
        "client"
    } else {
        return Err("could not identify the Enshrouded installation target".into());
    };
    let status = shroudforge_game_assets::originals::ensure_originals(root, target)?;
    Ok(format!(
        "Original gamefile backup status: {}",
        status["status"].as_str().unwrap_or("unknown")
    ))
}
