use crate::config::update_state_section;
use serde_json::{Value, json};
use std::path::Path;

pub fn publish_modloader_client_pid(root: &Path, pid: u32) -> Result<(), String> {
    update_state_section(root, "windows", |existing| {
        let mut windows = existing.cloned().unwrap_or_else(|| json!({}));
        let current = windows
            .get("modloaderUi")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let mut modloader_ui = current.as_object().cloned().unwrap_or_default();
        modloader_ui.insert("clientPid".into(), json!(pid));
        windows["modloaderUi"] = Value::Object(modloader_ui);
        Ok(windows)
    })
}

pub fn clear_modloader_client_pid(root: &Path, pid: u32) -> Result<(), String> {
    update_state_section(root, "windows", |existing| {
        let mut windows = existing.cloned().unwrap_or_else(|| json!({}));
        if let Some(modloader_ui) = windows
            .get_mut("modloaderUi")
            .and_then(Value::as_object_mut)
            && modloader_ui.get("clientPid").and_then(Value::as_u64) == Some(u64::from(pid))
        {
            modloader_ui.remove("clientPid");
        }
        Ok(windows)
    })
}
