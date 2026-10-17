use crate::Document;
use serde_json::Value;
use std::path::Path;
pub fn validate_document(_root: &Path, document: Document, value: &Value) -> Result<(), String> {
    let name = document.name;
    let schema: Value =
        serde_json::from_str(document.schema).map_err(|e| format!("schema for {name}: {e}"))?;
    let validator = jsonschema::validator_for(&schema).map_err(|e| e.to_string())?;
    validator
        .validate(value)
        .map_err(|e| format!("{name}: {e}"))
}
