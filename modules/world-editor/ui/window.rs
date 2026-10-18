use super::create_editor_window::create_editor_window;
use super::open_stop_event::open_event;
use super::parse_editor_message::Command;
use super::parse_editor_message::parse_editor_message;
use super::position_editor_window::position_window;
use super::read_editor_arguments::arguments;
use super::render_editor_document::render_editor_document;
use super::run_blueprint_action::run_blueprint_action;
use super::set_library_expanded::set_library_expanded;
use super::set_manager_open::set_manager_open;
use super::watch_editor_state::watch_editor_state;
use crate::blueprint_library::empty_state;
use crate::blueprint_library::view_state;
use crate::log_editor_action::log_world_editor;
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;
use tao::dpi::LogicalSize;
use tao::event::Event;
use tao::event::StartCause;
use tao::event::WindowEvent;
use tao::event_loop::ControlFlow;
use tao::event_loop::EventLoop;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
use windows_sys::Win32::System::Threading::GetCurrentProcessId;
use windows_sys::Win32::System::Threading::WaitForSingleObject;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_F1;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_F2;
use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
use wry::WebContext;
use wry::WebViewBuilder;

pub(crate) fn run_module() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = arguments()?;
    let config = shroudforge_loader_config::config::read_loader(&arguments.root)?;
    let preferences = config["modules"]["worldEditor"].clone();
    let toggle_key = preferences["toggleKey"].as_i64().unwrap_or(VK_F2 as i64) as i32;
    let refresh_ms = preferences["refreshMilliseconds"]
        .as_u64()
        .unwrap_or(400)
        .clamp(100, 5000);
    let initial_state = view_state(&arguments.root).unwrap_or_else(|_| empty_state());
    let initial_window_state = shroudforge_loader_config::config::window_state(&arguments.root);
    // Always start hidden for a new game session. A persisted open state must not
    // make the overlay appear before the player asks for it with F1–F8.
    let _ = shroudforge_loader_config::config::request_window_visibility(
        &arguments.root,
        "worldEditor",
        false,
    );
    let mut visibility_request_id = shroudforge_loader_config::config::window_state(
        &arguments.root,
    )["worldEditor"]["requestId"]
        .as_u64()
        .unwrap_or_else(|| {
            initial_window_state["worldEditor"]["requestId"]
                .as_u64()
                .unwrap_or(0)
        });
    let mut window_visible = false;
    let stop_event = arguments
        .stop_name
        .as_deref()
        .map(open_event)
        .unwrap_or(std::ptr::null_mut());
    let event_loop = EventLoop::new();
    let window = create_editor_window(&event_loop, &initial_state)?;
    position_window(
        &window,
        arguments.game_pid,
        initial_state.panel_width as f64,
        initial_state.panel_position_x,
        initial_state.panel_position_y,
    );
    log_world_editor(
        &arguments.root,
        'D',
        "ui",
        "window initialized with opaque ShroudForge panel and undecorated Tao window",
    );

    let (sender, receiver) = mpsc::channel();
    let handler = move |message: wry::http::Request<String>| {
        let Some(command) = parse_editor_message(message.body()) else {
            return;
        };
        let _ = sender.send(command);
    };
    let mut web_context = WebContext::new(Some(
        shroudforge_loader_config::paths::webview_profile(&arguments.root).join("world-editor"),
    ));
    let html = render_editor_document();
    let webview = WebViewBuilder::new_with_web_context(&mut web_context)
        .with_html(html)
        .with_ipc_handler(handler)
        .build(&window)?;
    // Keep the same native window and WebView setup as the Modloader UI.
    // The document paints its own opaque surface; per-window background and
    // DWM frame overrides caused this host to diverge from that working path.
    log_world_editor(
        &arguments.root,
        'D',
        "ui",
        "Window initialized with the Modloader-style undecorated Tao window. The document owns the opaque panel surface",
    );

    let mut panel_width = initial_state.panel_width;
    let mut panel_height = initial_state.panel_height;
    let mut panel_position_x = initial_state.panel_position_x;
    let mut panel_position_y = initial_state.panel_position_y;
    let mut manager_open = false;
    let mut library_expanded = false;
    let mut library_expanded_height = initial_state.panel_height;
    let mut module_enabled = preferences["enabled"] != false;
    let mut native_visible = false;
    let _ = shroudforge_loader_config::config::publish_window_visibility(
        &arguments.root,
        "worldEditor",
        module_enabled && window_visible,
    );
    let mut function_keys_down = [false; 8];
    let mut toggle_down = false;
    let mut next_visibility_poll = Instant::now();
    let mut next_settings_poll = Instant::now();
    // Keep disk enumeration and thumbnail reads off the window thread.
    // Bound the channel so slow rendering cannot accumulate snapshots.
    let state_updates = watch_editor_state(arguments.root.clone(), refresh_ms);
    let mut next_refresh = Instant::now();
    let mut last_payload = String::new();
    let mut last_ui_surface = String::new();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(35));
        match event {
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) | Event::NewEvents(StartCause::Init) => {
                if !stop_event.is_null() && unsafe { WaitForSingleObject(stop_event, 0) } == WAIT_OBJECT_0 {
                    let _ = shroudforge_loader_config::config::publish_window_visibility(&arguments.root, "worldEditor", false);
                    unsafe { CloseHandle(stop_event) };
                    if !arguments.game_process.is_null() { unsafe { CloseHandle(arguments.game_process) }; }
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                if !arguments.game_process.is_null() && unsafe { WaitForSingleObject(arguments.game_process, 0) } == WAIT_OBJECT_0 {
                    let _ = shroudforge_loader_config::config::publish_window_visibility(&arguments.root, "worldEditor", false);
                    if !stop_event.is_null() { unsafe { CloseHandle(stop_event) }; }
                    unsafe { CloseHandle(arguments.game_process) };
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                while let Ok(command) = receiver.try_recv() {
                    match command {
                        Command::Ready => {
                            next_refresh = Instant::now();
                            log_world_editor(&arguments.root, 'I', "ui", "World Editor window and interface are ready.");
                        }
                        Command::SurfaceChanged(surface) => {
                            if surface != last_ui_surface {
                                log_world_editor(&arguments.root, 'T', "ui-state", &surface);
                                last_ui_surface = surface;
                            }
                        }
                        Command::UiMessage { level, message } => {
                            if !message.trim().is_empty() {
                                log_world_editor(&arguments.root, level, "ui-message", &message);
                            }
                        }
                        Command::ScriptError(message) => {
                            log_world_editor(&arguments.root, 'E', "ui", &format!("World Editor interface script failed: {message}"));
                        }
                        Command::Hide => {
                            window_visible = false;
                            window.set_visible(false);
                            native_visible = false;
                            let _ = shroudforge_loader_config::config::request_window_visibility(
                                &arguments.root,
                                "worldEditor",
                                false,
                            );
                        }
                        Command::Minimize => window.set_minimized(true),
                        Command::OpenModSettings => {
                            if let Err(error) = shroudforge_loader_config::config::request_world_editor_module_settings(&arguments.root) {
                                log_world_editor(&arguments.root, 'E', "ui", &format!("Could not open module settings: {error}"));
                            }
                        }
                        Command::Drag => {}
                        Command::Manager(open) => set_manager_open(open, &mut manager_open, &window, &arguments, panel_width, panel_height, panel_position_x, panel_position_y, library_expanded, library_expanded_height),
                        Command::LibraryExpanded { expanded, height } => set_library_expanded(expanded, height, &mut library_expanded, &mut library_expanded_height, manager_open, &window, &arguments, panel_width, panel_height, panel_position_x, panel_position_y),
                        Command::Action { action, value } if module_enabled => run_blueprint_action(action, value, &arguments, &webview, &mut next_refresh),
                        Command::Action { .. } => {}

                    }
                }

                if Instant::now() >= next_visibility_poll {
                    let state = shroudforge_loader_config::config::window_state(&arguments.root);
                    let request_id = state["worldEditor"]["requestId"]
                        .as_u64()
                        .unwrap_or(visibility_request_id);
                    if request_id != visibility_request_id {
                        visibility_request_id = request_id;
                        window_visible = state["worldEditor"]["requestedVisible"]
                            .as_bool()
                            .unwrap_or(window_visible);
                        if window_visible {
                            let height = if manager_open {
                                560.0
                            } else if library_expanded {
                                library_expanded_height as f64
                            } else {
                                panel_height as f64
                            };
                            window.set_inner_size(LogicalSize::new(
                                panel_width as f64,
                                height,
                            ));
                        }
                        let _ = shroudforge_loader_config::config::publish_window_visibility(
                            &arguments.root,
                            "worldEditor",
                            module_enabled && window_visible,
                        );
                    }
                    next_visibility_poll = Instant::now() + Duration::from_millis(100);
                }

                if Instant::now() >= next_settings_poll {
                    if let Ok(config) =
                        shroudforge_loader_config::config::read_loader(&arguments.root)
                    {
                        let next_enabled = config["modules"]["worldEditor"]["enabled"] != false;
                        if next_enabled != module_enabled {
                            module_enabled = next_enabled;
                            if !module_enabled {
                                window_visible = false;
                                window.set_visible(false);
                                native_visible = false;
                                let _ = shroudforge_loader_config::config::request_window_visibility(
                                    &arguments.root,
                                    "worldEditor",
                                    false,
                                );
                            }
                            let _ = shroudforge_loader_config::config::publish_window_visibility(
                                &arguments.root,
                                "worldEditor",
                                module_enabled && window_visible,
                            );
                        }
                    }
                    next_settings_poll = Instant::now() + Duration::from_millis(300);
                }

                let foreground = unsafe { GetForegroundWindow() };
                let mut foreground_pid = 0;
                if !foreground.is_null() { unsafe { GetWindowThreadProcessId(foreground, &mut foreground_pid) }; }
                let is_game = arguments.desktop || foreground_pid == arguments.game_pid;
                let is_overlay = foreground_pid == unsafe { GetCurrentProcessId() };
                let mut function_key_pressed = [false; 8];
                for (index, was_down) in function_keys_down.iter_mut().enumerate() {
                    let down = unsafe { GetAsyncKeyState(VK_F1 as i32 + index as i32) } < 0;
                    function_key_pressed[index] = down && !*was_down;
                    *was_down = down;
                }
                let toggle = unsafe { GetAsyncKeyState(toggle_key) } < 0;
                let toggle_pressed = toggle && !toggle_down;
                if module_enabled && (is_game || is_overlay) {
                    let was_visible = window_visible;
                    if toggle_pressed || function_key_pressed[1] {
                        window_visible = !window_visible;
                        if window_visible {
                            let height = if manager_open {
                                560.0
                            } else if library_expanded {
                                library_expanded_height as f64
                            } else {
                                panel_height as f64
                            };
                            window.set_inner_size(LogicalSize::new(
                                panel_width as f64,
                                height,
                            ));
                        }
                    }
                    if was_visible != window_visible {
                        let _ = shroudforge_loader_config::config::request_window_visibility(
                            &arguments.root,
                            "worldEditor",
                            window_visible,
                        );
                    }
                }
                toggle_down = toggle;
                let should_show = module_enabled && window_visible && (is_game || is_overlay);
                if should_show != native_visible {
                    native_visible = should_show;
                    if native_visible {
                        window.set_minimized(false);
                    }
                    window.set_visible(native_visible);
                    log_world_editor(
                        &arguments.root,
                        'D',
                        "ui",
                        if native_visible { "World Editor window shown." } else { "World Editor window hidden." },
                    );
                }
                if Instant::now() >= next_refresh {
                    if let Some(state) = state_updates.try_iter().last() {
                        if let Ok(payload) = serde_json::to_string(&state) {
                            let state_changed = payload != last_payload;
                            if state_changed {
                                if state.panel_width != panel_width
                                    || state.panel_height != panel_height
                                    || state.panel_position_x != panel_position_x
                                    || state.panel_position_y != panel_position_y
                                {
                                    panel_width = state.panel_width;
                                    panel_height = state.panel_height;
                                    panel_position_x = state.panel_position_x;
                                    panel_position_y = state.panel_position_y;
                                    let height = if manager_open {
                                        560.0
                                    } else if library_expanded {
                                        library_expanded_height as f64
                                    } else {
                                        panel_height as f64
                                    };
                                    window.set_inner_size(LogicalSize::new(
                                        panel_width as f64,
                                        height,
                                    ));
                                    position_window(
                                        &window,
                                        arguments.game_pid,
                                        panel_width as f64,
                                        panel_position_x,
                                        panel_position_y,
                                    );
                                }
                                last_payload = payload.clone();
                            }
                            let script = super::fallback::render_script(&last_payload);
                            if let Err(error) = webview.evaluate_script(&script) {
                                log_world_editor(
                                    &arguments.root,
                                    'E',
                                    "ui",
                                    &format!("Could not send the visible interface state to the window: {error}"),
                                );
                            }
                        }
                    }
                    next_refresh = Instant::now() + Duration::from_millis(refresh_ms);
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                window_visible = false;
                window.set_visible(false);
                native_visible = false;
                let _ = shroudforge_loader_config::config::request_window_visibility(
                    &arguments.root,
                    "worldEditor",
                    false,
                );
            }
            _ => {}
        }
    });
}
