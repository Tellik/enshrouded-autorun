#[cfg(windows)]
use crate::update_queue_storage::read_update_queue_value;

#[cfg(windows)]
pub fn read_update_queue(root: &std::path::Path) -> serde_json::Value {
    read_update_queue_value(root)
}

#[cfg(not(windows))]
pub fn read_update_queue(_: &std::path::Path) -> serde_json::Value {
    serde_json::json!({"schemaVersion":1,"items":[],"run":null})
}
