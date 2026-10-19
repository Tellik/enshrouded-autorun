use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

pub(crate) fn cancel_path(root: &Path) -> PathBuf {
    shroudforge_loader_config::paths::updates_dir(root).join("cancel-system-update")
}

pub(crate) fn check_update_cancelled(root: &Path) -> Result<(), String> {
    let path = cancel_path(root);
    match fs::symlink_metadata(&path) {
        Ok(_) => Err("UPDATE_CANCELLED".into()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "could not inspect update cancellation request {}: {error}",
            path.display()
        )),
    }
}
