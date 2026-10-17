use super::read_host_arguments::Arguments;
use super::report_command_result::report_command_result;
use std::time::Instant;

pub(crate) fn remove_update_queue_item(
    id: String,
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    next_refresh: &mut Instant,
) {
    let result = shroudforge_updater::remove_update_queue_item(&arguments.root, &id);
    report_command_result(
        &arguments.root,
        &webview,
        &request_id,
        "Updates",
        "Remove update from queue",
        result,
        "updates.queue.removed",
        None,
        false,
    );
    *next_refresh = Instant::now();
}
