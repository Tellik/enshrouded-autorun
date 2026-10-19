use crate::update_queue_storage::update_update_queue;

pub(crate) fn enqueue_update_item(
    root: &std::path::Path,
    item: serde_json::Value,
) -> Result<bool, String> {
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
            if matches!(
                existing["state"].as_str(),
                Some("complete" | "error" | "cancelled")
            ) {
                *existing = item;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            items.push(item);
            Ok(true)
        }
    })
}
