use crate::windows::*;
use std::path::Path;
pub(crate) fn restore_gamefiles_worker(root: &Path) -> Result<(), String> {
    let status_path =
        shroudforge_loader_config::paths::backups_dir(root).join("gamefiles-restore-status.json");
    let write_status = |status: &str, message: &str| {
        shroudforge_loader_config::config::write_json(
            &status_path,
            &serde_json::json!({"schemaVersion":1,"status":status,"message":message,
                "updatedAt":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()}),
        )
    };
    let target = if root.join("enshrouded_server.exe").is_file() {
        "server"
    } else if root.join("enshrouded.exe").is_file() {
        "client"
    } else {
        return Err("could not identify the Enshrouded installation target".into());
    };
    let result: Result<(), String> = (|| {
        let _operation_lock = crate::wait_for_queue_worker_mutex(root)?;
        let queue = crate::update_queue_storage::read_update_queue_value(root);
        if let Some(error) = queue["error"].as_str() {
            return Err(error.to_owned());
        }
        if queue["run"]["state"] == "running" {
            return Err("game files cannot be restored while an update is running".into());
        }
        write_status("waitingForGame", "Waiting for Enshrouded to close")?;
        append_log(
            root,
            'I',
            "Gamefile restoration is waiting for the game process to exit",
        );
        super::scheduled::wait_for_game_processes(root)?;
        write_status("restoring", "Verifying and restoring original game files")?;
        let restored = shroudforge_game_assets::originals::restore_originals(root, target)?;
        shroudforge_loader_config::config::write_json(
            &status_path,
            &serde_json::json!({"schemaVersion":1,"status":"restart-required",
                "message":"Original game files restored. Restart Enshrouded",
                "target":target,"result":restored,
                "updatedAt":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()}),
        )?;
        append_log(
            root,
            'I',
            "Original game files restored and verified. Restart required",
        );
        Ok(())
    })();
    if let Err(error) = &result {
        let _ = write_status("error", error);
        append_log(root, 'E', &format!("Gamefile restoration failed: {error}"));
    }
    result
}
