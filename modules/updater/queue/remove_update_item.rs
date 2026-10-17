#[cfg(windows)]
use crate::update_queue_storage::update_update_queue;

#[cfg(windows)]
pub fn remove_update_queue_item(root: &std::path::Path, id: &str) -> Result<(), String> {
    update_update_queue(root, |queue| {
        let items = queue["items"]
            .as_array_mut()
            .ok_or("update queue has an invalid items list")?;
        if items.iter().any(|item| {
            item["id"].as_str() == Some(id)
                && matches!(item["state"].as_str(), Some("downloading" | "installing"))
        }) {
            return Err("an active update cannot be removed".into());
        }
        items.retain(|item| item["id"].as_str() != Some(id));
        Ok(())
    })
}

#[cfg(not(windows))]
pub fn remove_update_queue_item(_: &std::path::Path, _: &str) -> Result<(), String> {
    Err("update queue is available on Windows only".into())
}
