#[cfg(windows)]
use crate::enqueue_system_update;

#[cfg(windows)]
pub(crate) fn queue_stored_system_update(
    root: &std::path::Path,
    _parameters: &serde_json::Value,
) -> Result<String, String> {
    if !crate::update_check::system_updates_enabled(root) {
        return Err("System updates module is disabled".into());
    }
    let release = shroudforge_loader_config::config::read_loader(root)?
        .pointer("/modules/updates/system/latestRelease")
        .cloned()
        .filter(serde_json::Value::is_object)
        .ok_or("Run checkUpdates first. No verified release is stored in modloader-config.json")?;
    if release["updateAvailable"].as_bool() != Some(true) {
        return Err("the stored release is not newer than the installed version".into());
    }
    enqueue_system_update(root, release)?;
    Ok("System update added to the shared update queue".into())
}
