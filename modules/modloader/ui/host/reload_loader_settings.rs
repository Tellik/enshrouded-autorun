use super::notify_command_result::notify_command_result;
use super::read_host_arguments::Arguments;
use shroudforge_logging::{activity_log_level, write_activity_at_level};
use std::time::Instant;

pub(crate) fn reload_loader_settings(
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    next_refresh: &mut Instant,
) {
    let level = activity_log_level("Reload settings", "success");
    write_activity_at_level(
        &arguments.root,
        "Settings",
        "Reload settings",
        "Succeeded",
        None,
        level,
    );
    notify_command_result(
        &webview,
        &request_id,
        Ok(()),
        "settings.reloaded",
        level,
        None,
        false,
        None,
    );
    *next_refresh = Instant::now();
}
