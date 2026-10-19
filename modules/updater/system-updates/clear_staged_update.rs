use crate::check_update_cancelled::cancel_path;
use crate::write_update_status::write_status;
use std::{fs, io::Write, path::Path};

pub(crate) fn clear_system_update(root: &Path, run_token: Option<&str>) -> Result<(), String> {
    let updates = shroudforge_loader_config::paths::updates_dir(root);
    fs::create_dir_all(&updates).map_err(|error| error.to_string())?;
    let signal = run_token
        .map(|token| format!("run:{token}"))
        .unwrap_or_else(|| "cancel".to_owned());
    let path = cancel_path(root);
    let (marker_exists, existing) = match fs::read_to_string(&path) {
        Ok(existing) => (true, existing),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (false, String::new()),
        Err(error) => return Err(error.to_string()),
    };
    let already_cancelled = marker_exists
        && (existing.trim().is_empty()
            || existing.lines().any(|line| {
                let line = line.trim();
                line == "cancel" || line == signal
            }));
    if !already_cancelled {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|error| error.to_string())?;
        writeln!(file, "{signal}").map_err(|error| error.to_string())?;
    }
    // The worker owns staged files while the queue is running. It removes them
    // after observing this signal, once it has left the active operation.
    write_status(
        root,
        "cancelled",
        "Cancellation requested. Waiting for the updater to stop safely",
    );
    Ok(())
}
