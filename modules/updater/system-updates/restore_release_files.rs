use crate::apply_release_files::copy_file_with_progress;
use crate::remove_release_files::remove_entry;
use crate::validate_update_paths::installed_path;
use std::path::Path;

pub(crate) fn restore_with_progress(
    backup: &Path,
    target: &Path,
    paths: &[String],
    mut progress: impl FnMut(usize, usize, &str, u64, u64) -> Result<(), String>,
) -> Result<(), String> {
    for (index, relative) in paths.iter().enumerate() {
        let saved = backup.join(relative);
        let current = installed_path(target, relative);
        remove_entry(&current)
            .map_err(|error| format!("could not remove {relative} during rollback: {error}"))?;
        if saved.is_file() {
            copy_file_with_progress(&saved, &current, |bytes, total| {
                progress(index, paths.len(), relative, bytes, total)
            })
            .map_err(|error| format!("could not restore {relative} during rollback: {error}"))?;
            progress(index + 1, paths.len(), "", 0, 0)?;
        } else {
            progress(index + 1, paths.len(), "", 0, 0)?;
        }
    }
    Ok(())
}
