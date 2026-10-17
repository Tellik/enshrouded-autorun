use crate::check_update_cancelled::cancel_path;
use crate::write_update_status::write_status;
use std::{fs, path::Path};

pub(crate) fn clear_system_update(root: &Path) -> Result<(), String> {
    let updates = shroudforge_loader_config::paths::updates_dir(root);
    fs::create_dir_all(&updates).map_err(|error| error.to_string())?;
    fs::write(cancel_path(root), b"cancel").map_err(|error| error.to_string())?;
    for path in [
        updates.join("system-stage-request.json"),
        updates.join("worker-queue.json"),
        updates.join("pending.ready"),
        updates.join("download.zip"),
    ] {
        let _ = fs::remove_file(path);
    }
    for path in [updates.join("pending-download"), updates.join("pending")] {
        if path.exists() {
            let _ = fs::remove_dir_all(path);
        }
    }
    write_status(
        root,
        "cancelled",
        "Update download cancelled. Queue cleared",
    );
    Ok(())
}
