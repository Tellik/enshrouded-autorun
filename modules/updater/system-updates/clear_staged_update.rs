use crate::check_update_cancelled::cancel_path;
use crate::write_update_status::write_status;
use std::{fs, path::Path};

pub(crate) fn clear_system_update(root: &Path) -> Result<(), String> {
    let updates = shroudforge_loader_config::paths::updates_dir(root);
    fs::create_dir_all(&updates).map_err(|error| error.to_string())?;
    fs::write(cancel_path(root), b"cancel").map_err(|error| error.to_string())?;
    // The worker owns staged files while the queue is running. It removes them
    // after observing this signal, once it has left the active operation.
    write_status(
        root,
        "cancelled",
        "Cancellation requested. Waiting for the updater to stop safely",
    );
    Ok(())
}
