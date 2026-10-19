#![cfg(windows)]

use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    time::{Duration, Instant},
};
use tao::{
    dpi::LogicalSize,
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::{WebContext, WebViewBuilder};

pub fn run(root: Option<PathBuf>) -> Result<(), String> {
    let selected = root.or_else(installed_root);
    let selected = match selected {
        Some(path) if is_game_root(&path) => path,
        Some(_) => return Err("the updater root is not an Enshrouded installation".into()),
        None => shroudforge_window_management::choose_folder(
            "Choose the Enshrouded installation folder",
        )?
        .ok_or_else(|| "no installation folder was selected".to_owned())?,
    };
    if !is_game_root(&selected) {
        return Err("the selected folder is not an Enshrouded installation".into());
    }
    shroudforge_loader_config::config::initialize_loader_config(&selected)?;
    shroudforge_loader_config::config::request_window_visibility(&selected, "updates", true)?;
    let Some(window_mutex) = acquire_window_mutex(&selected)? else {
        return Ok(());
    };
    let initial_window_state = shroudforge_loader_config::config::window_state(&selected);
    let mut visibility_request_id = initial_window_state["updates"]["requestId"]
        .as_u64()
        .unwrap_or(0);
    let root = Arc::new(Mutex::new(Some(selected)));
    let (command_sender, command_receiver) = mpsc::channel::<serde_json::Value>();
    let (result_sender, result_receiver) = mpsc::channel::<serde_json::Value>();
    let check_state = Arc::new(Mutex::new(String::new()));
    let mod_check_running = Arc::new(Mutex::new(false));
    let event_loop = EventLoopBuilder::<()>::with_user_event().build();
    let window = WindowBuilder::new()
        .with_title("ShroudForge Updater")
        .with_resizable(true)
        .with_inner_size(LogicalSize::new(960.0, 720.0))
        .build(&event_loop)
        .map_err(|error| error.to_string())?;
    let profile = root.lock().ok().and_then(|root| root.as_ref().cloned());
    let profile = profile
        .map(|root| shroudforge_loader_config::paths::updates_dir(&root).join("webview-profile"));
    if let Some(profile) = &profile {
        std::fs::create_dir_all(profile).map_err(|error| error.to_string())?;
    }
    let mut context = WebContext::new(profile);
    if let Some(root) = root.lock().ok().and_then(|root| root.clone()) {
        let _ =
            shroudforge_logging::append(&root, 'I', "updater", "Standalone updater window opened");
    }
    let sender = command_sender.clone();
    let handler = move |request: wry::http::Request<String>| {
        if let Ok(command) = serde_json::from_str(request.body()) {
            let _ = sender.send(command);
        }
    };
    let webview = WebViewBuilder::new_with_web_context(&mut context)
        .with_html(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../build/ui/updater/updater.html"
        )))
        .with_ipc_handler(handler)
        .build(&window)
        .map_err(|error| error.to_string())?;
    if let Some(root) = root.lock().ok().and_then(|root| root.clone()) {
        let _ = shroudforge_loader_config::config::publish_window_visibility(
            &root, "updates", true,
        );
    }

    let mut next_refresh = Instant::now();
    let mut next_visibility_poll = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        let _keep_window_mutex = &window_mutex;
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(100));
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            if let Some(root) = root.lock().ok().and_then(|root| root.clone()) {
                let _ = shroudforge_loader_config::config::publish_window_visibility(
                    &root, "updates", false,
                );
            }
            *control_flow = ControlFlow::Exit;
            return;
        }
        if matches!(event, Event::MainEventsCleared)
            || matches!(event, Event::NewEvents(StartCause::Init))
            || matches!(event, Event::NewEvents(StartCause::ResumeTimeReached { .. }))
        {
            while let Ok(command) = command_receiver.try_recv() {
                let exit = handle_command(
                    &command,
                    &root,
                    &check_state,
                    &mod_check_running,
                    &result_sender,
                );
                if exit {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            while let Ok(result) = result_receiver.try_recv() {
                send_event(&webview, "__shroudforgeUpdaterResult", &result);
            }
            if Instant::now() >= next_visibility_poll {
                if let Some(root) = root.lock().ok().and_then(|root| root.clone()) {
                    let state = shroudforge_loader_config::config::window_state(&root);
                    let request_id = state["updates"]["requestId"]
                        .as_u64()
                        .unwrap_or(visibility_request_id);
                    if request_id != visibility_request_id {
                        visibility_request_id = request_id;
                        let visible = state["updates"]["requestedVisible"]
                            .as_bool()
                            .unwrap_or(true);
                        window.set_visible(visible);
                        if visible {
                            window.set_minimized(false);
                            window.set_focus();
                        }
                        let _ = shroudforge_loader_config::config::publish_window_visibility(
                            &root, "updates", visible,
                        );
                        shroudforge_logging::write_activity(
                            &root,
                            "Updates",
                            if visible {
                                "Show updater window"
                            } else {
                                "Hide updater window"
                            },
                            "Succeeded",
                            None,
                            "info",
                        );
                    }
                }
                next_visibility_poll = Instant::now() + Duration::from_millis(100);
            }
            if Instant::now() >= next_refresh {
                if let Some(root) = root.lock().ok().and_then(|root| root.clone()) {
                    let snapshot = read_snapshot(&root, &check_state, &mod_check_running);
                    send_event(&webview, "__shroudforgeUpdaterSnapshot", &snapshot);
                } else {
                    let snapshot = serde_json::json!({
                        "root": "",
                        "version": env!("CARGO_PKG_VERSION"),
                        "release": {"currentVersion": env!("CARGO_PKG_VERSION"), "state":"idle", "message":"Select the Enshrouded installation folder to continue."},
                        "queue": {"items": [], "run": null},
                    });
                    send_event(&webview, "__shroudforgeUpdaterSnapshot", &snapshot);
                }
                next_refresh = Instant::now() + Duration::from_millis(700);
            }
        }
    });
}

struct UpdaterWindowMutex(windows_sys::Win32::Foundation::HANDLE);

impl Drop for UpdaterWindowMutex {
    fn drop(&mut self) {
        unsafe { windows_sys::Win32::Foundation::CloseHandle(self.0) };
    }
}

fn acquire_window_mutex(root: &Path) -> Result<Option<UpdaterWindowMutex>, String> {
    use sha2::Digest;
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
    use windows_sys::Win32::{
        Foundation::{ERROR_ALREADY_EXISTS, GetLastError},
        System::Threading::CreateMutexW,
    };

    let normalized_root = root
        .canonicalize()
        .unwrap_or_else(|_| root.to_path_buf())
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase();
    let name = format!(
        "Local\\ShroudForgeUpdaterWindow_{:x}",
        sha2::Sha256::digest(normalized_root.as_bytes())
    );
    let wide: Vec<u16> = OsStr::new(&name).encode_wide().chain(Some(0)).collect();
    let handle = unsafe { CreateMutexW(std::ptr::null(), 1, wide.as_ptr()) };
    if handle.is_null() {
        return Err(std::io::Error::last_os_error().to_string());
    }
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) };
        return Ok(None);
    }
    Ok(Some(UpdaterWindowMutex(handle)))
}

fn handle_command(
    command: &serde_json::Value,
    root: &Arc<Mutex<Option<PathBuf>>>,
    check_state: &Arc<Mutex<String>>,
    mod_check_running: &Arc<Mutex<bool>>,
    results: &mpsc::Sender<serde_json::Value>,
) -> bool {
    let action = command["command"].as_str().unwrap_or_default();
    if action == "choose-root" {
        match shroudforge_window_management::choose_folder(
            "Choose the Enshrouded installation folder",
        ) {
            Ok(Some(path)) if is_game_root(&path) => {
                if let Ok(mut selected) = root.lock() {
                    *selected = Some(path.clone());
                }
                let _ = shroudforge_logging::append(
                    &path,
                    'I',
                    "updater",
                    &format!("Installation selected: {}", path.display()),
                );
            }
            Ok(Some(_)) => {
                let _ = results.send(serde_json::json!({"error":"The selected folder is not an Enshrouded installation."}));
            }
            Ok(None) => {}
            Err(error) => {
                let _ = results.send(serde_json::json!({"error":error}));
            }
        }
        return false;
    }
    let Some(root_path) = root.lock().ok().and_then(|root| root.clone()) else {
        let _ = results
            .send(serde_json::json!({"error":"Select the Enshrouded installation folder first."}));
        return false;
    };
    let result = match action {
        "check-updates" => {
            let check_state = check_state.clone();
            let results = results.clone();
            let root = root_path.clone();
            let Ok(mut value) = check_state.lock() else {
                let _ = results.send(serde_json::json!({"error":"Could not update the system check status."}));
                return false;
            };
            if *value == "checking" {
                return false;
            }
            *value = "checking".into();
            drop(value);
            std::thread::spawn(move || {
                let result = crate::run_request(
                    &root,
                    &serde_json::json!({"command":"check-system-update","manual":true}),
                );
                let message = match result {
                    Ok(value) => value,
                    Err(error) => serde_json::json!({"error":error}),
                };
                if let Ok(mut value) = check_state.lock() {
                    value.clear();
                }
                let _ = results.send(message);
            });
            Ok(())
        }
        "check-mod-updates" => {
            let running = mod_check_running.clone();
            let results = results.clone();
            let root = root_path.clone();
            let Ok(mut value) = running.lock() else {
                let _ = results.send(serde_json::json!({"error":"Could not update the mod check status."}));
                return false;
            };
            if *value {
                return false;
            }
            *value = true;
            drop(value);
            std::thread::spawn(move || {
                let result = crate::run_request(
                    &root,
                    &serde_json::json!({"command":"check-mod-updates","manual":true}),
                );
                if let Ok(mut value) = running.lock() {
                    *value = false;
                }
                let message = match result {
                    Ok(value) => value,
                    Err(error) => serde_json::json!({"error":error}),
                };
                let _ = results.send(message);
            });
            Ok(())
        }
        "open-url" => command["url"]
            .as_str()
            .ok_or_else(|| "updater link request is missing its URL".to_owned())
            .and_then(open_url)
            .map(|()| serde_json::json!({"opened":true}))
            .map(|_| ()),
        _ => {
            let request = updater_request(&root_path, command);
            crate::run_request(&root_path, &request).map(|_| ())
        }
    };
    match result {
        Ok(()) => {
            action == "start-update-queue"
        }
        Err(error) => {
            let _ = results.send(serde_json::json!({"error":error}));
            false
        }
    }
}

fn updater_request(root: &Path, command: &serde_json::Value) -> serde_json::Value {
    let mut request = command.clone();
    match command["command"].as_str().unwrap_or_default() {
        "queue-system-update" => request["command"] = "enqueue-system-update".into(),
        "remove-update-queue-item" => {
            request["id"] = command["key"].clone();
        }
        "search-catalog" | "install-mod" | "update-mod" => {
            let config = shroudforge_loader_config::config::read_loader(root).unwrap_or_default();
            request["provider"] =
                config
                    .pointer("/catalog/provider")
                    .cloned()
                    .unwrap_or_else(|| {
                        serde_json::to_value(
                            crate::provider_config::CatalogProviderConfig::default(),
                        )
                        .unwrap_or_default()
                    });
        }
        _ => {}
    }
    request
}

fn open_url(url: &str) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;

    if !url.starts_with("https://") && url != "mailto:support@shroudedit.com" {
        return Err("Only HTTPS links and the ShroudEdit support address can be opened.".into());
    }
    let operation = std::ffi::OsStr::new("open")
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let target = std::ffi::OsStr::new(url)
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            operation.as_ptr(),
            target.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1,
        ) as isize
    };
    if result <= 32 {
        return Err(format!(
            "Windows could not open the link (ShellExecuteW returned {result})."
        ));
    }
    Ok(())
}

fn read_snapshot(
    root: &Path,
    check_state: &Arc<Mutex<String>>,
    mod_check_running: &Arc<Mutex<bool>>,
) -> serde_json::Value {
    let config = shroudforge_loader_config::config::read_loader(root).unwrap_or_default();
    let installed = crate::installed_release(root);
    let mut release = crate::release_state::ReleaseState::idle(
        installed,
        shroudforge_loader_config::paths::updates_dir(root)
            .join("pending.ready")
            .is_file(),
    );
    let state = Arc::new(Mutex::new(release.clone()));
    crate::restore_checked_release::restore_persisted_update_result_from_config(&config, &state);
    if let Ok(value) = state.lock() {
        release = value.clone();
    }
    if check_state.lock().is_ok_and(|value| *value == "checking") {
        release.state = "checking".into();
        release.message = Some("Checking GitHub releases…".into());
    }
    let status_path =
        shroudforge_loader_config::paths::updates_dir(root).join("updater-status.json");
    if let Ok(bytes) = std::fs::read(status_path)
        && let Ok(status) = serde_json::from_slice::<serde_json::Value>(&bytes)
        && let Some(state) = status["status"].as_str()
        && matches!(
            state,
            "downloading"
                | "verifying"
                | "extracting"
                | "waitingForGame"
                | "installing"
                | "error"
                | "installed"
        )
    {
        release.state = state.to_owned();
        release.message = status["message"].as_str().map(str::to_owned);
        release.downloaded_bytes = status["downloadedBytes"].as_u64().unwrap_or_default();
        release.total_bytes = status["totalBytes"].as_u64();
        release.bytes_per_second = status["bytesPerSecond"].as_u64();
    }
    if release.update_available
        && let Some(notes) = config
            .pointer("/modules/updates/system/latestRelease/message")
            .and_then(serde_json::Value::as_str)
    {
        release.message = Some(notes.to_owned());
    }
    serde_json::json!({
        "root": root.to_string_lossy(),
        "version": release.current_version,
        "release": release,
        "queue": crate::read_update_queue(root),
        "modUpdates": config.pointer("/modules/updates/mods/latestUpdates").cloned().unwrap_or_else(|| serde_json::json!([])),
        "modCheckResult": config.pointer("/modules/updates/mods/lastCheckResult").cloned(),
        "modCheckRunning": mod_check_running.lock().is_ok_and(|value| *value),
        "updatesEnabled": config.pointer("/modules/updates/enabled").and_then(serde_json::Value::as_bool).unwrap_or(false),
        "locale": config.pointer("/general/locale").and_then(serde_json::Value::as_str).unwrap_or("de")
    })
}

fn send_event(webview: &wry::WebView, name: &str, value: &serde_json::Value) {
    if let Ok(json) = serde_json::to_string(value) {
        let script = format!("window.{name}&&window.{name}({json})");
        let _ = webview.evaluate_script(&script);
    }
}

fn installed_root() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    let module = executable.parent()?;
    let root = module.parent()?;
    is_game_root(root).then(|| root.to_path_buf())
}

fn is_game_root(root: &Path) -> bool {
    root.join("enshrouded.exe").is_file()
        || root.join("enshrouded_server.exe").is_file()
        || root.join("enshrouded_server").is_file()
        || root.join("enshrouded_server.x86_64").is_file()
        || root.join("shroudforge/version.json").is_file()
}
