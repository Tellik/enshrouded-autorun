use crate::config::update_state_section;
use serde_json::{Value, json};
use std::path::Path;

pub fn request_world_editor_module_settings(root: &Path) -> Result<(), String> {
    update_state_section(root, "windows", |existing| {
        let mut state = existing.cloned().unwrap_or_else(|| json!({}));
        let current = state
            .get("modloaderUi")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let mut window = current.as_object().cloned().unwrap_or_default();
        let request_id = current
            .get("requestId")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .saturating_add(1);
        let focus_request_id = current
            .get("focusRequestId")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .saturating_add(1);
        window.insert(
            "visible".into(),
            json!(
                current
                    .get("visible")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            ),
        );
        window.insert("requestedVisible".into(), json!(true));
        window.insert("requestId".into(), json!(request_id));
        window.insert("focusTarget".into(), json!("worldEditorSettings"));
        window.insert("focusRequestId".into(), json!(focus_request_id));
        state["modloaderUi"] = Value::Object(window);
        Ok(state)
    })
}
