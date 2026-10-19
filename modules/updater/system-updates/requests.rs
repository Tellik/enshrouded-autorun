use crate::*;
#[cfg(windows)]
pub fn request_staged_install(root: &std::path::Path, pid: u32) -> Result<(), String> {
    if !crate::update_check::updates_enabled(root) {
        return Err(crate::UPDATE_MODULE_DISABLED_MESSAGE.into());
    }
    let staged = shroudforge_loader_config::paths::updates_dir(root).join("pending");
    let release: serde_json::Value = serde_json::from_slice(
        &std::fs::read(staged.join("shroudforge/version.json"))
            .map_err(|error| format!("pending update manifest is unavailable: {error}"))?,
    )
    .map_err(|error| format!("pending update manifest is invalid: {error}"))?;
    let version = release["version"]
        .as_str()
        .ok_or("pending update manifest has no version")?;
    let id = format!("system-staged:{version}");
    enqueue_update_item(
        root,
        serde_json::json!({
            "id": id,
            "kind": "system",
            "operation": "install-staged",
            "title": format!("Install staged ShroudForge {version}"),
            "version": version,
            "state": "queued",
            "selected": true,
            "message": "Verified package is ready for installation",
            "payload": {"version":version,"waitPid":pid}
        }),
    )?;
    start_update_queue(root, &[id], true, false)
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
    let checksum = release["checksum"]
        .as_str()
        .ok_or("release has no SHA-256 checksum")?;
    if release["downloadUrl"]
        .as_str()
        .is_none_or(|url| !url.starts_with("https://"))
        || checksum.len() != 64
        || !checksum.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("release download URL or SHA-256 checksum is invalid".into());
    }
    crate::parse_release_tag(version)?;
    enqueue_update_item(
        root,
        serde_json::json!({"id":format!("system:{version}"),"kind":"system","title":format!("ShroudForge {version}"),"version":version,"prerelease":release["prerelease"].as_bool().unwrap_or(false),"state":"queued","selected":true,"releaseUrl":release["releaseUrl"],"payload":release}),
    )
    .map(|_| ())
}

#[cfg(not(windows))]
pub fn enqueue_system_update(_: &std::path::Path, _: serde_json::Value) -> Result<(), String> {
    Err("update queue is available on Windows only".into())
}

#[cfg(windows)]
pub fn request_system_update(
    root: &std::path::Path,
    release: serde_json::Value,
    wait_for_game: bool,
    show_window: bool,
) -> Result<(), String> {
    let version = release["version"]
        .as_str()
        .ok_or("release has no version")?
        .to_owned();
    enqueue_system_update(root, release)?;
    start_update_queue(
        root,
        &[format!("system:{version}")],
        wait_for_game,
        show_window,
    )
}

#[cfg(windows)]
pub fn cancel_system_update(root: &std::path::Path) -> Result<(), String> {
    crate::cancel_update_queue(root)
}

#[cfg(not(windows))]
pub fn cancel_system_update(_: &std::path::Path) -> Result<(), String> {
    Err("system update cancellation is available on Windows only".into())
}

#[cfg(not(windows))]
pub fn request_system_update(
    _: &std::path::Path,
    _: serde_json::Value,
    _: bool,
    _: bool,
) -> Result<(), String> {
    Err("scheduled updater is available on Windows only".into())
}

#[cfg(not(windows))]
pub fn request_staged_install(_: &std::path::Path, _: u32) -> Result<(), String> {
    Err("scheduled updater is available on Windows only".into())
}
