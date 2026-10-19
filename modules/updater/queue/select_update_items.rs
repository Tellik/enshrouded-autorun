#[cfg(windows)]
use crate::update_queue_storage::update_update_queue;
#[cfg(windows)]
use std::collections::HashSet;

#[cfg(windows)]
pub fn select_update_queue_item(
    root: &std::path::Path,
    id: &str,
    selected: bool,
) -> Result<(), String> {
    select_update_queue_items(root, &[id.to_owned()], selected)
}

#[cfg(not(windows))]
pub fn select_update_queue_item(_: &std::path::Path, _: &str, _: bool) -> Result<(), String> {
    Err("update queue is available on Windows only".into())
}

#[cfg(windows)]
pub fn select_update_queue_items(
    root: &std::path::Path,
    ids: &[String],
    selected: bool,
) -> Result<(), String> {
    let requested_ids = ids.iter().map(String::as_str).collect::<HashSet<_>>();
    update_update_queue(root, |queue| {
        if queue["run"]["state"] == "running" {
            return Err("update selection cannot change while the queue is running".into());
        }
        let items = queue["items"]
            .as_array_mut()
            .ok_or("update queue has an invalid items list")?;
        let mut found_ids = HashSet::with_capacity(requested_ids.len());
        for item in items {
            if let Some(id) = item["id"].as_str().filter(|id| requested_ids.contains(id)) {
                found_ids.insert(id.to_owned());
                if !selected
                    && item["kind"] == "maintenance"
                    && crate::maintenance_jobs::is_required_migration(item)
                {
                    return Err("a required updater migration cannot be deselected".into());
                }
                if matches!(item["state"].as_str(), Some("downloading" | "installing")) {
                    return Err("an active update cannot change selection".into());
                }
                item["selected"] = serde_json::Value::Bool(selected);
            }
        }
        if found_ids.len() != requested_ids.len() {
            return Err("one or more update queue items were not found".into());
        }
        Ok(())
    })
}

#[cfg(not(windows))]
pub fn select_update_queue_items(_: &std::path::Path, _: &[String], _: bool) -> Result<(), String> {
    Err("update queue is available on Windows only".into())
}
