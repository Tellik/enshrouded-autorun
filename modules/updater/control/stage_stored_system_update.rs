#[cfg(windows)]
use crate::enqueue_system_update;
#[cfg(windows)]
use crate::headless_check_system_updates;
#[cfg(windows)]
use crate::start_update_queue::start_update_queue;

#[cfg(windows)]
pub(crate) fn stage_stored_system_update(
    root: &std::path::Path,
    parameters: &serde_json::Value,
) -> Result<String, String> {
    if !crate::update_check::system_updates_enabled(root) {
        return Err("System updates module is disabled".into());
    }
    let config = shroudforge_loader_config::config::read_loader(root)?;
    let mut release = config
        .pointer("/modules/updates/system/latestRelease")
        .cloned()
        .filter(serde_json::Value::is_object);
    if release.is_none() {
        headless_check_system_updates(root, crate::update_check::SystemUpdateCheckMode::Automatic)?;
        release = shroudforge_loader_config::config::read_loader(root)?
            .pointer("/modules/updates/system/latestRelease")
            .cloned()
            .filter(serde_json::Value::is_object);
    }
    let release = release.ok_or("no valid ShroudForge release is available")?;
    if release["updateAvailable"].as_bool() != Some(true) {
        return Err("ShroudForge is already up to date".into());
    }
    let version = release["version"]
        .as_str()
        .ok_or("stored release has no version")?
        .to_owned();
    enqueue_system_update(root, release)?;
    let wait_for_game = parameters
        .get("waitForGame")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    start_update_queue(root, &[format!("system:{version}")], wait_for_game, false)?;
    Ok("Headless system update queued. The updater will wait for Enshrouded to stop".into())
}
