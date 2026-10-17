use super::mark_control_action::mark_control_action;
use super::notify_command_result::notify_command_result;
use shroudforge_logging::write_activity;
use shroudforge_package::package_lookup::mod_ecosystem;
use shroudforge_updater::command_result::AsyncUiResult;
use std::path::Path;

pub(crate) fn report_async_result(root: &Path, webview: &wry::WebView, command: AsyncUiResult) {
    match &command.result {
        Ok(message) => mark_control_action(root, &command.request_id, "succeeded", message),
        Err(error) => mark_control_action(root, &command.request_id, "failed", error),
    }
    let (result, activity_result, details, level) = match command.result {
        Ok(details) => (Ok(()), "Succeeded", Some(details), "success"),
        Err(error) => (Err(error.clone()), "Failed", Some(error), "error"),
    };
    write_activity(
        root,
        &command.source,
        &command.action,
        activity_result,
        details.as_deref(),
        level,
    );
    let ecosystem = mod_ecosystem(root, &command.source);
    notify_command_result(
        webview,
        &command.request_id,
        result,
        &command.success_message,
        None,
        false,
        ecosystem,
    );
}
