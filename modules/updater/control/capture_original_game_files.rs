#[cfg(windows)]
pub(crate) fn capture_original_game_files(
    root: &std::path::Path,
    _parameters: &serde_json::Value,
) -> Result<String, String> {
    let target = installation_target(root, None)?;
    let status = shroudforge_game_assets::originals::ensure_originals(root, target)?;
    Ok(format!(
        "Original gamefile backup status: {}",
        status["status"].as_str().unwrap_or("unknown")
    ))
}

#[cfg(windows)]
pub(crate) fn capture_verified_original_game_files(
    root: &std::path::Path,
    parameters: &serde_json::Value,
) -> Result<String, String> {
    let target = installation_target(
        root,
        parameters.get("target").and_then(serde_json::Value::as_str),
    )?;
    let status = shroudforge_game_assets::originals::capture_verified_originals(root, target)?;
    Ok(format!(
        "Original gamefile backup status: {}",
        status["status"].as_str().unwrap_or("unknown")
    ))
}

#[cfg(windows)]
fn installation_target(
    root: &std::path::Path,
    requested: Option<&str>,
) -> Result<&'static str, String> {
    match requested {
        Some("client") => Ok("client"),
        Some("server") => Ok("server"),
        Some(_) => Err("gamefile capture target must be client or server".into()),
        None if root.join("enshrouded_server.exe").is_file() => Ok("server"),
        None if root.join("enshrouded.exe").is_file() => Ok("client"),
        None => Err("could not identify the Enshrouded installation target".into()),
    }
}
