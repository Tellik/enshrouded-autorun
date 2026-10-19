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

pub(crate) fn hand_off_installed_process(root: &Path) -> Result<bool, String> {
    use std::os::windows::process::CommandExt;

    let args = std::env::args_os().collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--ephemeral-worker") {
        return Ok(false);
    }
    let managed_mode = args.iter().any(|arg| {
        matches!(
            arg.to_str(),
            Some("--update-worker" | "--run-queue" | "--open-window")
        )
    });
    let direct_window = args.len() == 1;
    if !managed_mode && !direct_window {
        return Ok(false);
    }
    if !crate::ui::host::window::is_game_root(root) {
        return Ok(false);
    }
    let current = std::env::current_exe().map_err(|error| error.to_string())?;
    let installed = shroudforge_loader_config::paths::updater_executable(root);
    if !installed.is_file() {
        return Ok(false);
    }
    let current = current
        .canonicalize()
        .map_err(|error| format!("could not resolve updater executable: {error}"))?;
    let installed = installed
        .canonicalize()
        .map_err(|error| format!("could not resolve installed updater: {error}"))?;
    if current != installed {
        return Ok(false);
    }

    // Windows locks a running executable. Run windows and workers from a
    // unique copy outside managed files so the installed updater can be replaced.
    let worker = prepare_independent_worker(root, &current)?;
    let mut command = std::process::Command::new(&worker);
    if direct_window {
        command.args(["--open-window", "--root"]).arg(root);
    } else {
        command.args(args.into_iter().skip(1));
    }
    if let Err(error) = command
        .arg("--ephemeral-worker")
        .creation_flags(0x08000000)
        .spawn()
    {
        let _ = fs::remove_file(&worker);
        return Err(format!(
            "could not start independent updater worker: {error}"
        ));
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
            return Err(format!(
                "could not start independent updater worker: {error}"
            ));
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

pub(crate) fn schedule_shroudforge_data_purge(root: &Path) -> Result<(), String> {
    use std::os::windows::fs::MetadataExt;
    use std::os::windows::process::CommandExt;

    let root = root
        .canonicalize()
        .map_err(|error| format!("could not resolve the Enshrouded folder: {error}"))?;
    let data = shroudforge_loader_config::paths::data_dir(&root);
    let temp = std::env::temp_dir()
        .canonicalize()
        .map_err(|error| format!("could not resolve the Windows temporary folder: {error}"))?;
    match fs::symlink_metadata(&data) {
        Ok(metadata)
            if !metadata.is_dir()
                || metadata.file_type().is_symlink()
                || metadata.file_attributes() & 0x400 != 0 =>
        {
            return Err(format!(
                "refusing to purge a non-folder or reparse point: {}",
                data.display()
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "could not inspect the ShroudForge data folder {}: {error}",
                data.display()
            ));
        }
    }
    if path_contains_path(&data, &temp) {
        return Err(format!(
            "Windows temporary folder is inside the ShroudForge data folder: {}",
            temp.display()
        ));
    }

    let current = std::env::current_exe().map_err(|error| error.to_string())?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let worker = temp.join(format!(
        "ShroudForge-Purge-{}-{stamp}.exe",
        std::process::id()
    ));
    let expected = fs::metadata(&current)
        .map_err(|error| format!("could not inspect the updater executable: {error}"))?
        .len();
    let copied = fs::copy(&current, &worker)
        .map_err(|error| format!("could not prepare the ShroudForge purge worker: {error}"))?;
    if copied != expected || !fs::metadata(&worker).is_ok_and(|metadata| metadata.len() == expected)
    {
        let _ = fs::remove_file(&worker);
        return Err("ShroudForge purge worker copy has an unexpected size".into());
    }

    let wait_pid = std::process::id().to_string();
    if let Err(error) = std::process::Command::new(&worker)
        .args(["--purge-shroudforge-data", "--root"])
        .arg(root)
        .arg("--wait-pid")
        .arg(wait_pid)
        .arg("--ephemeral-worker")
        .creation_flags(0x08000000)
        .spawn()
    {
        let _ = fs::remove_file(&worker);
        return Err(format!(
            "could not start the ShroudForge purge worker: {error}"
        ));
    }
    Ok(())
}

pub(crate) fn run_shroudforge_data_purge_worker(
    root: &Path,
    wait_pid: u32,
    executable: &Path,
) -> Result<(), String> {
    use std::os::windows::fs::MetadataExt;

    let result = (|| {
        let root = root
            .canonicalize()
            .map_err(|error| format!("could not resolve the Enshrouded folder: {error}"))?;
        if !crate::ui::host::window::is_game_root(&root) {
            return Err(format!(
                "refusing to purge ShroudForge data outside an Enshrouded folder: {}",
                root.display()
            ));
        }
        if wait_pid == 0 || wait_pid == std::process::id() {
            return Err("ShroudForge purge worker received an invalid parent process ID".into());
        }
        wait_for_process_exit(wait_pid)?;

        let data = shroudforge_loader_config::paths::data_dir(&root);
        let metadata = match fs::symlink_metadata(&data) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                return Err(format!(
                    "could not inspect the ShroudForge data folder {}: {error}",
                    data.display()
                ));
            }
        };
        if !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || metadata.file_attributes() & 0x400 != 0
        {
            return Err(format!(
                "refusing to recursively remove a non-folder or reparse point: {}",
                data.display()
            ));
        }
        crate::scheduled::prepare_shroudforge_data_purge_locks(&root)?;
        let _ = shroudforge_logging::append(
            &root,
            'I',
            "updater",
            "Purging the complete shroudforge data folder recursively",
        );
        fs::remove_dir_all(&data).map_err(|error| {
            format!(
                "could not recursively remove the ShroudForge data folder {}: {error}",
                data.display()
            )
        })
    })();
    if let Err(error) = &result {
        let _ = shroudforge_logging::append(
            root,
            'E',
            "updater",
            &format!("ShroudForge data purge failed: {error}"),
        );
    }
    schedule_delete_after_reboot(executable);
    result
}

fn path_contains_path(parent: &Path, child: &Path) -> bool {
    let normalize = |path: &Path| {
        let mut value = path.to_string_lossy().replace('/', "\\").to_lowercase();
        while value.len() > 3 && value.ends_with('\\') {
            value.pop();
        }
        value
    };
    let parent = normalize(parent);
    let child = normalize(child);
    if parent == child {
        return true;
    }
    child.starts_with(&format!("{}\\", parent.trim_end_matches('\\')))
}

fn wait_for_process_exit(pid: u32) -> Result<(), String> {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, ERROR_INVALID_PARAMETER, GetLastError},
        System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject},
    };

    let process = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, pid) };
    if process.is_null() {
        let error = unsafe { GetLastError() };
        return if error == ERROR_INVALID_PARAMETER {
            Ok(())
        } else {
            Err(format!(
                "cannot wait for the ShroudForge updater process {pid} (Windows error {error})"
            ))
        };
    }
    loop {
        match unsafe { WaitForSingleObject(process, 1_000) } {
            0 => break,
            0x0000_0102 => {}
            _ => {
                let error = unsafe { GetLastError() };
                unsafe { CloseHandle(process) };
                return Err(format!(
                    "could not wait for the ShroudForge updater process {pid} (Windows error {error})"
                ));
            }
        }
    }
    unsafe { CloseHandle(process) };
    Ok(())
}

fn schedule_delete_after_reboot(path: &Path) {
    use std::{os::windows::ffi::OsStrExt, ptr};
    use windows_sys::Win32::Storage::FileSystem::MoveFileExW;

    let path = path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    unsafe {
        MoveFileExW(path.as_ptr(), ptr::null(), 0x0000_0004);
    }
}

pub(crate) fn append_log(root: &Path, level: char, message: &str) {
    let _ = shroudforge_logging::append(root, level, "updater", message);
}

pub(crate) fn prepare_independent_worker(
    root: &Path,
    current: &Path,
) -> Result<std::path::PathBuf, String> {
    let updates = shroudforge_loader_config::paths::updates_dir(root);
    fs::create_dir_all(&updates).map_err(|error| {
        format!(
            "could not prepare the updater worker directory {}: {error}",
            updates.display()
        )
    })?;
    cleanup_stale_workers(&updates);
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let worker = updates.join(format!("updater-worker-{}-{stamp}.exe", std::process::id()));
    let started = std::time::Instant::now();
    loop {
        match fs::copy(current, &worker) {
            Ok(copied) => {
                let expected = fs::metadata(current)
                    .map_err(|error| format!("could not inspect updater executable: {error}"))?
                    .len();
                if copied == expected
                    && fs::metadata(&worker).is_ok_and(|metadata| metadata.len() == expected)
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
                return Err(format!(
                    "could not prepare independent updater worker: {error}"
                ));
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
