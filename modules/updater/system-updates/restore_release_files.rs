use crate::apply_release_files::copy_entry;
use crate::remove_release_files::remove_entry;
use crate::validate_update_paths::installed_path;
use std::path::Path;

pub(crate) fn restore(backup: &Path, target: &Path, paths: &[String]) -> Result<(), String> {
    for relative in paths {
        let saved = backup.join(relative);
        let current = installed_path(target, relative);
        remove_entry(&current)
            .map_err(|error| format!("could not remove {relative} during rollback: {error}"))?;
        if saved.exists() {
            copy_entry(&saved, &current).map_err(|error| {
                format!("could not restore {relative} during rollback: {error}")
            })?;
        }
    }
    Ok(())
}
