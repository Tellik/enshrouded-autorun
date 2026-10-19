#[cfg(windows)]
use crate::scheduled;
#[cfg(windows)]
use crate::update_queue_storage::update_update_queue;

#[cfg(windows)]
pub fn cancel_update_queue(root: &std::path::Path) -> Result<(), String> {
    let queue = match crate::try_read_update_queue_value(root) {
        Ok(queue) if queue["run"]["state"] == "running" => queue,
        Ok(_) => return Err("there is no running update queue to cancel".into()),
        Err(_) => {
            scheduled::clear_system_update(root, None)?;
            return Ok(());
        }
    };
    let run_token = crate::update_queue_storage::update_queue_run_token(&queue["run"]);
    // Signal first so the worker can stop a long operation even while it owns
    // the shared installation lock used to persist queue state.
    scheduled::clear_system_update(root, run_token.as_deref())?;
    if let Some(run_token) = run_token {
        update_update_queue(root, |queue| {
            if queue["run"]["state"] == "running"
                && crate::update_queue_storage::update_queue_run_token(&queue["run"]).as_deref()
                    == Some(run_token.as_str())
            {
                queue["run"]["cancelRequested"] = serde_json::Value::Bool(true);
            }
            Ok(())
        })?;
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn cancel_update_queue(_: &std::path::Path) -> Result<(), String> {
    Err("update queue is available on Windows only".into())
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn cancel_action_signals_the_active_queue_run() {
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
                    "runId": "active-run",
                    "itemIds": ["system:test"],
                    "cancelRequested": false
                }
            }),
        )
        .unwrap();

        cancel_update_queue(root.path()).unwrap();

        assert!(updates.join("cancel-system-update").is_file());
        assert_eq!(
            crate::check_update_cancelled::check_update_cancelled(root.path()).unwrap_err(),
            "UPDATE_CANCELLED"
        );
    }

    #[test]
    fn cancel_action_on_a_corrupt_queue_still_signals_cancellation() {
        let root = tempfile::tempdir().unwrap();
        let updates = shroudforge_loader_config::paths::updates_dir(root.path());
        std::fs::create_dir_all(&updates).unwrap();
        std::fs::write(updates.join("update-queue.json"), b"{").unwrap();

        cancel_update_queue(root.path()).unwrap();

        assert!(updates.join("cancel-system-update").is_file());
        assert_eq!(
            crate::check_update_cancelled::check_update_cancelled(root.path()).unwrap_err(),
            "UPDATE_CANCELLED"
        );
    }
}
