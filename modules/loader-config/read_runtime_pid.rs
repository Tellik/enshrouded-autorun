use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn runtime_pid(root: &Path) -> Option<u32> {
    let bytes = fs::read(crate::paths::runtime_dir(root).join("heartbeat.json")).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    let updated = value.get("updatedAt")?.as_u64()?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if now.saturating_sub(updated) > 8 {
        return None;
    }
    value
        .get("pid")?
        .as_u64()
        .and_then(|pid| u32::try_from(pid).ok())
}
