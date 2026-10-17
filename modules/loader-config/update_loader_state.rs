use crate::config::{installation_lock, write_json};
use crate::read_loader_state::read_state_file;
use serde_json::{Value, json};
use std::path::Path;

pub fn update_state_section(
    root: &Path,
    section: &str,
    update: impl FnOnce(Option<&Value>) -> Result<Value, String>,
) -> Result<(), String> {
    let _lock = installation_lock(root)?;
    let mut state = read_state_file(root)?;
    if state
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .is_some_and(|version| version != 1)
    {
        return Err(format!(
            "unsupported schemaVersion in {}",
            crate::paths::state_file(root).display()
        ));
    }
    state["schemaVersion"] = json!(1);
    let value = update(state.get(section))?;
    state[section] = value;
    write_json(&crate::paths::state_file(root), &state)
}
