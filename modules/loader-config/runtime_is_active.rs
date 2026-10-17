use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn runtime_is_active(root: &Path) -> bool {
    let heartbeat = crate::paths::runtime_dir(root).join("heartbeat.json");
    let Ok(bytes) = fs::read(heartbeat) else {
        return false;
    };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return false;
    };
    let Some(updated) = value.get("updatedAt").and_then(serde_json::Value::as_u64) else {
        return false;
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    now.saturating_sub(updated) <= 8
        && value
            .get("pid")
            .and_then(serde_json::Value::as_u64)
            .is_some()
}
