use crate::*;
#[cfg(windows)]
pub fn run_scheduled_worker() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let root = args
        .windows(2)
        .find(|pair| pair[0] == "--root")
        .map(|pair| std::path::PathBuf::from(&pair[1]))
        .ok_or("missing --root")?;
    let root = root
        .canonicalize()
        .map_err(|error| format!("could not resolve update worker installation: {error}"))?;
    let Some(_worker_mutex) = acquire_queue_worker_mutex(&root)? else {
        let _ = shroudforge_logging::append(
            &root,
            'D',
            "updater",
            "A queue worker for this installation is already running",
        );
        return Ok(());
    };
    let queue = read_update_queue_value(&root);
    if queue["run"]["state"] == "running" {
        return run_queue_with_failure_handling(&root);
    }

    // Old request files are imported by versioned jobs in the one update queue.
    // They are not processed by a second worker path.
    crate::maintenance_jobs::enqueue_before_update(&root)?;
    let queue = read_update_queue_value(&root);
    let ids = queue["items"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|item| item["kind"] == "maintenance" && item["state"] == "queued")
        .filter_map(|item| item["id"].as_str().map(str::to_owned))
        .collect::<Vec<_>>();
    if ids.is_empty() {
        return Ok(());
    }
    let updates = shroudforge_loader_config::paths::updates_dir(&root);
    let wait_for_game = updates.join("worker-queue.json").is_file();
    let wait_pid = std::fs::read(updates.join("worker-queue.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|value| value["waitPid"].as_u64())
        .unwrap_or_default();
    crate::update_queue_storage::update_update_queue(&root, |queue| {
        if queue["run"]["state"] == "running" {
            return Err("the update queue is already running".into());
        }
        queue["run"] = serde_json::json!({
            "state": "running",
            "waitForGame": wait_for_game,
            "waitPid": wait_pid,
            "showWindow": false,
            "itemIds": ids,
            "activeItemId": null,
            "installing": false,
            "startedAt": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });
        Ok(())
    })?;
    run_queue_with_failure_handling(&root)
}

#[cfg(windows)]
pub(crate) struct QueueWorkerMutex(windows_sys::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl Drop for QueueWorkerMutex {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::System::Threading::ReleaseMutex(self.0);
            windows_sys::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

#[cfg(windows)]
pub(crate) fn acquire_queue_worker_mutex(
    root: &std::path::Path,
) -> Result<Option<QueueWorkerMutex>, String> {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, WAIT_ABANDONED, WAIT_OBJECT_0, WAIT_TIMEOUT},
        System::Threading::WaitForSingleObject,
    };
    let handle = create_queue_worker_mutex_handle(root)?;
    match unsafe { WaitForSingleObject(handle, 0) } {
        WAIT_OBJECT_0 | WAIT_ABANDONED => Ok(Some(QueueWorkerMutex(handle))),
        WAIT_TIMEOUT => {
            unsafe { CloseHandle(handle) };
            Ok(None)
        }
        result => {
            let error = unsafe { windows_sys::Win32::Foundation::GetLastError() };
            unsafe { CloseHandle(handle) };
            Err(format!(
                "could not acquire the update queue lock (wait result {result}, Windows error {error})"
            ))
        }
    }
}

#[cfg(windows)]
pub(crate) fn wait_for_queue_worker_mutex(
    root: &std::path::Path,
) -> Result<QueueWorkerMutex, String> {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, WAIT_ABANDONED, WAIT_OBJECT_0},
        System::Threading::{WaitForSingleObject, INFINITE},
    };
    let handle = create_queue_worker_mutex_handle(root)?;
    match unsafe { WaitForSingleObject(handle, INFINITE) } {
        WAIT_OBJECT_0 | WAIT_ABANDONED => Ok(QueueWorkerMutex(handle)),
        result => {
            let error = unsafe { windows_sys::Win32::Foundation::GetLastError() };
            unsafe { CloseHandle(handle) };
            Err(format!(
                "could not wait for the update queue lock (wait result {result}, Windows error {error})"
            ))
        }
    }
}

#[cfg(windows)]
fn create_queue_worker_mutex_handle(
    root: &std::path::Path,
) -> Result<windows_sys::Win32::Foundation::HANDLE, String> {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
    use windows_sys::Win32::System::Threading::CreateMutexW;
    use sha2::Digest;

    let normalized_root = root
        .canonicalize()
        .map_err(|error| format!("could not resolve installation path for update lock: {error}"))?;
    let name = format!(
        "Local\\ShroudForgeUpdateQueue_{:x}",
        sha2::Sha256::digest(normalized_root.to_string_lossy().to_ascii_lowercase().as_bytes())
    );
    let wide: Vec<u16> = OsStr::new(&name).encode_wide().chain(Some(0)).collect();
    let handle = unsafe { CreateMutexW(std::ptr::null(), 0, wide.as_ptr()) };
    if handle.is_null() {
        return Err(format!(
            "could not create the update queue lock (Windows error {})",
            unsafe { windows_sys::Win32::Foundation::GetLastError() }
        ));
    }
    Ok(handle)
}

#[cfg(windows)]
fn run_queue_with_failure_handling(root: &std::path::Path) -> Result<(), String> {
    match run_update_queue(root) {
        Ok(()) => Ok(()),
        Err(error) if error == "UPDATE_CANCELLED" => {
            let ids = read_update_queue_value(root)["run"]["itemIds"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_owned)
                .collect::<Vec<_>>();
            crate::finish_cancelled_queue::finish_cancelled_queue(root, &ids)
        }
        Err(error) => {
            if read_update_queue_value(root)["run"]["state"] != "running" {
                return Ok(());
            }
            record_queue_worker_failure(root, &error)
        }
    }
}

#[cfg(windows)]
pub(crate) fn record_queue_worker_failure(
    root: &std::path::Path,
    error: &str,
) -> Result<(), String> {
    let message = error.to_owned();
    let recorded = crate::update_queue_storage::update_update_queue(root, |queue| {
        if queue["run"]["state"] != "running" {
            return Ok(false);
        }
        if let Some(active_id) = queue["run"]["activeItemId"].as_str().map(str::to_owned)
            && let Some(item) = queue["items"].as_array_mut().and_then(|items| {
                items
                    .iter_mut()
                    .find(|item| item["id"].as_str() == Some(&active_id))
            })
            && matches!(item["state"].as_str(), Some("downloading" | "installing"))
        {
            item["state"] = serde_json::Value::String("error".into());
            item["message"] = serde_json::Value::Null;
        }
        queue["run"]["state"] = serde_json::Value::String("error".into());
        queue["run"]["error"] = serde_json::Value::String(message.clone());
        queue["run"]["installing"] = serde_json::Value::Bool(false);
        queue["run"]["finishedAt"] = serde_json::json!(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
        Ok(true)
    })?;
    if !recorded {
        return Ok(());
    }
    scheduled::write_status(root, "error", error);
    shroudforge_logging::write_activity(
        root,
        "Updates",
        "Run update queue",
        "Failed",
        Some(error),
        "error",
    );
    Ok(())
}
#[cfg(windows)]
pub fn run_module() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(root) = args
        .windows(2)
        .find(|pair| pair[0] == "--root")
        .map(|pair| std::path::PathBuf::from(&pair[1]))
    {
        match windows::hand_off_installed_worker(&root) {
            Ok(true) => return Ok(()),
            Ok(false) => {}
            Err(error) => {
                if args.iter().any(|argument| argument == "--run-queue") {
                    return match record_queue_worker_failure(&root, &error) {
                        Ok(()) => Ok(()),
                        Err(record_error) => Err(format!(
                            "could not start the independent updater worker: {error}. Could not record this failure in the update queue: {record_error}"
                        )),
                    };
                }
                return Err(error);
            }
        }
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
        return crate::ui::host::window::run(Some(root));
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
        return crate::ui::host::window::run(Some(root));
    }
    windows::run()
}

#[cfg(not(windows))]
pub fn run_module() -> Result<(), String> {
    Err("ShroudForge updater is available on Windows only".into())
}
