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
        let existing_index = items
            .iter()
            .position(|existing| existing["id"].as_str() == Some(&id));
        if existing_index.is_some_and(|index| {
            !matches!(
                items[index]["state"].as_str(),
                Some("complete" | "error" | "cancelled")
            )
        }) {
            return Ok(false);
        }
        if item["kind"] == "installer" && item["operation"] == "uninstall" {
            for existing in items
                .iter_mut()
                .filter(|existing| existing["kind"] != "maintenance")
            {
                existing["selected"] = serde_json::Value::Bool(false);
            }
        }
        if let Some(existing_index) = existing_index {
            items[existing_index] = item;
        } else {
            items.push(item);
        }
        Ok(true)
    })
}
