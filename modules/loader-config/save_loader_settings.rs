use crate::config::*;
use serde_json::Value;
use std::{fs, path::Path};

pub fn update_loader(
    root: &Path,
    update: impl FnOnce(&mut Value) -> Result<(), String>,
) -> Result<(), String> {
    let path = document_path(root, crate::documents::LOADER);
    fs::create_dir_all(path.parent().unwrap()).map_err(|error| error.to_string())?;
    let _lock = installation_lock(root)?;
    let mut value = read_loader(root)?;
    update(&mut value)?;
    validate_document(root, crate::documents::LOADER, &value)?;
    write_json(&path, &value)
}
