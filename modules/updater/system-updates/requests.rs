use crate::*;
#[cfg(windows)]
pub fn request_install_after_game(root: &std::path::Path, pid: u32) -> Result<(), String> {
    let request = serde_json::json!({"schemaVersion":1,"operation":"installPending","waitPid":pid});
    shroudforge_loader_config::config::write_json(
        &shroudforge_loader_config::paths::updates_dir(root).join("worker-queue.json"),
        &request,
    )
    .map_err(|e| e.to_string())?;
    request_worker(root)
}


#[cfg(windows)]
pub fn check_update_cancelled(root: &std::path::Path) -> Result<(), String> {
    scheduled::check_update_cancelled(root)
}


#[cfg(not(windows))]
pub fn check_update_cancelled(_: &std::path::Path) -> Result<(), String> {
    Ok(())
}


#[cfg(windows)]
pub fn enqueue_system_update(
    root: &std::path::Path,
    release: serde_json::Value,
) -> Result<(), String> {
    let version = release["version"]
        .as_str()
        .ok_or("release has no version")?;
    if release["downloadUrl"]
        .as_str()
        .map_or(true, |url| !url.starts_with("https://"))
        || release["checksum"].as_str().is_none()
    {
        return Err("release download URL or checksum is missing".into());
    }
    enqueue_update_item(
        root,
        serde_json::json!({"id":format!("system:{version}"),"kind":"system","title":format!("ShroudForge {version}"),"version":version,"prerelease":release["prerelease"].as_bool().unwrap_or(false),"state":"queued","selected":true,"message":release["message"],"releaseUrl":release["releaseUrl"],"payload":release}),
    )
}


#[cfg(not(windows))]
pub fn enqueue_system_update(_: &std::path::Path, _: serde_json::Value) -> Result<(), String> {
    Err("update queue is available on Windows only".into())
}


#[cfg(windows)]
pub fn request_system_stage(
    root: &std::path::Path,
    release: serde_json::Value,
    _wait_pid: u32,
    wait_for_game: bool,
) -> Result<(), String> {
    let version = release["version"]
        .as_str()
        .ok_or("release has no version")?
        .to_owned();
    enqueue_system_update(root, release)?;
    start_update_queue(root, &[format!("system:{version}")], wait_for_game, true)
}


#[cfg(windows)]
pub fn cancel_system_update(root: &std::path::Path) -> Result<(), String> {
    scheduled::clear_system_update(root)
}


#[cfg(not(windows))]
pub fn cancel_system_update(_: &std::path::Path) -> Result<(), String> {
    Err("system update cancellation is available on Windows only".into())
}


#[cfg(not(windows))]
pub fn request_system_stage(
    _: &std::path::Path,
    _: serde_json::Value,
    _: u32,
    _: bool,
) -> Result<(), String> {
    Err("scheduled updater is available on Windows only".into())
}


#[cfg(not(windows))]
pub fn request_install_after_game(_: &std::path::Path, _: u32) -> Result<(), String> {
    Err("scheduled updater is available on Windows only".into())
}
