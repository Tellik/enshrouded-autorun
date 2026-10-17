use crate::read_loader_state::read_state_file;
use serde_json::{Value, json};
use std::path::Path;

pub fn window_state(root: &Path) -> Value {
    read_state_file(root)
        .ok()
        .and_then(|state| state.get("windows").cloned())
        .unwrap_or_else(|| json!({}))
}
