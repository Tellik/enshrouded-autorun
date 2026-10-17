use serde_json::{Value, json};
use std::{fs, path::Path};

pub(crate) fn read_state_file(root: &Path) -> Result<Value, String> {
    let state_path = crate::paths::state_file(root);
    match fs::read(&state_path) {
        Ok(bytes) => {
            let value: Value = serde_json::from_slice(&bytes)
                .map_err(|error| format!("{}: {error}", state_path.display()))?;
            let schema: Value = serde_json::from_str(include_str!("state.schema.json"))
                .map_err(|error| error.to_string())?;
            jsonschema::validator_for(&schema)
                .map_err(|error| error.to_string())?
                .validate(&value)
                .map_err(|error| format!("{}: {error}", state_path.display()))?;
            Ok(value)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(json!({"schemaVersion":1}))
        }
        Err(error) => Err(error.to_string()),
    }
}
