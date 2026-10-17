use crate::directory;
use std::{fs, path::Path};

pub fn validate_event(root: &Path, value: &serde_json::Value) -> Result<(), String> {
    let path = directory(root).join("event-schema.json");
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            include_str!("event-schema.json").into()
        }
        Err(error) => return Err(error.to_string()),
    };
    let schema = serde_json::from_str(&source).map_err(|e| e.to_string())?;
    jsonschema::validator_for(&schema)
        .map_err(|e| e.to_string())?
        .validate(value)
        .map_err(|e| e.to_string())
}
