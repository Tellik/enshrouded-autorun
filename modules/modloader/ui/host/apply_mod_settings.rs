use super::read_host_arguments::Arguments;
use super::report_command_result::report_command_result;
use shroudforge_package::save_mod_settings;
use std::time::Instant;

pub(crate) fn apply_mod_settings(
    id: String,
    values: serde_json::Value,
    revision: String,
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    next_refresh: &mut Instant,
) {
    let result = save_mod_settings(&arguments.root, &id, &values, &revision);
    report_command_result(
        &arguments.root,
        &webview,
        &request_id,
        &id,
        "Save mod settings",
        result,
        "settings.saved",
        None,
        false,
    );
    *next_refresh = Instant::now();
}
