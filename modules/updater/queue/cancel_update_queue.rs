#[cfg(windows)]
use crate::scheduled;
#[cfg(windows)]
use crate::update_queue_storage::update_update_queue;

#[cfg(windows)]
pub fn cancel_update_queue(root: &std::path::Path) -> Result<(), String> {
    update_update_queue(root, |queue| {
        if queue["run"]["state"] != "running" {
            return Err("there is no running update queue to cancel".into());
        }
        if queue["run"]["installing"] == true {
            return Err("the update is being installed and can no longer be cancelled".into());
        }
        scheduled::clear_system_update(root)?;
        Ok(())
    })?;
    Ok(())
}

#[cfg(not(windows))]
pub fn cancel_update_queue(_: &std::path::Path) -> Result<(), String> {
    Err("update queue is available on Windows only".into())
}
