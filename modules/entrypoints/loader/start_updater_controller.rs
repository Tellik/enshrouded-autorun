use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};

pub(super) fn start(root: &Path) -> Result<(), String> {
    static STARTED_ROOTS: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();
    let root = root
        .canonicalize()
        .map_err(|error| format!("could not resolve game root for updater controller: {error}"))?;
    let updates_enabled = shroudforge_loader_config::config::read_loader(&root)
        .ok()
        .and_then(|config| {
            config
                .pointer("/modules/updates/enabled")
                .and_then(serde_json::Value::as_bool)
        })
        .unwrap_or(false);
    if !updates_enabled {
        return Ok(());
    }
    let mut started = STARTED_ROOTS
        .get_or_init(Default::default)
        .lock()
        .map_err(|_| "updater controller registry is unavailable")?;
    if started.contains(&root) {
        return Ok(());
    }
    let executable = shroudforge_loader_config::paths::updater_executable(&root);
    if !executable.is_file() {
        let _ = shroudforge_loader_config::config::initialize_loader_config(&root);
        return Ok(());
    }
    let mut command = Command::new(&executable);
    command
        .arg("--controller")
        .arg("--root")
        .arg(&root)
        .arg("--parent-pid")
        .arg(std::process::id().to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    match command.spawn() {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let _ = shroudforge_loader_config::config::initialize_loader_config(&root);
            return Ok(());
        }
        Err(error) => {
            return Err(format!(
                "could not start updater controller {}: {error}",
                executable.display()
            ));
        }
    }
    started.insert(root);
    Ok(())
}
