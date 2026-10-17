use super::position_console_window::position_window;
use super::read_console_arguments::Arguments;
use super::read_console_settings::Config;
use tao::event_loop::ControlFlow;

pub(crate) fn refresh_console_settings(
    arguments: &Arguments,
    window: &tao::window::Window,
    preferences: &mut serde_json::Value,
    config: &mut Config,
    user_enabled: &mut bool,
    last_config_error: &mut Option<String>,
    control_flow: &mut ControlFlow,
) -> Option<String> {
    let mut minimum_level = "INFO".to_owned();
    match shroudforge_loader_config::config::read_loader(&arguments.root) {
        Ok(value) => {
            if last_config_error.take().is_some() {
                let _ = shroudforge_logging::append(
                    &arguments.root,
                    'I',
                    "debug-console",
                    "Debug Console configuration is readable again.",
                );
            }
            minimum_level = value
                .pointer("/logging/minimumLevel")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("INFO")
                .to_owned();
            let next_preferences = value["modules"]["debugConsole"].clone();
            if next_preferences["window"]["position"] != preferences["window"]["position"] {
                position_window(&window, &next_preferences["window"]["position"], true);
            }
            *preferences = next_preferences;
            *user_enabled = preferences["enabled"] != false;
            if let Ok(next) = serde_json::from_value(preferences.clone()) {
                *config = next;
            }
            if preferences["enabled"] == false && !arguments.startup_watch {
                let _ = shroudforge_loader_config::config::publish_window_visibility(
                    &arguments.root,
                    "debugConsole",
                    false,
                );
                *control_flow = ControlFlow::Exit;
                return None;
            }
        }
        Err(error) => {
            eprintln!("debug configuration: {error}");
            let message = error.to_string();
            let level = if last_config_error.as_deref() == Some(message.as_str()) {
                'D'
            } else {
                'W'
            };
            let _ = shroudforge_logging::append(
                &arguments.root,
                level,
                "debug-console",
                &format!(
                    "Could not read Debug Console configuration, using the last available settings. {message}"
                ),
            );
            *last_config_error = Some(message);
        }
    }
    Some(minimum_level)
}
