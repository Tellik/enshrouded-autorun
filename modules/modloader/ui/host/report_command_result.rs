use super::mark_control_action::mark_control_action;
use super::notify_command_result::notify_command_result;
use shroudforge_logging::{activity_log_level, write_activity_at_level};
use shroudforge_package::package_lookup::mod_ecosystem;
use std::path::Path;

pub(crate) fn report_command_result(
    root: &Path,
    webview: &wry::WebView,
    request_id: &str,
    source: &str,
    action: &str,
    result: Result<(), String>,
    success_message: &str,
    batch_key: Option<&str>,
    restore_settings: bool,
) {
    match &result {
        Ok(()) => mark_control_action(root, request_id, "succeeded", success_message),
        Err(error) => mark_control_action(root, request_id, "failed", error),
    }
    let (activity_result, details, outcome) = match &result {
        Ok(()) => ("Succeeded", None, "success"),
        Err(error) => ("Failed", Some(error.as_str()), "error"),
    };
    let log_level = activity_log_level(action, outcome);
    write_activity_at_level(root, source, action, activity_result, details, log_level);
    let ecosystem = mod_ecosystem(root, source);
    notify_command_result(
        webview,
        request_id,
        result,
        success_message,
        log_level,
        batch_key,
        restore_settings,
        ecosystem,
    );
}
