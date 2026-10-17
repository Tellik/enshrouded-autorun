use super::read_host_arguments::Arguments;
use super::report_command_result::report_command_result;
use shroudforge_package::set_mod_enabled;
use std::time::Instant;

pub(crate) fn apply_mod_enabled(
    id: String,
    enabled: bool,
    revision: String,
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    next_refresh: &mut Instant,
) {
    let result = set_mod_enabled(&arguments.root, &id, enabled, &revision);
    let action = if enabled { "Enable mod" } else { "Disable mod" };
    let message = if enabled {
        "mod.enabledToast"
    } else {
        "mod.disabledToast"
    };
    report_command_result(
        &arguments.root,
        &webview,
        &request_id,
        &id,
        action,
        result,
        message,
        None,
        false,
    );
    *next_refresh = Instant::now();
}
