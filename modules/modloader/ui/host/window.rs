use super::apply_language_change::apply_language_change;
use super::apply_loader_setting::apply_loader_setting;
use super::apply_loader_settings::apply_loader_settings;
use super::apply_mod_enabled::apply_mod_enabled;
use super::apply_mod_settings::apply_mod_settings;
use super::cancel_update::cancel_update;
use super::capture_game_file_originals::capture_game_file_originals;
use super::change_notice_read_state::change_notice_read_state;
use super::change_storage_folder::change_storage_folder;
use super::clear_update_queue::clear_update_queue;
use super::copy_activity_log::copy_activity_log;
use super::log_configuration_changes::log_configuration_check_transitions;
use super::log_player_name_detection::log_player_name_detection;
use super::open_requested_folder::open_requested_folder;
use super::open_requested_link::open_requested_link;
use super::parse_ui_command::Command;
use super::parse_ui_command::UiCommand;
use super::parse_ui_command::parse_ui_command;
use super::queue_system_update::queue_system_update;
use super::read_host_arguments::Arguments;
use super::read_host_arguments::arguments;
use super::read_installed_mods::read_mods;
use super::read_ui_configuration::default_refresh;
use super::read_ui_configuration::default_toggle_key;
use super::read_ui_configuration::effective_provider;
use super::read_ui_configuration::live_settings_revision;
use super::read_ui_configuration::load_module_config;
use super::read_ui_configuration::load_user_config;
use super::read_ui_configuration::read_central_config;
use super::read_ui_snapshot::read_ui_snapshot;
use super::read_window_focus::foreground_is_shroudforge_window;
use super::read_window_focus::foreground_process;
use super::read_window_focus::open_event;
use super::refresh_activity_log::refresh_activity_log;
use super::refresh_installed_mod::refresh_installed_mod;
use super::reload_loader_settings::reload_loader_settings;
use super::remove_requested_mod::remove_requested_mod;
use super::remove_update_queue_item::remove_update_queue_item;
use super::report_async_result::report_async_result;
use super::report_command_result::report_command_result;
use super::request_diagnostics::request_diagnostics;
use super::request_mod_action::request_mod_action;
use super::request_module_visibility::request_module_visibility;
use super::restore_game_files::restore_game_files;
use super::select_update_queue_items::select_update_queue_items;
use super::stage_system_update::stage_system_update;
use super::start_update_queue::start_update_queue;
use super::ui_snapshot_types::ModuleConfig;
use super::{
    dispatch_control_actions::dispatch_control_actions,
    log_configuration_errors::log_configuration_errors, watch_toggle_key::watch_toggle_key,
};
use sha2::Digest;
use sha2::Sha256;
use shroudforge_logging::write_activity;
use shroudforge_updater::catalog_search_state::CatalogState;
use shroudforge_updater::command_result::AsyncUiResult;
use shroudforge_updater::installed_release;
use shroudforge_updater::provider_config::default_system_provider;
use shroudforge_updater::release_state::ReleaseState;
use shroudforge_updater::start_catalog_search;
use shroudforge_updater::start_mod_install;
use shroudforge_updater::start_mod_update;
use shroudforge_updater::start_update_check;
use shroudforge_window_management::WindowDefaults;
use shroudforge_window_management::apply_window_geometry;
use shroudforge_window_management::current_window_geometry;
use shroudforge_window_management::save_window_geometry;
use std::fs;
use std::process::Child;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;
use tao::dpi::LogicalSize;
use tao::event::Event;
use tao::event::StartCause;
use tao::event::WindowEvent;
use tao::event_loop::ControlFlow;
use tao::event_loop::EventLoopBuilder;
use tao::platform::windows::WindowBuilderExtWindows;
use tao::window::WindowBuilder;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::ERROR_ALREADY_EXISTS;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::SetLastError;
use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::System::Threading::GetCurrentProcessId;
use windows_sys::Win32::System::Threading::WaitForSingleObject;
use wry::WebContext;
use wry::WebViewBuilder;

pub(crate) const DEFAULT_TOGGLE_KEY: u32 = 120; // F9

#[derive(Clone, Copy)]
pub(crate) enum UiEvent {
    Toggle,
}

pub(crate) fn run() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = arguments()?;
    shroudforge_loader_config::config::initialize_loader_config(&arguments.root)?;
    run_with_arguments(arguments)
}

pub(crate) fn run_with_arguments(arguments: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let _updater_window_mutex = if arguments.updater_window {
        let root = arguments
            .root
            .canonicalize()
            .unwrap_or_else(|_| arguments.root.clone());
        let name = format!(
            "Local\\ShroudForgeUpdaterWindow_{:x}",
            Sha256::digest(root.to_string_lossy().to_ascii_lowercase().as_bytes())
        );
        let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            SetLastError(0);
        }
        let mutex = unsafe { CreateMutexW(std::ptr::null_mut(), 0, wide.as_ptr()) };
        if mutex.is_null() {
            return Err(std::io::Error::last_os_error().into());
        }
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            unsafe {
                CloseHandle(mutex);
            }
            shroudforge_loader_config::config::request_window_visibility(
                &arguments.root,
                "modloaderUi",
                true,
            )?;
            return Ok(());
        }
        Some(mutex)
    } else {
        None
    };
    let mut module_config = load_module_config(&arguments.root).unwrap_or_else(|_| ModuleConfig {
        toggle_key: default_toggle_key(),
        refresh_milliseconds: default_refresh(),
        updates_enabled: false,
        system_update_provider: default_system_provider(),
    });
    let user_config = load_user_config(&arguments.root);
    let provider = effective_provider(&user_config);
    let installed = installed_release(&arguments.root);
    let staged = shroudforge_loader_config::paths::updates_dir(&arguments.root)
        .join("pending.ready")
        .is_file();
    let release_state = Arc::new(Mutex::new(ReleaseState::idle(installed, staged)));
    let catalog_state = Arc::new(Mutex::new(CatalogState {
        state: "idle".into(),
        ..CatalogState::default()
    }));
    let stop_event = arguments
        .stop_name
        .as_deref()
        .map(open_event)
        .unwrap_or(std::ptr::null_mut());
    let event_loop = EventLoopBuilder::<UiEvent>::with_user_event().build();
    let modloader_height = event_loop
        .primary_monitor()
        .map(|monitor| {
            ((monitor.size().height as f64 / monitor.scale_factor()) * 0.92)
                .min(988.0)
                .max(560.0)
        })
        .unwrap_or(988.0);
    let (icon_rgba, icon_width, icon_height) = shroudforge_ui_resources::rgba()?;
    let icon = tao::window::Icon::from_rgba(icon_rgba, icon_width, icon_height)?;
    let (title, inner_width, inner_height, min_width, min_height) = if arguments.updater_window {
        shroudforge_updater::ui::host::window::window_geometry()
    } else {
        (
            "ShroudForge | Modloader",
            1180.0,
            modloader_height,
            860.0,
            560.0,
        )
    };
    let window_defaults = WindowDefaults {
        width: inner_width,
        height: inner_height,
        minimum_width: min_width,
        minimum_height: min_height,
    };
    let window = WindowBuilder::new()
        .with_window_icon(Some(icon.clone()))
        .with_taskbar_icon(Some(icon))
        .with_title(title)
        .with_decorations(false)
        .with_always_on_top(true)
        .with_resizable(true)
        .with_visible(arguments.desktop)
        .with_inner_size(LogicalSize::new(inner_width, inner_height))
        .with_min_inner_size(LogicalSize::new(min_width, min_height))
        .build(&event_loop)?;

    let window_config_module = if arguments.updater_window {
        "updates"
    } else {
        "modloaderUi"
    };
    let mut saved_position = read_central_config(&arguments.root)["modules"][window_config_module]
        ["window"]["position"]
        .clone();
    let mut effective_window_geometry =
        apply_window_geometry(&window, &saved_position, false, window_defaults);
    let (sender, receiver) = mpsc::channel();
    let hotkey_proxy = event_loop.create_proxy();
    let hotkey_key = Arc::new(AtomicU32::new(module_config.toggle_key));
    let hotkey_key_watcher = hotkey_key.clone();
    let game_pid = arguments.game_pid;
    let desktop = arguments.desktop;
    watch_toggle_key(hotkey_proxy, hotkey_key_watcher, game_pid, desktop);
    let control_sender = sender.clone();
    let (async_sender, async_receiver) = mpsc::channel::<AsyncUiResult>();
    let handler = move |message: wry::http::Request<String>| {
        let Ok(value) = serde_json::from_str::<UiCommand>(message.body()) else {
            return;
        };
        let Some(command) = parse_ui_command(
            value,
            arguments.desktop,
            arguments.updater_window,
            arguments.server,
        ) else {
            return;
        };
        let _ = sender.send(command);
    };
    // WebView2 locks a user-data directory while a WebContext is alive.
    // The Modloader, Debug Console, and World Editor windows run side by
    // side, so each host must own a distinct profile directory.
    let profile_name = if arguments.updater_window {
        "updater"
    } else {
        "modloader-ui"
    };
    let profile =
        shroudforge_loader_config::paths::webview_profile(&arguments.root).join(profile_name);
    fs::create_dir_all(&profile)?;
    let mut web_context = WebContext::new(Some(profile));
    let html = if arguments.updater_window {
        shroudforge_updater::ui::host::window::launch_html(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../build/ui/modloader/index.html"
        )))
    } else {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../build/ui/modloader/index.html"
        ))
        .to_owned()
    };
    let webview = WebViewBuilder::new_with_web_context(&mut web_context)
        .with_html(html)
        .with_ipc_handler(handler)
        .build(&window)?;

    let mut shown = arguments.desktop;
    let mut previous_configuration_errors = std::collections::HashSet::<String>::new();
    let mut previous_player_name_detection = None;
    let mut previous_configuration_checks =
        std::collections::BTreeMap::<String, (String, String)>::new();
    let mut previous_asset_status = std::collections::BTreeMap::<String, (String, String)>::new();
    let mut previous_mod_statuses = std::collections::BTreeMap::<String, (String, String)>::new();
    let initial_window_state = shroudforge_loader_config::config::window_state(&arguments.root);
    let mut visibility_request_id = initial_window_state["modloaderUi"]["requestId"]
        .as_u64()
        .unwrap_or(0);
    let _ = shroudforge_loader_config::config::publish_window_visibility(
        &arguments.root,
        "modloaderUi",
        shown,
    );
    if read_central_config(&arguments.root)["modules"]["debugConsole"]["enabled"] == false {
        let _ = shroudforge_loader_config::config::publish_window_visibility(
            &arguments.root,
            "debugConsole",
            false,
        );
    }
    let mut next_refresh = Instant::now();
    let mut next_visibility_poll = Instant::now();
    let mut next_window_state_refresh = Instant::now();
    let mut next_control_poll = Instant::now();
    let mut next_update_check =
        if module_config.updates_enabled && module_config.system_update_provider.enabled {
            Instant::now() + shroudforge_updater::system_update_check_delay(&arguments.root)
        } else {
            Instant::now() + Duration::from_secs(86400)
        };
    let mut active_provider = provider;
    let mut system_provider = module_config.system_update_provider.clone();
    let mut desktop_debug_console: Option<Child> = None;
    let mut recovered_control_actions = false;
    let mut loaded_settings_revision = live_settings_revision(&arguments.root);
    let mut pending_window_geometry_save: Option<Instant> = None;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(50));
        match event {
            Event::NewEvents(StartCause::ResumeTimeReached { .. })
            | Event::NewEvents(StartCause::Init) => {
                if pending_window_geometry_save
                    .is_some_and(|deadline| Instant::now() >= deadline)
                {
                    pending_window_geometry_save = None;
                    if current_window_geometry(&window)
                        .is_some_and(|geometry| geometry != effective_window_geometry)
                    {
                        if let Ok(Some(geometry)) = save_window_geometry(
                            &arguments.root,
                            window_config_module,
                            &window,
                        ) {
                            saved_position = geometry.clone();
                            effective_window_geometry = geometry;
                        }
                    }
                }
                if desktop_debug_console
                    .as_mut()
                    .is_some_and(|child| child.try_wait().ok().flatten().is_some())
                {
                    desktop_debug_console = None;
                }
                if (!stop_event.is_null()
                    && unsafe { WaitForSingleObject(stop_event, 0) } == WAIT_OBJECT_0)
                    || (!arguments.game_process.is_null()
                        && unsafe { WaitForSingleObject(arguments.game_process, 0) }
                            == WAIT_OBJECT_0)
                {
                    let _ = shroudforge_loader_config::config::publish_window_visibility(
                        &arguments.root,
                        "modloaderUi",
                        false,
                    );
                    if !stop_event.is_null() {
                        unsafe { CloseHandle(stop_event) };
                    }
                    if !arguments.game_process.is_null() {
                        unsafe { CloseHandle(arguments.game_process) };
                    }
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                if Instant::now() >= next_control_poll {
                    let current_settings_revision = live_settings_revision(&arguments.root);
                    if current_settings_revision != loaded_settings_revision {
                        if let Ok(updated_module_config) = load_module_config(&arguments.root) {
                            let updates_enabled = updated_module_config.updates_enabled;
                            let updates_enabled_changed =
                                updates_enabled != module_config.updates_enabled;
                            let toggle_key = updated_module_config.toggle_key;
                            active_provider =
                                effective_provider(&load_user_config(&arguments.root));
                            let updated_system_provider =
                                updated_module_config.system_update_provider.clone();
                            if updates_enabled_changed
                                || serde_json::to_value(&updated_system_provider).ok()
                                != serde_json::to_value(&system_provider).ok()
                            {
                                system_provider = updated_system_provider;
                                next_update_check = if updates_enabled && system_provider.enabled {
                                    Instant::now()
                                        + shroudforge_updater::system_update_check_delay(
                                            &arguments.root,
                                        )
                                } else {
                                    Instant::now() + Duration::from_secs(86400)
                                };
                            }
                            module_config = updated_module_config;
                            hotkey_key.store(toggle_key, Ordering::Relaxed);
                            if read_central_config(&arguments.root)["modules"]["debugConsole"]["enabled"] == false {
                                let _ = shroudforge_loader_config::config::publish_window_visibility(
                                    &arguments.root,
                                    "debugConsole",
                                    false,
                                );
                            }
                            if !arguments.desktop
                                && read_central_config(&arguments.root)["modules"]["modloaderUi"]["enabled"] == false
                            {
                                shown = false;
                                window.set_visible(false);
                                let _ = shroudforge_loader_config::config::publish_window_visibility(
                                    &arguments.root,
                                    "modloaderUi",
                                    false,
                                );
                            }
                        }
                        loaded_settings_revision = current_settings_revision;
                    }
                    dispatch_control_actions(&arguments, &control_sender, &mut recovered_control_actions);
                    next_control_poll = Instant::now() + Duration::from_millis(250);
                }
                while let Ok(command) = receiver.try_recv() {
                    match command {
                        Command::MinimizeWindow => window.set_minimized(true),
                        Command::ToggleMaximizeWindow => {
                            window.set_maximized(!window.is_maximized());
                        }
                        Command::ResetWindowGeometry(request_id) => {
                            let result =
                                shroudforge_window_management::reset_window_geometry(&arguments.root);
                            report_command_result(
                                &arguments.root,
                                &webview,
                                &request_id,
                                "Settings",
                                "Reset window positions and sizes",
                                result,
                                "settings.windowGeometry.resetSuccess",
                                None,
                                false,
                            );
                            next_refresh = Instant::now();
                        }
                        Command::Hide(request_id) => {
                            let result = shroudforge_loader_config::config::publish_window_visibility(
                                &arguments.root,
                                "modloaderUi",
                                false,
                            );
                            report_command_result(
                                &arguments.root,
                                &webview,
                                &request_id,
                                "Modloader UI",
                                "Hide window",
                                result,
                                "settings.visibilityRequested",
                                None,
                                false,
                            );
                            if arguments.desktop {
                                *control_flow = ControlFlow::Exit;
                            } else {
                                shown = false;
                                window.set_visible(false);
                            }
                        }
                        Command::Drag(request_id) => {
                            let result = window.drag_window().map_err(|error| error.to_string());
                            report_command_result(
                                &arguments.root,
                                &webview,
                                &request_id,
                                "Modloader UI",
                                "Move window",
                                result,
                                "settings.visibilityRequested",
                                None,
                                false,
                            );
                        }
                        Command::Refresh(request_id) => refresh_activity_log(
                            request_id,
                            &arguments,
                            &webview,
                            &mut next_refresh,
                        ),
                        Command::CopyLogs(request_id) => copy_activity_log(
                            request_id,
                            &arguments,
                            &webview,
                        ),
                        Command::RefreshMod(id, request_id) => refresh_installed_mod(
                            id,
                            request_id,
                            &arguments,
                            &webview,
                            &mut next_refresh,
                        ),
                        Command::ReloadSettings(request_id) => reload_loader_settings(
                            request_id,
                            &arguments,
                            &webview,
                            &mut next_refresh,
                        ),
                        Command::CheckUpdates(request_id) => {
                            start_update_check(
                                arguments.root.clone(),
                                system_provider.clone(),
                                release_state.clone(),
                                async_sender.clone(),
                                Some(request_id),
                            );
                            next_update_check = Instant::now()
                                + Duration::from_secs(
                                    system_provider.check_minutes.clamp(5, 1440) * 60,
                                );
                        }
                        Command::StageUpdate(request_id, wait_for_game) => stage_system_update(
                            request_id,
                            wait_for_game,
                            &arguments,
                            &release_state,
                            &async_sender,
                        ),
                        Command::QueueSystemUpdate(request_id) => queue_system_update(
                            request_id,
                            &arguments,
                            &webview,
                            &release_state,
                            &mut next_refresh,
                        ),
                        Command::SelectUpdateQueueItems(ids, selected, request_id) => select_update_queue_items(
                            ids,
                            selected,
                            request_id,
                            &arguments,
                            &webview,
                            &mut next_refresh,
                        ),
                        Command::RemoveUpdateQueueItem(id, request_id) => remove_update_queue_item(
                            id,
                            request_id,
                            &arguments,
                            &webview,
                            &mut next_refresh,
                        ),
                        Command::StartUpdateQueue(ids, wait_for_game, show_window, request_id) => start_update_queue(
                            ids,
                            wait_for_game,
                            show_window,
                            request_id,
                            &arguments,
                            &webview,
                            &release_state,
                            &mut next_refresh,
                        ),
                        Command::ClearUpdateQueue(request_id) => {
                            if clear_update_queue(
                                request_id,
                                &arguments,
                                &webview,
                                &release_state,
                                &mut next_refresh,
                                control_flow,
                            ) {
                                return;
                            }
                        }
                        Command::CancelUpdate(request_id) => {
                            if cancel_update(
                                request_id,
                                &arguments,
                                &webview,
                                &release_state,
                                &mut next_refresh,
                                control_flow,
                            ) {
                                return;
                            }
                        }
                        Command::RestoreGamefiles(request_id) => restore_game_files(
                            request_id,
                            &arguments,
                            &webview,
                            &mut next_refresh,
                        ),
                        Command::CaptureGamefileOriginals(request_id) => capture_game_file_originals(
                            request_id,
                            &arguments,
                            &webview,
                            &mut next_refresh,
                        ),
                        Command::SearchCatalog(query, request_id) => start_catalog_search(
                            arguments.root.clone(),
                            active_provider.clone(),
                            query,
                            catalog_state.clone(),
                            async_sender.clone(),
                            request_id,
                        ),
                        Command::InstallMod(project_id, title, request_id) => start_mod_install(
                            arguments.root.clone(),
                            active_provider.clone(),
                            project_id,
                            title,
                            async_sender.clone(),
                            request_id,
                        ),
                        Command::UpdateMod(mod_id, title, request_id) => start_mod_update(
                            arguments.root.clone(),
                            active_provider.clone(),
                            mod_id,
                            title,
                            async_sender.clone(),
                            request_id,
                        ),
                        Command::SaveLanguage(locale, request_id) => apply_language_change(
                            locale,
                            request_id,
                            &arguments,
                            &webview,
                            &mut next_refresh,
                        ),
                        Command::OpenUrl(url, request_id) => open_requested_link(
                            url,
                            request_id,
                            &arguments,
                            &webview,
                        ),
                        Command::SetModEnabled(id, enabled, revision, request_id) => apply_mod_enabled(
                            id,
                            enabled,
                            revision,
                            request_id,
                            &arguments,
                            &webview,
                            &mut next_refresh,
                        ),
                        Command::SaveModSettings(id, values, revision, request_id) => apply_mod_settings(
                            id,
                            values,
                            revision,
                            request_id,
                            &arguments,
                            &webview,
                            &mut next_refresh,
                        ),
                        Command::SaveSettings(settings, request_id) => apply_loader_settings(
                            settings,
                            request_id,
                            &arguments,
                            &webview,
                            &mut active_provider,
                            &mut next_refresh,
                        ),
                        Command::SaveSetting(scope, module, key, setting, request_id) => apply_loader_setting(
                            scope,
                            module,
                            key,
                            setting,
                            request_id,
                            &arguments,
                            &webview,
                            &mut next_refresh,
                        ),
                        Command::BrowsePath(key, request_id) => change_storage_folder(
                            key,
                            request_id,
                            &arguments,
                            &webview,
                            &mut next_refresh,
                        ),
                        Command::OpenPath(key, request_id) => open_requested_folder(
                            key,
                            request_id,
                            &arguments,
                            &webview,
                        ),
                        Command::SetWindowVisibility(module, visible, request_id) => request_module_visibility(
                            module,
                            visible,
                            request_id,
                            &arguments,
                            &webview,
                            &mut desktop_debug_console,
                            &mut next_refresh,
                        ),
                        Command::RunModAction(id, action, request_id) => request_mod_action(
                            id,
                            action,
                            request_id,
                            &arguments,
                            &webview,
                            &mut next_refresh,
                        ),
                        Command::RemoveMod(id, request_id) => remove_requested_mod(
                            id,
                            request_id,
                            &arguments,
                            &webview,
                            &mut next_refresh,
                        ),
                        Command::SetNewsRead(ids, read, request_id) => change_notice_read_state(
                            ids,
                            read,
                            request_id,
                            &arguments,
                            &webview,
                            &mut next_refresh,
                        ),
                        Command::Diagnostics(action, job_id, request_id) => request_diagnostics(
                            action,
                            job_id,
                            request_id,
                            &arguments,
                            &webview,
                            &mut next_refresh,
                        ),
                        Command::UiReady => write_activity(
                            &arguments.root,
                            "modloader-ui",
                            "WebView rendered",
                            "Ready",
                            None,
                            "debug",
                        ),
                        Command::UiError(message) => write_activity(
                            &arguments.root,
                            "modloader-ui",
                            "WebView JavaScript error",
                            "Failed",
                            Some(&message),
                            "error",
                        ),
                    }
                }
                while let Ok(result) = async_receiver.try_recv() {
                    report_async_result(&arguments.root, &webview, result);
                    next_refresh = Instant::now();
                }

                if Instant::now() >= next_visibility_poll {
                    let window_state =
                        shroudforge_loader_config::config::window_state(&arguments.root);
                    let requested_id = window_state["modloaderUi"]["requestId"]
                        .as_u64()
                        .unwrap_or(0);
                    if requested_id != visibility_request_id {
                        visibility_request_id = requested_id;
                        shown = if arguments.server && !arguments.desktop {
                            false
                        } else {
                            window_state["modloaderUi"]["requestedVisible"]
                                .as_bool()
                                .unwrap_or(shown)
                        };
                        if shown {
                            window.set_minimized(false);
                        }
                        window.set_visible(shown);
                        if shown {
                            window.set_focus();
                            if arguments.updater_window {
                                next_update_check = Instant::now()
                                    + shroudforge_updater::system_update_check_delay(
                                        &arguments.root,
                                    );
                            }
                        }
                        let _ = shroudforge_loader_config::config::publish_window_visibility(
                            &arguments.root,
                            "modloaderUi",
                            shown,
                        );
                    }
                    next_visibility_poll = Instant::now() + Duration::from_millis(100);
                }
                if Instant::now() >= next_window_state_refresh {
                    let windows = shroudforge_loader_config::config::window_state(&arguments.root);
                    if let Ok(payload) = serde_json::to_string(&windows) {
                        let _ = webview.evaluate_script(&format!(
                            "window.__shroudforgeWindowState({payload});"
                        ));
                    }
                    next_window_state_refresh = Instant::now() + Duration::from_millis(250);
                }
                let foreground = foreground_process();
                let game_focused = arguments.desktop || foreground == arguments.game_pid;
                let ui_focused = foreground == unsafe { GetCurrentProcessId() }
                    || foreground_is_shroudforge_window();
                if !arguments.desktop && shown && !game_focused && !ui_focused {
                    window.set_visible(false);
                } else if shown && (game_focused || ui_focused) && !window.is_visible() {
                    window.set_visible(true);
                }

                if module_config.updates_enabled
                    && system_provider.enabled
                    && Instant::now() >= next_update_check
                {
                    start_update_check(
                        arguments.root.clone(),
                        system_provider.clone(),
                        release_state.clone(),
                        async_sender.clone(),
                        None,
                    );
                    next_update_check = Instant::now()
                        + Duration::from_secs(
                            system_provider.check_minutes.clamp(5, 1440) * 60,
                        );
                }
                if Instant::now() >= next_refresh {
                    let next_position = read_central_config(&arguments.root)["modules"]
                        [window_config_module]["window"]["position"]
                        .clone();
                    if next_position != saved_position {
                        pending_window_geometry_save = None;
                        effective_window_geometry = apply_window_geometry(
                            &window,
                            &next_position,
                            false,
                            window_defaults,
                        );
                        saved_position = next_position;
                    }
                    if let Ok(next) = load_module_config(&arguments.root) {
                        if next.updates_enabled != module_config.updates_enabled
                            || next.system_update_provider.enabled != system_provider.enabled
                            || next.system_update_provider.check_minutes
                                != system_provider.check_minutes
                            || next.system_update_provider.include_prereleases
                                != system_provider.include_prereleases
                        {
                            next_update_check = Instant::now()
                                + shroudforge_updater::system_update_check_delay(&arguments.root);
                        }
                        if !next.updates_enabled || !next.system_update_provider.enabled {
                            next_update_check = Instant::now() + Duration::from_secs(86400);
                        }
                        system_provider = next.system_update_provider.clone();
                        active_provider = effective_provider(&load_user_config(&arguments.root));
                        module_config = next;
                    }
                    next_refresh = Instant::now()
                        + Duration::from_millis(if arguments.updater_window {
                            350
                        } else {
                            module_config.refresh_milliseconds.clamp(250, 10_000)
                        });
                    let central = read_central_config(&arguments.root);
                    let mods = read_mods(&arguments.root);
                    let notification_mods: Vec<_> = mods.iter().map(|item| shroudforge_news::ModNotification {
                        id: &item.id, name: &item.name, version: &item.version, ecosystem: item.ecosystem, changelog: &item.changelog,
                    }).collect();
                    shroudforge_news::sync_mod_events(&arguments.root, &notification_mods);
                    let player_name_detection = shroudforge_player_name::detect_active_character();
                    let snapshot = read_ui_snapshot(
                        &arguments,
                        &active_provider,
                        &release_state,
                        &catalog_state,
                        central,
                        mods,
                        &player_name_detection,
                    );
                    log_player_name_detection(
                        &arguments.root,
                        &player_name_detection,
                        &mut previous_player_name_detection,
                    );
                    log_configuration_check_transitions(
                        &arguments.root,
                        &snapshot.configuration,
                        &mut previous_configuration_checks,
                        &mut previous_asset_status,
                        &mut previous_mod_statuses,
                    );
                    log_configuration_errors(&arguments.root, &snapshot, &mut previous_configuration_errors);
                    if let Ok(payload) = serde_json::to_string(&snapshot) {
                        let _ = webview.evaluate_script(&format!(
                            "window.__shroudforgeUpdate({payload});"
                        ));
                    }
                }
            }
            Event::UserEvent(UiEvent::Toggle) => {
                if !arguments.server
                    && (arguments.desktop
                        || read_central_config(&arguments.root)["modules"]["modloaderUi"]["enabled"]
                            != false)
                {
                    shown = !shown;
                    if shown {
                        window.set_minimized(false);
                    }
                    window.set_visible(shown);
                    let _ = shroudforge_loader_config::config::publish_window_visibility(
                        &arguments.root,
                        "modloaderUi",
                        shown,
                    );
                    if shown {
                        next_refresh = Instant::now();
                        window.set_focus();
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Moved(_),
                ..
            }
            | Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                if !window.is_maximized() && !window.is_minimized() {
                    pending_window_geometry_save = current_window_geometry(&window)
                        .filter(|geometry| geometry != &effective_window_geometry)
                        .map(|_| Instant::now() + Duration::from_millis(250));
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                if arguments.desktop {
                    let _ = shroudforge_loader_config::config::publish_window_visibility(
                        &arguments.root,
                        "modloaderUi",
                        false,
                    );
                    *control_flow = ControlFlow::Exit;
                } else {
                    shown = false;
                    window.set_visible(false);
                    let _ = shroudforge_loader_config::config::publish_window_visibility(
                        &arguments.root,
                        "modloaderUi",
                        shown,
                    );
                }
            }
            Event::LoopDestroyed => {
                if current_window_geometry(&window)
                    .is_some_and(|geometry| geometry != effective_window_geometry)
                {
                    let _ = save_window_geometry(
                        &arguments.root,
                        window_config_module,
                        &window,
                    );
                }
            }
            _ => {}
        }
    });
}
