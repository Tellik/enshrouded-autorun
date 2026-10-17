use crate::validate_event;
use std::path::Path;

pub fn write_event(root: &Path, filename: &str, value: &serde_json::Value) -> Result<(), String> {
    if filename.is_empty()
        || filename.len() > 240
        || filename == "."
        || filename == ".."
        || !filename
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_' | b'+'))
    {
        return Err("invalid notification filename".into());
    }
    validate_event(root, value)?;
    crate::config::update_state_section(root, "events", |stored| {
        let mut events = stored.cloned().unwrap_or_else(|| serde_json::json!({}));
        events[format!("{filename}.json")] = value.clone();
        Ok(events)
    })
}
