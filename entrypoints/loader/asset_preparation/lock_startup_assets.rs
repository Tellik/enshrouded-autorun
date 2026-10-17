use crate::LoaderError;
use std::path::Path;

#[cfg(windows)]
pub(super) fn startup_asset_lease(path: &Path) -> Result<std::fs::File, LoaderError> {
    use std::os::windows::fs::OpenOptionsExt;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| LoaderError::Pregame(error.to_string()))?;
    }
    std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .share_mode(0)
        .open(path)
        .map_err(|error| {
            LoaderError::Pregame(format!(
                "asset startup is already active or locked: {error}"
            ))
        })
}

#[cfg(not(windows))]
pub(super) fn startup_asset_lease(path: &Path) -> Result<std::fs::File, LoaderError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| LoaderError::Pregame(error.to_string()))?;
    }
    std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)
        .map_err(|error| LoaderError::Pregame(error.to_string()))
}
