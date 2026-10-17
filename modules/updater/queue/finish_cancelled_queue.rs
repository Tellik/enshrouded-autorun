#[cfg(windows)]
use crate::scheduled;
#[cfg(windows)]
use crate::set_update_item_state::set_update_item_state;
#[cfg(windows)]
use crate::update_queue_storage::read_update_queue_value;
#[cfg(windows)]
use crate::update_queue_storage::update_update_queue;

#[cfg(windows)]
pub(crate) fn finish_cancelled_queue(root: &std::path::Path, ids: &[String]) {
    for id in ids {
        let queue = read_update_queue_value(root);
        let state = queue["items"]
            .as_array()
            .and_then(|items| items.iter().find(|item| item["id"].as_str() == Some(id)))
            .and_then(|item| item["state"].as_str());
        if !matches!(state, Some("complete" | "error")) {
            let _ = set_update_item_state(root, id, "cancelled", Some("Cancelled by user"));
        }
    }
    let _ = update_update_queue(root, |queue| {
        queue["run"]["state"] = serde_json::Value::String("cancelled".into());
        queue["run"]["finishedAt"] = serde_json::json!(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
        Ok(())
    });
    let updates = shroudforge_loader_config::paths::updates_dir(root);
    let _ = std::fs::remove_file(updates.join("mod-install-queue.json"));
    let _ = std::fs::remove_file(updates.join("worker-queue.json"));
    let _ = std::fs::remove_file(updates.join("system-stage-request.json"));
    let _ = std::fs::remove_file(updates.join("download.zip"));
    let _ = std::fs::remove_dir_all(updates.join("pending-download"));
    let _ = std::fs::remove_file(updates.join("pending.ready"));
    scheduled::write_status(root, "cancelled", "Selected update queue cancelled");
    let _ = std::fs::remove_file(updates.join("cancel-system-update"));
}
