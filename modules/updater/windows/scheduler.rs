pub(crate) use crate::check_update_cancelled::check_update_cancelled;
pub(crate) use crate::clear_staged_update::clear_system_update;
pub(crate) use crate::system_stage::stage_system_update;
pub(crate) use crate::write_update_status::{write_status, write_status_with_version};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, ERROR_INVALID_PARAMETER, ERROR_NO_MORE_FILES, GetLastError},
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
            TH32CS_SNAPPROCESS,
        },
        Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW},
    },
    UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId, PostMessageW, WM_CLOSE},
};

pub(crate) fn prepare_release_file_locks(
    root: &Path,
    relative_paths: &[String],
) -> Result<(), String> {
    use windows_sys::Win32::System::RestartManager::{
        RmEndSession, RmRegisterResources, RmStartSession,
    };

    let mut files = Vec::new();
    for relative in relative_paths {
        let path = crate::validate_update_paths::installed_path(root, relative);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.is_file() => files.push(
                path.to_string_lossy()
                    .encode_utf16()
                    .chain(Some(0))
                    .collect::<Vec<_>>(),
            ),
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "could not inspect update target {} before installation: {error}",
                    path.display()
                ));
            }
        }
    }
    if files.is_empty() {
        return Ok(());
    }
    let pointers = files.iter().map(|path| path.as_ptr()).collect::<Vec<_>>();
    let mut session = 0;
    let mut key = vec![0u16; 33];
    let start = unsafe { RmStartSession(&mut session, 0, key.as_mut_ptr()) };
    if start != 0 {
        return Err(format!(
            "Windows Restart Manager could not start (error {start})"
        ));
    }
    let result = (|| {
        let registered = unsafe {
            RmRegisterResources(
                session,
                pointers.len() as u32,
                pointers.as_ptr(),
                0,
                std::ptr::null(),
                0,
                std::ptr::null(),
            )
        };
        if registered != 0 {
            return Err(format!(
                "Windows Restart Manager could not inspect update files (error {registered})"
            ));
        }
        let processes = restart_manager_processes(session)?;
        if processes.is_empty() {
            return Ok(());
        }
        let mut owned = Vec::new();
        let mut foreign = Vec::new();
        for process in processes {
            let image = process_image_path(process.Process.dwProcessId)?;
            if is_owned_shroudforge_process(root, &image) {
                owned.push((process.Process.dwProcessId, image));
            } else {
                let name = String::from_utf16_lossy(
                    &process.strAppName[..process
                        .strAppName
                        .iter()
                        .position(|unit| *unit == 0)
                        .unwrap_or(process.strAppName.len())],
                );
                foreign.push(format!(
                    "{name} (PID {}, {})",
                    process.Process.dwProcessId,
                    image.display()
                ));
            }
        }
        if !foreign.is_empty() {
            return Err(format!(
                "Update files are in use by another process. Close it and retry: {}",
                foreign.join(", ")
            ));
        }
        if owned.is_empty() {
            return Ok(());
        }
        let pids = owned.iter().map(|(pid, _)| *pid).collect::<HashSet<_>>();
        let close_requests = request_process_close(&pids)?;
        let _ = shroudforge_logging::append(
            root,
            'I',
            "updater",
            &format!(
                "Asked {} ShroudForge process(es) to close before replacing update files ({} window close request(s))",
                owned.len(),
                close_requests
            ),
        );
        let started = std::time::Instant::now();
        while started.elapsed() < std::time::Duration::from_secs(15) {
            check_update_cancelled(root)?;
            let mut remaining = Vec::new();
            for (pid, image) in &owned {
                if process_is_running(*pid)? {
                    remaining.push((pid, image));
                }
            }
            if remaining.is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
        let mut remaining = Vec::new();
        for (pid, image) in owned {
            if process_is_running(pid)? {
                remaining.push((pid, image));
            }
        }
        if !remaining.is_empty() {
            let _ = shroudforge_logging::append(
                root,
                'W',
                "updater",
                &format!(
                    "ShroudForge still has {} process(es) open after the graceful close request",
                    remaining.len()
                ),
            );
            let processes = remaining
                .iter()
                .map(|(pid, image)| format!("{} (PID {pid})", image.display()))
                .collect::<Vec<_>>();
            return Err(format!(
                "Update files are still in use by ShroudForge. Save your work, close these windows, then retry: {}. The updater left them open.",
                processes.join(", ")
            ));
        }
        let remaining = restart_manager_processes(session)?;
        if !remaining.is_empty() {
            return Err(format!(
                "Update files remain locked after closing ShroudForge processes ({} process(es) reported)",
                remaining.len()
            ));
        }
        Ok(())
    })();
    unsafe { RmEndSession(session) };
    result
}

pub(crate) fn prepare_shroudforge_data_purge_locks(root: &Path) -> Result<(), String> {
    let data = shroudforge_loader_config::paths::data_dir(root);
    let mut paths = Vec::new();
    collect_purge_lock_paths(root, &data, &mut paths)?;
    paths.sort();
    paths.dedup();
    prepare_release_file_locks(root, &paths)
}

fn collect_purge_lock_paths(
    root: &Path,
    directory: &Path,
    paths: &mut Vec<String>,
) -> Result<(), String> {
    use std::os::windows::fs::MetadataExt;

    for entry in fs::read_dir(directory).map_err(|error| {
        format!(
            "could not inspect ShroudForge data before purge: {}: {error}",
            directory.display()
        )
    })? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            format!(
                "could not inspect ShroudForge data before purge: {}: {error}",
                path.display()
            )
        })?;
        if metadata.file_type().is_symlink() || metadata.file_attributes() & 0x400 != 0 {
            continue;
        }
        if metadata.is_dir() {
            collect_purge_lock_paths(root, &path, paths)?;
        } else if metadata.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|error| error.to_string())?
                .to_str()
                .ok_or("ShroudForge data contains a path that cannot be checked safely")?
                .replace('\\', "/");
            paths.push(relative);
        }
    }
    Ok(())
}

fn restart_manager_processes(
    session: u32,
) -> Result<Vec<windows_sys::Win32::System::RestartManager::RM_PROCESS_INFO>, String> {
    use windows_sys::Win32::System::RestartManager::{RM_PROCESS_INFO, RmGetList};
    let mut needed = 0;
    let mut count = 0;
    let mut reasons = 0;
    let first = unsafe {
        RmGetList(
            session,
            &mut needed,
            &mut count,
            std::ptr::null_mut(),
            &mut reasons,
        )
    };
    if first == 0 {
        return Ok(Vec::new());
    }
    if first != 234 {
        return Err(format!(
            "Windows Restart Manager could not list file locks (error {first})"
        ));
    }
    let mut processes = vec![RM_PROCESS_INFO::default(); needed as usize];
    count = needed;
    let listed = unsafe {
        RmGetList(
            session,
            &mut needed,
            &mut count,
            processes.as_mut_ptr(),
            &mut reasons,
        )
    };
    if listed != 0 {
        return Err(format!(
            "Windows Restart Manager could not read file-lock details (error {listed})"
        ));
    }
    processes.truncate(count as usize);
    Ok(processes)
}

fn process_image_path(pid: u32) -> Result<PathBuf, String> {
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if process.is_null() {
        return Err(format!(
            "cannot inspect update file-lock process {pid} (Windows error {})",
            unsafe { GetLastError() }
        ));
    }
    let mut path = vec![0u16; 32_768];
    let mut length = path.len() as u32;
    let queried =
        unsafe { QueryFullProcessImageNameW(process, 0, path.as_mut_ptr(), &mut length) } != 0;
    let error = if queried {
        0
    } else {
        unsafe { GetLastError() }
    };
    unsafe { CloseHandle(process) };
    if !queried {
        return Err(format!(
            "cannot read image path for update file-lock process {pid} (Windows error {error})"
        ));
    }
    Ok(PathBuf::from(String::from_utf16_lossy(
        &path[..length as usize],
    )))
}

fn is_owned_shroudforge_process(root: &Path, image: &Path) -> bool {
    let name = image
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if ![
        "shroudforge.exe",
        "shroudforge-updater.exe",
        "updater-worker.exe",
    ]
    .iter()
    .any(|known| name.eq_ignore_ascii_case(known))
        && !name.to_ascii_lowercase().starts_with("shroudforge-ui-")
        && !name.to_ascii_lowercase().starts_with("updater-worker-")
    {
        return false;
    }
    if name.to_ascii_lowercase().starts_with("updater-worker-") {
        return image
            .canonicalize()
            .ok()
            .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
            .zip(
                shroudforge_loader_config::paths::updates_dir(root)
                    .canonicalize()
                    .ok(),
            )
            .is_some_and(|(parent, updates)| parent == updates);
    }
    let module_dir = shroudforge_loader_config::paths::data_dir(root);
    if image
        .canonicalize()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
        .zip(module_dir.canonicalize().ok())
        .is_some_and(|(parent, module)| parent == module)
    {
        return true;
    }
    [
        shroudforge_loader_config::paths::loader_executable(root),
        shroudforge_loader_config::paths::updater_executable(root),
    ]
    .iter()
    .filter_map(|path| hash_file(path).ok())
    .any(|expected| hash_file(image).is_ok_and(|actual| actual == expected))
}

fn hash_file(path: &Path) -> Result<[u8; 32], String> {
    use sha2::Digest;
    use std::io::Read;
    let mut file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut hash = sha2::Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(hash.finalize().into())
}

fn request_process_close(pids: &HashSet<u32>) -> Result<usize, String> {
    struct CloseRequests<'a> {
        pids: &'a HashSet<u32>,
        posted: usize,
    }
    unsafe extern "system" fn close_window(
        window: windows_sys::Win32::Foundation::HWND,
        context: isize,
    ) -> i32 {
        let requests = unsafe { &mut *(context as *mut CloseRequests<'_>) };
        let mut pid = 0;
        unsafe { GetWindowThreadProcessId(window, &mut pid) };
        if requests.pids.contains(&pid) && unsafe { PostMessageW(window, WM_CLOSE, 0, 0) } != 0 {
            requests.posted += 1;
        }
        1
    }
    let mut requests = CloseRequests { pids, posted: 0 };
    if unsafe {
        EnumWindows(
            Some(close_window),
            &mut requests as *mut CloseRequests<'_> as isize,
        )
    } == 0
    {
        return Err(format!(
            "Windows could not request ShroudForge windows to close (error {})",
            unsafe { GetLastError() }
        ));
    }
    Ok(requests.posted)
}

pub(crate) fn process_is_running(pid: u32) -> Result<bool, String> {
    use windows_sys::Win32::System::Threading::{PROCESS_SYNCHRONIZE, WaitForSingleObject};
    let process = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, pid) };
    if process.is_null() {
        let error = unsafe { GetLastError() };
        return if error == ERROR_INVALID_PARAMETER {
            Ok(false)
        } else {
            Err(format!(
                "cannot verify whether ShroudForge process {pid} has exited (Windows error {error})"
            ))
        };
    }
    let wait = unsafe { WaitForSingleObject(process, 0) };
    unsafe { CloseHandle(process) };
    match wait {
        0 => Ok(false),
        0x0000_0102 => Ok(true),
        _ => Err(format!(
            "could not verify whether ShroudForge process {pid} has exited (Windows error {})",
            unsafe { GetLastError() }
        )),
    }
}

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
        if !has_entry {
            let error = unsafe { GetLastError() };
            if error != ERROR_NO_MORE_FILES {
                unsafe { CloseHandle(snapshot) };
                return Err(format!(
                    "could not read the process list (Windows error {error})"
                ));
            }
        }
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
            if !has_entry {
                let error = unsafe { GetLastError() };
                if error != ERROR_NO_MORE_FILES {
                    unsafe { CloseHandle(snapshot) };
                    return Err(format!(
                        "could not continue reading the process list (Windows error {error})"
                    ));
                }
            }
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

pub(crate) fn wait_for_process_id(root: &Path, pid: u32) -> Result<(), String> {
    use windows_sys::Win32::System::Threading::WaitForSingleObject;
    let process = unsafe { OpenProcess(0x0010_0000, 0, pid) };
    if process.is_null() {
        let error = unsafe { GetLastError() };
        if error == ERROR_INVALID_PARAMETER {
            return Ok(());
        }
        return Err(format!(
            "cannot wait for bootstrap process {pid} (Windows error {error})"
        ));
    }
    let _ = shroudforge_logging::append(
        root,
        'D',
        "updater",
        &format!("Waiting for bootstrap process {pid} to exit before installation"),
    );
    loop {
        check_update_cancelled(root)?;
        match unsafe { WaitForSingleObject(process, 1_000) } {
            0 => break,
            0x0000_0102 => {}
            _ => {
                let error = unsafe { GetLastError() };
                unsafe { CloseHandle(process) };
                return Err(format!(
                    "could not wait for bootstrap process {pid} (Windows error {error})"
                ));
            }
        }
    }
    unsafe { CloseHandle(process) };
    let _ = shroudforge_logging::append(
        root,
        'I',
        "updater",
        &format!("Bootstrap process {pid} exited. Updater continues"),
    );
    Ok(())
}

pub fn open_update_window(root: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let installed = shroudforge_loader_config::paths::updater_executable(root);
    if !installed.is_file() {
        return Err(format!(
            "ShroudForge updater executable is missing: {}",
            installed.display()
        ));
    }
    // The update window must not keep a managed updater executable open while
    // the queue worker replaces the installed program files.
    let executable = crate::windows::prepare_independent_worker(root, &installed)?;
    let result = Command::new(&executable)
        .args(["--open-window", "--root"])
        .arg(root)
        .arg("--ephemeral-worker")
        .creation_flags(0x0800_0000)
        .spawn();
    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            let _ = fs::remove_file(executable);
            Err(format!("could not open standalone updater window: {error}"))
        }
    }
}

pub fn start_system_waiter(root: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let executable = shroudforge_loader_config::paths::updater_executable(root);
    if !executable.is_file() {
        return Err(format!(
            "updater executable missing: {}",
            executable.display()
        ));
    }
    let canonical_root = root.canonicalize().map_err(|error| error.to_string())?;
    Command::new(executable)
        .args(["--run-queue", "--root"])
        .arg(canonical_root)
        .creation_flags(0x08000000)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("could not start the queued update waiter: {error}"))
}
