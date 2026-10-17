use super::read_host_arguments::Arguments;
use super::report_command_result::report_command_result;
use super::save_loader_setting::save_setting;
use std::time::Instant;

pub(crate) fn apply_loader_setting(
    scope: String,
    module: Option<String>,
    key: String,
    setting: serde_json::Value,
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    next_refresh: &mut Instant,
) {
    let previous_log_level = if scope == "logging" && key == "minimumLevel" {
        shroudforge_loader_config::config::read_loader(&arguments.root)
            .ok()
            .and_then(|value| {
                value
                    .pointer("/logging/minimumLevel")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned)
            })
    } else {
        None
    };
    let result = save_setting(&arguments.root, &scope, module.as_deref(), &key, &setting);
    let target = module.as_deref().map_or_else(
        || format!("{scope}.{key}"),
        |module| format!("{module}.{key}"),
    );
    let action = match (previous_log_level.as_deref(), setting.as_str()) {
        (Some(previous), Some(next)) if previous != next && result.is_ok() => {
            format!("Minimum log level changed from {previous} to {next}")
        }
        _ => format!("Save {target}"),
    };
    report_command_result(
        &arguments.root,
        &webview,
        &request_id,
        "Settings",
        &action,
        result,
        "settings.saved",
        Some("settings"),
        true,
    );
    *next_refresh = Instant::now();
}
