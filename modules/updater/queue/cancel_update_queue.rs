#[cfg(windows)]
use crate::scheduled;
#[cfg(windows)]
use crate::update_queue_storage::update_update_queue;

#[cfg(windows)]
pub fn cancel_update_queue(root: &std::path::Path) -> Result<(), String> {
    let status = std::fs::read(
        shroudforge_loader_config::paths::updates_dir(root).join("updater-status.json"),
    )
    .ok()
    .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
    .and_then(|value| value["status"].as_str().map(str::to_owned));
    if status.as_deref() == Some("installing") {
        return Err("the update is being installed and can no longer be cancelled".into());
    }
    update_update_queue(root, |queue| {
        if queue["run"]["state"] != "running" {
            return Err("there is no running update queue to cancel".into());
        }
        Ok(())
    })?;
    scheduled::clear_system_update(root)
}

#[cfg(not(windows))]
pub fn cancel_update_queue(_: &std::path::Path) -> Result<(), String> {
    Err("update queue is available on Windows only".into())
}
