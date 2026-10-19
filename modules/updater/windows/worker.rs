use crate::read_install_arguments::Arguments;
pub(crate) use crate::restore_worker::*;
use crate::system_install::run_install;
use std::{fs, io::Write, path::Path, process::Stdio};

const STALE_WORKER_AGE: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);

pub fn run() -> Result<(), String> {
    let result = run_install();
    if let Err(error) = &result
        && let Ok(arguments) = Arguments::read()
    {
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

    // Windows locks a running executable. Use a unique copy outside managed
    // files so simultaneous starts cannot overwrite one another's worker.
    let worker = prepare_independent_worker(root, &current)?;
    if let Err(error) = std::process::Command::new(&worker)
        .args(args.into_iter().skip(1))
        .arg("--ephemeral-worker")
        .creation_flags(0x08000000)
        .spawn()
    {
        let _ = fs::remove_file(&worker);
        return Err(format!("could not start independent updater worker: {error}"));
    }
    Ok(true)
}

pub(crate) fn hand_off_request_worker(
    root: &Path,
    request: &serde_json::Value,
    input: &[u8],
) -> Result<bool, String> {
    use std::os::windows::process::CommandExt;

    if request["command"] != "recycle-bin-restore"
        || std::env::args_os().any(|arg| arg == "--ephemeral-worker")
        || crate::read_update_queue(root)["run"]["state"] == "running"
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

    // Restore can replace the updater itself. Run it from a unique copy in the
    // updates folder, then let it wait for this short-lived request process.
    let worker = prepare_independent_worker(root, &current)?;
    let mut child = match std::process::Command::new(&worker)
        .args(["--request", "--root"])
        .arg(root)
        .arg("--ephemeral-worker")
        .arg("--wait-pid")
        .arg(std::process::id().to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .creation_flags(0x08000000)
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            let _ = fs::remove_file(&worker);
            return Err(format!("could not start independent updater worker: {error}"));
        }
    };
    let Some(mut stdin) = child.stdin.take() else {
        let _ = child.kill();
        let _ = child.wait();
        let _ = fs::remove_file(&worker);
        return Err("updater worker request stdin is unavailable".into());
    };
    if let Err(error) = stdin.write_all(input) {
        drop(stdin);
        let _ = child.kill();
        let _ = child.wait();
        let _ = fs::remove_file(&worker);
        return Err(format!("could not send request to updater worker: {error}"));
    }
    drop(stdin);
    Ok(true)
}

pub(crate) fn append_log(root: &Path, level: char, message: &str) {
    let _ = shroudforge_logging::append(root, level, "updater", message);
}

fn prepare_independent_worker(root: &Path, current: &Path) -> Result<std::path::PathBuf, String> {
    let updates = shroudforge_loader_config::paths::updates_dir(root);
    fs::create_dir_all(&updates).map_err(|error| {
        format!("could not prepare the updater worker directory {}: {error}", updates.display())
    })?;
    cleanup_stale_workers(&updates);
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let worker = updates.join(format!(
        "updater-worker-{}-{stamp}.exe",
        std::process::id()
    ));
    let started = std::time::Instant::now();
    loop {
        match fs::copy(current, &worker) {
            Ok(copied) => {
                let expected = fs::metadata(current)
                    .map_err(|error| format!("could not inspect updater executable: {error}"))?
                    .len();
                if copied == expected
                    && fs::metadata(&worker)
                        .is_ok_and(|metadata| metadata.len() == expected)
                {
                    return Ok(worker);
                }
                let _ = fs::remove_file(&worker);
                return Err("independent updater worker copy has an unexpected size".into());
            }
            Err(error) if started.elapsed() < std::time::Duration::from_secs(15) => {
                std::thread::sleep(std::time::Duration::from_millis(250));
                if started.elapsed() >= std::time::Duration::from_secs(15) {
                    let _ = fs::remove_file(&worker);
                    return Err(format!(
                        "could not prepare independent updater worker after 15 seconds: {error}"
                    ));
                }
            }
            Err(error) => {
                let _ = fs::remove_file(&worker);
                return Err(format!("could not prepare independent updater worker: {error}"));
            }
        }
    }
}

fn cleanup_stale_workers(updates: &Path) {
    let Ok(entries) = fs::read_dir(updates) else {
        return;
    };
    let now = std::time::SystemTime::now();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(worker_name) = name
            .to_str()
            .and_then(|name| name.strip_prefix("updater-worker-"))
            .and_then(|name| name.strip_suffix(".exe"))
        else {
            continue;
        };
        let Some((pid, timestamp)) = worker_name.split_once('-') else {
            continue;
        };
        let Ok(pid) = pid.parse::<u32>() else {
            continue;
        };
        if !timestamp.bytes().all(|byte| byte.is_ascii_digit()) {
            continue;
        }
        let Ok(metadata) = fs::symlink_metadata(entry.path()) else {
            continue;
        };
        if !metadata.is_file()
            || metadata
                .modified()
                .ok()
                .and_then(|modified| now.duration_since(modified).ok())
                .is_none_or(|age| age < STALE_WORKER_AGE)
            || crate::scheduled::process_is_running(pid).unwrap_or(true)
        {
            continue;
        }
        let _ = fs::remove_file(entry.path());
    }
}
