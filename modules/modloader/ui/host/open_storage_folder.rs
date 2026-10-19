use super::read_storage_paths::storage_path;
use std::fs;
use std::path::Path;

pub(crate) fn open_storage_path(root: &Path, key: &str) -> Result<(), String> {
    let path = match key {
        "diagnostic-jobs" => shroudforge_loader_config::paths::diagnostic_jobs_dir(root),
        "diagnostic-exports" => {
            shroudforge_loader_config::paths::export_dir(root).join("diagnostics")
        }
        _ => storage_path(root, key)?,
    };
    fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    std::process::Command::new("explorer.exe")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}
