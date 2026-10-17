use crate::config::*;
use crate::read_loader_state::read_state_file;
use serde_json::Value;
use std::{fs, path::Path};
pub fn read_document(root: &Path, document: crate::Document) -> Result<Value, String> {
    if matches!(document.storage, crate::Storage::Package) {
        return Err("mod.json is package data. Read it from the mod package instead".into());
    }
    if let Some(section) = document.state_section() {
        let state_path = crate::paths::state_file(root);
        let state = read_state_file(root)?;
        let Some(value) = state.get(section) else {
            return Err(format!("{} has no {section} state", state_path.display()));
        };
        validate_document(root, document, value)?;
        return Ok(value.clone());
    }
    let path = document_path(root, document);
    let value: Value = serde_json::from_slice(
        &fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?,
    )
    .map_err(|error| format!("{}: {error}", path.display()))?;
    validate_document(root, document, &value)?;
    Ok(value)
}
