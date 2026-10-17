use super::ui_snapshot_types::StoragePath;
use std::path::Path;
use std::path::PathBuf;

pub(crate) fn storage_path(root: &Path, key: &str) -> Result<PathBuf, String> {
    match key {
        "mods" => Ok(shroudforge_loader_config::paths::mods_dir(root)),
        "state" => Ok(shroudforge_loader_config::paths::state_dir(root)),
        "logs" => Ok(shroudforge_loader_config::paths::logs_dir(root)),
        "cache" => Ok(shroudforge_loader_config::paths::cache_dir(root)),
        "exports" => Ok(shroudforge_loader_config::paths::export_dir(root)),
        "updates" => Ok(shroudforge_loader_config::paths::updates_dir(root)),
        "ui" => Ok(shroudforge_loader_config::paths::ui_data_dir(root)),
        "runtime" => Ok(shroudforge_loader_config::paths::runtime_dir(root)),
        _ => Err("unsupported storage folder".into()),
    }
}

pub(crate) fn default_storage_path(root: &Path, key: &str) -> Result<PathBuf, String> {
    shroudforge_loader_config::paths::default_directory(root, key)
        .ok_or_else(|| "unsupported storage folder".into())
}

pub(crate) fn storage_paths(root: &Path) -> Vec<StoragePath> {
    [
        "mods", "state", "logs", "cache", "exports", "updates", "ui", "runtime",
    ]
    .into_iter()
    .filter_map(|id| {
        Some(StoragePath {
            id: id.to_owned(),
            path: storage_path(root, id).ok()?.display().to_string(),
            default_path: default_storage_path(root, id).ok()?.display().to_string(),
        })
    })
    .collect()
}
