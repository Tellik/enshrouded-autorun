use super::notify_command_result::notify_command_result;
use super::read_host_arguments::Arguments;
use shroudforge_logging::write_activity;
use std::time::Instant;

pub(crate) fn reload_loader_settings(
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    next_refresh: &mut Instant,
) {
    write_activity(
        &arguments.root,
        "Settings",
        "Reload settings",
        "Succeeded",
        None,
        "success",
    );
    notify_command_result(
        &webview,
        &request_id,
        Ok(()),
        "settings.reloaded",
        None,
        false,
        None,
    );
    *next_refresh = Instant::now();
}
