use crate::config::update_state_section;
use serde_json::{Value, json};
use std::path::Path;

pub fn request_window_visibility(root: &Path, module: &str, visible: bool) -> Result<(), String> {
    if !matches!(module, "modloaderUi" | "debugConsole" | "worldEditor") {
        return Err("unknown window module".into());
    }
    update_state_section(root, "windows", |existing| {
        let mut state = existing.cloned().unwrap_or_else(|| json!({}));
        let current = state.get(module).cloned().unwrap_or_else(|| json!({}));
        let request_id = current
            .get("requestId")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .saturating_add(1);
        let mut window = current.as_object().cloned().unwrap_or_default();
        window.insert(
            "visible".into(),
            json!(
                current
                    .get("visible")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            ),
        );
        window.insert("requestedVisible".into(), json!(visible));
        window.insert("requestId".into(), json!(request_id));
        state[module] = Value::Object(window);
        Ok(state)
    })
}
