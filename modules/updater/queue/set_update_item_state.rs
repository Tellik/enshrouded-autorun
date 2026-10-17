#[cfg(windows)]
use crate::scheduled;
#[cfg(windows)]
use crate::update_queue_storage::update_update_queue;

#[cfg(windows)]
pub(crate) fn set_update_item_state(
    root: &std::path::Path,
    id: &str,
    state: &str,
    message: Option<&str>,
) -> Result<(), String> {
    update_update_queue(root, |queue| {
        let item = queue["items"]
            .as_array_mut()
            .and_then(|items| {
                items
                    .iter_mut()
                    .find(|item| item["id"].as_str() == Some(id))
            })
            .ok_or_else(|| format!("update queue item '{id}' disappeared"))?;
        item["state"] = serde_json::Value::String(state.into());
        item["message"] = message
            .map(|message| serde_json::Value::String(message.into()))
            .unwrap_or(serde_json::Value::Null);
        Ok(())
    })
}

#[cfg(windows)]
pub fn set_active_queue_item_state(
    root: &std::path::Path,
    id: &str,
    state: &str,
    message: Option<&str>,
) -> Result<(), String> {
    set_update_item_state(root, id, state, message)?;
    scheduled::write_status(root, state, message.unwrap_or(""));
    Ok(())
}

#[cfg(not(windows))]
pub fn set_active_queue_item_state(
    _: &std::path::Path,
    _: &str,
    _: &str,
    _: Option<&str>,
) -> Result<(), String> {
    Err("update queue is available on Windows only".into())
}
