use crate::*;
#[cfg(windows)]
pub fn request_worker(root: &std::path::Path) -> Result<(), String> {
    scheduled::request(root)
}


#[cfg(not(windows))]
pub fn request_worker(_: &std::path::Path) -> Result<(), String> {
    Err("scheduled updater is available on Windows only".into())
}


#[cfg(windows)]
pub fn run_scheduled_worker() -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let args: Vec<String> = std::env::args().collect();
    let value = |name: &str| args.windows(2).find(|p| p[0] == name).map(|p| p[1].clone());
    let root = std::path::PathBuf::from(value("--root").ok_or("missing --root")?);
    let update_queue = read_update_queue_value(&root);
    if update_queue["run"]["state"] == "running" {
        return run_update_queue(&root);
    }
    let queue_path = shroudforge_loader_config::paths::updates_dir(&root).join("worker-queue.json");
    let mut first_error = None;
    let stage_request =
        shroudforge_loader_config::paths::updates_dir(&root).join("system-stage-request.json");
    if stage_request.is_file() {
        let result = (|| {
            let bytes = std::fs::read(&stage_request)
                .map_err(|e| format!("cannot read system stage request: {e}"))?;
            let request: serde_json::Value = serde_json::from_slice(&bytes)
                .map_err(|e| format!("invalid system stage request: {e}"))?;
            if request["waitForGame"]
                .as_bool()
                .unwrap_or_else(|| request["waitPid"].as_u64().unwrap_or_default() != 0)
            {
                scheduled::write_status(
                    &root,
                    "waitingForGame",
                    "Waiting for Enshrouded to close before preparing the update",
                );
                scheduled::wait_for_game_processes(&root)?;
            }
            scheduled::open_update_window(&root)?;
            scheduled::write_status(
                &root,
                "downloading",
                "Downloading and verifying the system update",
            );
            scheduled::stage_system_update(&root, &request)
        })();
        match &result {
            Ok(()) => scheduled::write_status(
                &root,
                "staged",
                "Verified and staged. Installation waits for game exit",
            ),
            Err(error) if error == "UPDATE_CANCELLED" => {
                let updates = shroudforge_loader_config::paths::updates_dir(&root);
                let _ = std::fs::remove_file(updates.join("download.zip"));
                let _ = std::fs::remove_dir_all(updates.join("pending-download"));
                let _ = std::fs::remove_dir_all(updates.join("pending"));
                let _ = std::fs::remove_file(updates.join("pending.ready"));
                scheduled::write_status(
                    &root,
                    "cancelled",
                    "Update download cancelled. Queue cleared",
                );
            }
            Err(error) => {
                scheduled::write_status(&root, "error", error);
                let _ = shroudforge_logging::append(
                    &root,
                    'E',
                    "updater",
                    &format!("System update download failed: {error}"),
                );
                first_error = Some(error.clone());
            }
        }
        let _ = std::fs::remove_file(&stage_request);
        let _ = std::fs::remove_file(
            shroudforge_loader_config::paths::updates_dir(&root).join("cancel-system-update"),
        );
    }
    // Catalog mod packages are independent of the ShroudForge binary release.
    // Complete these while the game may still be running, before a full-package
    // installer can wait for the game to exit and replace the loader executable.
    let mod_queue = shroudforge_loader_config::paths::updates_dir(&root).join("mod-install-queue.json");
    if mod_queue.is_file() {
        let result =
            std::process::Command::new(shroudforge_loader_config::paths::loader_executable(&root))
                .args(["--catalog-install-worker", "--root"])
                .arg(&root)
                .creation_flags(0x08000000)
                .status()
                .map_err(|e| format!("could not launch catalog install worker: {e}"))
                .and_then(|status| {
                    if status.success() {
                        Ok(())
                    } else {
                        Err(format!("catalog install worker returned {status}"))
                    }
                });
        if let Err(ref error) = result {
            let _ = shroudforge_logging::append(
                &root,
                'E',
                "updater",
                &format!("Catalog install failed: {error}"),
            );
            if first_error.is_none() {
                first_error = Some(error.clone());
            }
        }
    }
    if queue_path.is_file() {
        let result = (|| {
            let bytes = std::fs::read(&queue_path)
                .map_err(|e| format!("cannot read updater queue: {e}"))?;
            let queue: serde_json::Value = serde_json::from_slice(&bytes)
                .map_err(|e| format!("invalid updater queue: {e}"))?;
            if queue["operation"] != "installPending" {
                return Err("unsupported updater operation".into());
            }
            let staged = shroudforge_loader_config::paths::updates_dir(&root).join("pending");
            let status =
                std::process::Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
                    .args(["--update-worker", "--root"])
                    .arg(&root)
                    .arg("--staged")
                    .arg(&staged)
                    .status()
                    .map_err(|e| format!("could not launch validated installer: {e}"))?;
            if status.success() {
                Ok(())
            } else {
                Err(format!("installer returned {status}"))
            }
        })();
        if let Err(ref error) = result {
            scheduled::write_status(&root, "error", error);
            let _ = shroudforge_logging::append(
                &root,
                'E',
                "updater",
                &format!("Scheduled system updater failed: {error}"),
            );
            first_error = Some(error.clone());
        }
        let _ = std::fs::remove_file(&queue_path);
    }
    first_error.map_or(Ok(()), Err)
}


#[cfg(windows)]
pub fn run_module() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(root) = args
        .windows(2)
        .find(|pair| pair[0] == "--root")
        .map(|pair| std::path::PathBuf::from(&pair[1]))
        && windows::hand_off_installed_worker(&root)?
    {
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--restore-gamefiles-worker") {
        let root = args
            .windows(2)
            .find(|pair| pair[0] == "--root")
            .map(|pair| std::path::PathBuf::from(&pair[1]))
            .ok_or("missing --root")?;
        return windows::restore_gamefiles_worker(&root);
    }
    if args.iter().any(|a| a == "--run-queue") {
        return run_scheduled_worker();
    }
    if args.iter().any(|a| a == "--open-window") {
        let root = args
            .windows(2)
            .find(|pair| pair[0] == "--root")
            .map(|pair| std::path::PathBuf::from(&pair[1]))
            .ok_or("missing --root")?;
        return scheduled::open_update_window(&root).map_err(|error| {
            let _ = shroudforge_logging::append(&root, 'E', "updater-window", &error);
            error
        });
    }
    if args.len() == 1 {
        let executable = std::env::current_exe().map_err(|error| error.to_string())?;
        let module_directory = executable
            .parent()
            .ok_or("updater executable has no module directory")?;
        let root = module_directory
            .parent()
            .ok_or("updater executable is not inside the ShroudForge module directory")?
            .to_path_buf();
        match crate::headless_check_system_updates(
            &root,
            crate::update_check::SystemUpdateCheckMode::Manual,
        ) {
            Ok(message) => {
                let _ = shroudforge_logging::append(&root, 'I', "updater", &message);
            }
            Err(error) => {
                let _ = shroudforge_logging::append(
                    &root,
                    'W',
                    "updater",
                    &format!("Manual update check did not complete: {error}"),
                );
            }
        }
        return scheduled::open_update_window(&root).map_err(|error| {
            let _ = shroudforge_logging::append(&root, 'E', "updater-window", &error);
            error
        });
    }
    windows::run()
}


#[cfg(not(windows))]
pub fn run_module() -> Result<(), String> {
    Err("ShroudForge updater is available on Windows only".into())
}
