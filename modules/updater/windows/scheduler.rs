use std::{fs,path::{Path,PathBuf},process::Command};
pub(crate) use crate::system_stage::stage_system_update;
pub(crate) use crate::write_update_status::{write_status, write_status_with_version};
pub(crate) use crate::check_update_cancelled::check_update_cancelled;
pub(crate) use crate::clear_staged_update::clear_system_update;
use windows_sys::Win32::{
    Foundation::{CloseHandle, ERROR_INVALID_PARAMETER, GetLastError},
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
            TH32CS_SNAPPROCESS,
        },
        Threading::{
            OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
        },
    },
};

pub(crate) fn wait_for_game_processes(root: &Path) -> Result<(), String> {
    use std::{mem::size_of, time::Duration};
    let root = root
        .canonicalize()
        .map_err(|e| format!("invalid install root for process scan: {e}"))?;
    let root_key = root.to_string_lossy().to_lowercase();
    let mut last_waiting_pid = None;
    loop {
        check_update_cancelled(&root)?;
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        if snapshot == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
            return Err(format!(
                "could not enumerate game processes (Windows error {})",
                unsafe { GetLastError() }
            ));
        }
        let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let mut running: Option<(u32, String)> = None;
        let mut has_entry = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
        while has_entry {
            let exe = String::from_utf16_lossy(
                &entry.szExeFile[..entry
                    .szExeFile
                    .iter()
                    .position(|c| *c == 0)
                    .unwrap_or(entry.szExeFile.len())],
            );
            if exe.eq_ignore_ascii_case("enshrouded.exe")
                || exe.eq_ignore_ascii_case("enshrouded_server.exe")
            {
                let process = unsafe {
                    OpenProcess(
                        PROCESS_QUERY_LIMITED_INFORMATION | 0x0010_0000,
                        0,
                        entry.th32ProcessID,
                    )
                };
                if process.is_null() {
                    let error = unsafe { GetLastError() };
                    if error != ERROR_INVALID_PARAMETER {
                        unsafe { CloseHandle(snapshot) };
                        return Err(format!(
                            "cannot verify Enshrouded process {} (Windows error {error})",
                            entry.th32ProcessID
                        ));
                    }
                } else {
                    let mut path = vec![0u16; 32768];
                    let mut length = path.len() as u32;
                    let queried = unsafe {
                        QueryFullProcessImageNameW(process, 0, path.as_mut_ptr(), &mut length)
                    } != 0;
                    unsafe { CloseHandle(process) };
                    if !queried {
                        unsafe { CloseHandle(snapshot) };
                        return Err(format!(
                            "cannot read image path for Enshrouded process {}",
                            entry.th32ProcessID
                        ));
                    }
                    let image_path = String::from_utf16_lossy(&path[..length as usize]);
                    let image_root = PathBuf::from(&image_path)
                        .parent()
                        .and_then(|path| path.canonicalize().ok());
                    // Canonicalize both sides so Steam libraries reached through junctions
                    // still match. If the executable path cannot be resolved, wait safely
                    // instead of assuming the game has exited.
                    let belongs_to_install = image_root
                        .map(|path| path.to_string_lossy().eq_ignore_ascii_case(&root_key))
                        .unwrap_or(true);
                    if belongs_to_install {
                        running = Some((entry.th32ProcessID, image_path));
                        break;
                    }
                }
            }
            has_entry = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
        }
        unsafe { CloseHandle(snapshot) };
        if let Some((pid, image_path)) = running {
            if last_waiting_pid != Some(pid) {
                let _ = shroudforge_logging::append(
                    &root,
                    'D',
                    "updater",
                    &format!(
                        "Enshrouded is still running (PID {pid}, {image_path}). Waiting before installation"
                    ),
                );
                last_waiting_pid = Some(pid);
            }
        } else {
            if last_waiting_pid.is_some() {
                let _ = shroudforge_logging::append(
                    &root,
                    'I',
                    "updater",
                    "Enshrouded process exited. Updater continues",
                );
            }
            return Ok(());
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}


pub(crate) fn task_id(root: &Path) -> Result<String, String> {
    let canonical = root.canonicalize().map_err(|e| e.to_string())?;
    let mut hash = 0xcbf29ce484222325u64;
    for byte in canonical.to_string_lossy().to_ascii_lowercase().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    Ok(format!("ShroudForgeUpdater_{hash:016x}"))
}


pub(crate) fn task_executable(root: &Path) -> Result<PathBuf, String> {
    let source = shroudforge_loader_config::paths::updater_executable(root);
    if !source.is_file() {
        return Err(format!("updater executable missing: {}", source.display()));
    }
    let local = std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is unavailable")?;
    let modified = fs::metadata(&source)
        .and_then(|m| m.modified())
        .map_err(|e| e.to_string())?
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let destination = PathBuf::from(local)
        .join("ShroudForge")
        .join("Updater")
        .join(task_id(root)?)
        .join(modified.to_string())
        .join("shroudforge-updater.exe");
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if !destination.is_file() {
        fs::copy(&source, &destination)
            .map_err(|e| format!("cannot install independent updater: {e}"))?;
    }
    Ok(destination)
}


pub fn open_update_window(root: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    super::refresh_cached_repair_status(root)?;
    let source = shroudforge_loader_config::paths::loader_executable(root);
    if !source.is_file() {
        return Err(format!(
            "ShroudForge UI executable is missing: {}",
            source.display()
        ));
    }
    let updater = task_executable(root)?;
    let directory = updater
        .parent()
        .ok_or("updater cache has no parent directory")?;
    let metadata = fs::metadata(&source).map_err(|error| error.to_string())?;
    let changed = metadata
        .modified()
        .map_err(|error| error.to_string())?
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let ui = directory.join(format!("shroudforge-ui-{changed}.exe"));
    if !ui.is_file() {
        fs::copy(&source, &ui)
            .map_err(|error| format!("could not prepare independent update window: {error}"))?;
    }
    let target = if root.join("enshrouded.exe").is_file() {
        "client"
    } else {
        "server"
    };
    Command::new(&ui)
        .args(["--module-ui", "--desktop", "--updater-window", "--root"])
        .arg(root)
        .args(["--target", target])
        .creation_flags(0x0800_0000)
        .spawn()
        .map_err(|error| format!("could not launch independent update window: {error}"))?;
    Ok(())
}


pub fn start_system_waiter(root: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let executable = task_executable(root)?;
    let canonical_root = root.canonicalize().map_err(|error| error.to_string())?;
    Command::new(executable)
        .args(["--run-queue", "--root"])
        .arg(canonical_root)
        .creation_flags(0x08000000)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("could not start the queued update waiter: {error}"))
}


pub(crate) fn ensure_task(root: &Path) -> Result<String, String> {
    let executable = task_executable(root)?;
    let name = task_id(root)?;
    let action = format!(
        "\"{}\" --run-queue --root \"{}\"",
        executable.display(),
        root.canonicalize().map_err(|e| e.to_string())?.display()
    );
    let output = Command::new("schtasks.exe")
        .args([
            "/Create", "/SC", "ONLOGON", "/TN", &name, "/TR", &action, "/F", "/RL", "LIMITED",
            "/IT",
        ])
        .output()
        .map_err(|e| format!("could not register updater task: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "could not register updater task: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(name)
}


pub(crate) fn spawn_worker(root: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let executable = task_executable(root)?;
    let canonical_root = root.canonicalize().map_err(|error| error.to_string())?;
    Command::new(executable)
        .args(["--run-queue", "--root"])
        .arg(canonical_root)
        .creation_flags(0x08000000)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("could not start updater worker: {error}"))
}


pub fn request(root: &Path) -> Result<(), String> {
    let name = match ensure_task(root) {
        Ok(name) => name,
        Err(task_error) => {
            return spawn_worker(root).map_err(|spawn_error| {
                format!("{task_error}. Direct updater start also failed: {spawn_error}")
            });
        }
    };
    let output = match Command::new("schtasks.exe")
        .args(["/Run", "/TN", &name])
        .output()
    {
        Ok(output) => output,
        Err(task_error) => {
            let task_error = format!("could not start scheduled updater: {task_error}");
            return spawn_worker(root).map_err(|spawn_error| {
                format!("{task_error}. Direct updater start also failed: {spawn_error}")
            });
        }
    };
    if !output.status.success() {
        let task_error = format!(
            "could not start scheduled updater: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        return spawn_worker(root).map_err(|spawn_error| {
            format!("{task_error}. Direct updater start also failed: {spawn_error}")
        });
    }
    Ok(())
}
