use super::read_host_arguments::Arguments;
use super::report_command_result::report_command_result;
use std::time::Instant;

pub(crate) fn request_diagnostics(
    action: String,
    job_id: Option<String>,
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    next_refresh: &mut Instant,
) {
    let result =
        shroudforge_runtime_diagnostics::request_job(&arguments.root, &action, job_id.as_deref());
    let message = match action.as_str() {
        "start" => "settings.diagnosticStarted",
        "stop" => "settings.diagnosticStopped",
        _ => "settings.snapshotRequested",
    };
    let action_name = match action.as_str() {
        "start" => "Start diagnostics",
        "stop" => "Stop diagnostics",
        _ => "Request diagnostic snapshot",
    };
    report_command_result(
        &arguments.root,
        &webview,
        &request_id,
        "diagnostics",
        action_name,
        result,
        message,
        None,
        false,
    );
    *next_refresh = Instant::now();
}
