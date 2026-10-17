use super::read_storage_paths::storage_path;
use std::fs;
use std::path::Path;

pub(crate) fn open_storage_path(root: &Path, key: &str) -> Result<(), String> {
    let path = storage_path(root, key)?;
    fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    std::process::Command::new("explorer.exe")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}
