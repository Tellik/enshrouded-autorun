pub fn run_headless_control_controller(
    root: &std::path::Path,
    parent_pid: u32,
) -> Result<(), String> {
    #[cfg(windows)]
    use crate::claim_update_actions::claim_headless_control_actions;
    #[cfg(windows)]
    use crate::complete_update_action::mark_headless_control_action;
    #[cfg(windows)]
    use crate::execute_update_action::execute_headless_control_action;
    #[cfg(target_os = "linux")]
    use std::time::Duration;
    use std::time::Instant;

    if parent_pid == 0 {
        return Err("updater controller is missing its parent process ID".into());
    }
    let root = root
        .canonicalize()
        .map_err(|error| format!("could not resolve game root for updater controller: {error}"))?;
    #[cfg(windows)]
    let process = {
        use windows_sys::Win32::Foundation::{ERROR_INVALID_PARAMETER, GetLastError};
        use windows_sys::Win32::System::Threading::OpenProcess;
        let process = unsafe { OpenProcess(0x0010_0000, 0, parent_pid) };
        if process.is_null() {
            let error = unsafe { GetLastError() };
            if error == ERROR_INVALID_PARAMETER {
                return Ok(());
            }
            return Err(format!(
                "cannot monitor loader process {parent_pid} (Windows error {error})"
            ));
        }
        process
    };
    #[cfg(target_os = "linux")]
    let parent_process = std::path::PathBuf::from(format!("/proc/{parent_pid}"));
    #[cfg(not(any(windows, target_os = "linux")))]
    return Err("automatic updater checks are supported on Windows and Linux".into());

    let mut update_settings = read_update_settings(&root);
    let mut next_mod_check = Instant::now();
    #[cfg(windows)]
    let mut next_system_check = Instant::now();
    loop {
        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0, WAIT_TIMEOUT};
            use windows_sys::Win32::System::Threading::WaitForSingleObject;
            match unsafe { WaitForSingleObject(process, 500) } {
                WAIT_OBJECT_0 => break,
                WAIT_TIMEOUT => {}
                result => {
                    unsafe { CloseHandle(process) };
                    return Err(format!(
                        "could not monitor loader process {parent_pid} (wait result {result})"
                    ));
                }
            }
        }
        #[cfg(target_os = "linux")]
        {
            if !parent_process.exists() {
                break;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        let current_settings = read_update_settings(&root);
        if current_settings != update_settings {
            update_settings = current_settings;
            next_mod_check = Instant::now() + crate::mod_update_check_delay(&root);
            #[cfg(windows)]
            {
                next_system_check = Instant::now() + crate::system_update_check_delay(&root);
            }
        }
        let updates_enabled = update_settings["enabled"].as_bool();
        if updates_enabled == Some(true) {
            if Instant::now() >= next_mod_check {
                next_mod_check = crate::check_due_mod_updates(&root);
            }
            #[cfg(windows)]
            if Instant::now() >= next_system_check {
                next_system_check = crate::check_due_system_update::check_due_system_update(&root);
            }
        }
        #[cfg(windows)]
        if updates_enabled.is_some() {
            match claim_headless_control_actions(&root) {
                Ok(actions) => {
                    for action in actions {
                        if updates_enabled == Some(false) {
                            mark_headless_control_action(
                                &root,
                                &action.name,
                                &action.request_id,
                                "idle",
                                crate::UPDATE_MODULE_DISABLED_MESSAGE,
                            );
                            let _ = shroudforge_logging::append(
                                &root,
                                'W',
                                "config-controller",
                                &format!(
                                    "Updater action {} unavailable: {}",
                                    action.name,
                                    crate::UPDATE_MODULE_DISABLED_MESSAGE
                                ),
                            );
                            continue;
                        }
                        let result = execute_headless_control_action(&root, &action);
                        let (state, message) = match result {
                            Ok(message) => ("succeeded", message),
                            Err(error) => ("failed", error),
                        };
                        mark_headless_control_action(
                            &root,
                            &action.name,
                            &action.request_id,
                            state,
                            &message,
                        );
                    }
                }
                Err(error) => {
                    let _ = shroudforge_logging::append(
                        &root,
                        'E',
                        "config-controller",
                        &format!("Could not read pending server actions: {error}"),
                    );
                }
            }
        }
    }
    #[cfg(windows)]
    unsafe {
        windows_sys::Win32::Foundation::CloseHandle(process);
    }
    #[cfg(target_os = "linux")]
    let _ = parent_process;
    Ok(())
}

fn read_update_settings(root: &std::path::Path) -> serde_json::Value {
    shroudforge_loader_config::config::read_loader(root)
        .ok()
        .and_then(|config| {
            config
                .get("modules")
                .and_then(|modules| modules.get("updates"))
                .cloned()
        })
        .unwrap_or_default()
}
