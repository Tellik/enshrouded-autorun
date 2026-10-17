#[cfg(windows)]
use crate::update_queue_storage::update_update_queue;

#[cfg(windows)]
pub(crate) fn enqueue_update_item(
    root: &std::path::Path,
    item: serde_json::Value,
) -> Result<(), String> {
    let id = item["id"]
        .as_str()
        .ok_or("update queue item has no ID")?
        .to_owned();
    update_update_queue(root, |queue| {
        let items = queue["items"]
            .as_array_mut()
            .ok_or("update queue has an invalid items list")?;
        if let Some(existing) = items
            .iter_mut()
            .find(|existing| existing["id"].as_str() == Some(&id))
        {
            if existing["state"] == "complete" {
                *existing = item;
            }
        } else {
            items.push(item);
        }
        Ok(())
    })
}
