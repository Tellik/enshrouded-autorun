use std::path::{Path, PathBuf};

pub fn directory(root: &Path) -> PathBuf {
    crate::paths::config_dir(root).join("news")
}
pub fn events_directory(root: &Path) -> PathBuf {
    directory(root).join("events")
}
