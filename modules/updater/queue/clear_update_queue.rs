#[cfg(windows)]
use crate::update_queue_storage::update_update_queue;

#[cfg(windows)]
pub fn clear_update_queue(root: &std::path::Path) -> Result<(), String> {
    update_update_queue(root, |queue| {
        if queue["run"]["state"] == "running" {
            return Err("cancel the running update queue before clearing it".into());
        }
        let items = queue["items"]
            .as_array_mut()
            .ok_or("update queue has an invalid items list")?;
        items.retain(|item| {
            item["kind"] == "maintenance" && crate::maintenance_jobs::is_required_migration(item)
        });
        queue["run"] = serde_json::Value::Null;
        Ok(())
    })
}

#[cfg(not(windows))]
pub fn clear_update_queue(_: &std::path::Path) -> Result<(), String> {
    Err("update queue is available on Windows only".into())
}
