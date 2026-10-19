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
    let signal = match fs::read_to_string(&path) {
        Ok(signal) => signal,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "could not inspect update cancellation request {}: {error}",
                path.display()
            ));
        }
    };
    let signals = signal.lines().map(str::trim).collect::<Vec<_>>();
    if signals.is_empty() || signals.iter().any(|signal| *signal == "cancel") {
        return Err("UPDATE_CANCELLED".into());
    }
    let requested_tokens = signals
        .iter()
        .map(|signal| signal.strip_prefix("run:"))
        .collect::<Option<Vec<_>>>();
    let Some(requested_tokens) = requested_tokens else {
        return Err("UPDATE_CANCELLED".into());
    };
    if requested_tokens.iter().any(|token| token.is_empty()) {
        return Err("UPDATE_CANCELLED".into());
    }
    let queue = match crate::update_queue_storage::try_read_update_queue_value(root) {
        Ok(queue) => queue,
        Err(_) => return Err("UPDATE_CANCELLED".into()),
    };
    if queue["run"]["state"] != "running" {
        return Ok(());
    }
    let Some(active_token) = crate::update_queue_storage::update_queue_run_token(&queue["run"])
    else {
        return Err("UPDATE_CANCELLED".into());
    };
    if requested_tokens
        .iter()
        .any(|requested_token| *requested_token == active_token)
    {
        Err("UPDATE_CANCELLED".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_signal_only_cancels_the_matching_queue_run() {
        let root = tempfile::tempdir().unwrap();
        let updates = shroudforge_loader_config::paths::updates_dir(root.path());
        std::fs::create_dir_all(&updates).unwrap();
        shroudforge_loader_config::config::write_json(
            &updates.join("update-queue.json"),
            &serde_json::json!({
                "schemaVersion": 1,
                "items": [],
                "run": {
                    "state": "running",
                    "runId": "new-run",
                    "itemIds": ["system:new"]
                }
            }),
        )
        .unwrap();

        std::fs::write(cancel_path(root.path()), "run:old-run\n").unwrap();
        assert!(check_update_cancelled(root.path()).is_ok());

        let mut signal = std::fs::OpenOptions::new()
            .append(true)
            .open(cancel_path(root.path()))
            .unwrap();
        use std::io::Write;
        writeln!(signal, "run:new-run").unwrap();
        assert_eq!(
            check_update_cancelled(root.path()).unwrap_err(),
            "UPDATE_CANCELLED"
        );
    }

    #[test]
    fn legacy_cancellation_signal_still_cancels_the_active_queue() {
        let root = tempfile::tempdir().unwrap();
        let updates = shroudforge_loader_config::paths::updates_dir(root.path());
        std::fs::create_dir_all(&updates).unwrap();
        std::fs::write(cancel_path(root.path()), "cancel").unwrap();

        assert_eq!(
            check_update_cancelled(root.path()).unwrap_err(),
            "UPDATE_CANCELLED"
        );
    }
}
