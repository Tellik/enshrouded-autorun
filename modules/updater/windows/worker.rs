use std::{fs, path::Path};
use crate::system_install::run_install;
use crate::read_install_arguments::Arguments;
pub(crate) use crate::restore_worker::*;
pub fn run() -> Result<(), String> {
    let result = run_install();
    if let Err(error) = &result {
        if let Ok(arguments) = Arguments::read() {
            if error == "UPDATE_CANCELLED" {
                super::scheduled::write_status(
                    &arguments.root,
                    "cancelled",
                    "Update download cancelled. Queue cleared",
                );
            } else {
                super::scheduled::write_status(&arguments.root, "error", error);
                append_log(
                    &arguments.root,
                    'E',
                    &format!("Update worker failed before completion: {error}"),
                );
            }
        }
    }
    result
}


pub(crate) fn hand_off_installed_worker(root: &Path) -> Result<bool, String> {
    use std::os::windows::process::CommandExt;

    let args = std::env::args_os().collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--ephemeral-worker") {
        return Ok(false);
    }
    if !args
        .iter()
        .any(|arg| arg == "--update-worker" || arg == "--run-queue")
    {
        return Ok(false);
    }
    let current = std::env::current_exe().map_err(|error| error.to_string())?;
    let installed = shroudforge_loader_config::paths::updater_executable(root);
    let current = current
        .canonicalize()
        .map_err(|error| format!("could not resolve updater executable: {error}"))?;
    let installed = installed
        .canonicalize()
        .map_err(|error| format!("could not resolve installed updater: {error}"))?;
    if current != installed {
        return Ok(false);
    }

    // Windows locks a running executable. Run the installer from the protected
    // updates directory so the child can replace shroudforge-updater.exe too.
    let worker = shroudforge_loader_config::paths::updates_dir(root).join("updater-worker.exe");
    fs::create_dir_all(worker.parent().ok_or("worker path has no parent")?)
        .map_err(|error| error.to_string())?;
    fs::copy(&current, &worker)
        .map_err(|error| format!("could not prepare independent updater worker: {error}"))?;
    std::process::Command::new(&worker)
        .args(args.into_iter().skip(1))
        .arg("--ephemeral-worker")
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|error| format!("could not start independent updater worker: {error}"))?;
    Ok(true)
}


pub(crate) fn append_log(root: &Path, level: char, message: &str) {
    let _ = shroudforge_logging::append(root, level, "updater", message);
}
