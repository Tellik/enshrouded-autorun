#[cfg(windows)]
pub fn request_gamefiles_restore(root: &std::path::Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let status_path =
        shroudforge_loader_config::paths::backups_dir(root).join("gamefiles-restore-status.json");
    let status = crate::originals::originals_status(
        root,
        if root.join("enshrouded_server.exe").is_file() {
            "server"
        } else {
            "client"
        },
    );
    if status["status"] != "ready" {
        return Err("verified original game files are not available".into());
    }
    if let Ok(bytes) = std::fs::read(&status_path) {
        if let Ok(previous) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if matches!(
                previous["status"].as_str(),
                Some("queued" | "waitingForGame" | "restoring")
            ) {
                return Err("gamefile restoration is already in progress".into());
            }
        }
    }
    shroudforge_loader_config::config::write_json(
        &status_path,
        &serde_json::json!({"schemaVersion":1,"status":"queued","message":"Gamefile restoration queued"}),
    )?;
    let _ = std::fs::remove_file(
        shroudforge_loader_config::paths::updates_dir(root).join("cancel-system-update"),
    );
    let result =
        std::process::Command::new(shroudforge_loader_config::paths::updater_executable(root))
            .args(["--restore-gamefiles-worker", "--root"])
            .arg(root)
            .creation_flags(0x08000000)
            .spawn()
            .map_err(|error| {
                format!("could not start gamefile restoration in the updater: {error}")
            });
    if result.is_err() {
        let _ = std::fs::remove_file(status_path);
    }
    result.map(|_| ())
}

#[cfg(not(windows))]
pub fn request_gamefiles_restore(_: &std::path::Path) -> Result<(), String> {
    Err("gamefile restoration is available on Windows only".into())
}
