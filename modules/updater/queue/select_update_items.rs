#[cfg(windows)]
use crate::update_queue_storage::update_update_queue;

#[cfg(windows)]
pub fn select_update_queue_item(
    root: &std::path::Path,
    id: &str,
    selected: bool,
) -> Result<(), String> {
    update_update_queue(root, |queue| {
        let item = queue["items"]
            .as_array_mut()
            .and_then(|items| {
                items
                    .iter_mut()
                    .find(|item| item["id"].as_str() == Some(id))
            })
            .ok_or("update queue item was not found")?;
        if matches!(item["state"].as_str(), Some("downloading" | "installing")) {
            return Err("an active update cannot change selection".into());
        }
        item["selected"] = serde_json::Value::Bool(selected);
        Ok(())
    })
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
    update_update_queue(root, |queue| {
        let items = queue["items"]
            .as_array_mut()
            .ok_or("update queue has an invalid items list")?;
        for item in items {
            if ids
                .iter()
                .any(|id| item["id"].as_str() == Some(id.as_str()))
            {
                if matches!(item["state"].as_str(), Some("downloading" | "installing")) {
                    return Err("an active update cannot change selection".into());
                }
                item["selected"] = serde_json::Value::Bool(selected);
            }
        }
        Ok(())
    })
}

#[cfg(not(windows))]
pub fn select_update_queue_items(_: &std::path::Path, _: &[String], _: bool) -> Result<(), String> {
    Err("update queue is available on Windows only".into())
}
