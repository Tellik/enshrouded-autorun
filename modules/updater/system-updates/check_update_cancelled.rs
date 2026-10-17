use std::path::{Path, PathBuf};

pub(crate) fn cancel_path(root: &Path) -> PathBuf {
    shroudforge_loader_config::paths::updates_dir(root).join("cancel-system-update")
}

pub(crate) fn check_update_cancelled(root: &Path) -> Result<(), String> {
    if cancel_path(root).exists() {
        Err("UPDATE_CANCELLED".into())
    } else {
        Ok(())
    }
}
