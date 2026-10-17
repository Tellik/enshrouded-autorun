use crate::config::{document_path, update_state_section, validate_document, write_json};
use serde_json::Value;
use std::path::Path;

pub fn write_document(root: &Path, document: crate::Document, value: &Value) -> Result<(), String> {
    if matches!(document.storage, crate::Storage::Package) {
        return Err("mod.json is package data. Edit the mod package instead".into());
    }
    validate_document(root, document, value)?;
    if let Some(section) = document.state_section() {
        return update_state_section(root, section, |_| Ok(value.clone())).and_then(|_| Ok(()));
    }
    write_json(&document_path(root, document), value)
}
