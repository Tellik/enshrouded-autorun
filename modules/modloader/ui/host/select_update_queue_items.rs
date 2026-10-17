use super::read_host_arguments::Arguments;
use super::report_command_result::report_command_result;
use std::time::Instant;

pub(crate) fn select_update_queue_items(
    ids: Vec<String>,
    selected: bool,
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    next_refresh: &mut Instant,
) {
    let result = shroudforge_updater::select_update_queue_items(&arguments.root, &ids, selected);
    report_command_result(
        &arguments.root,
        &webview,
        &request_id,
        "Updates",
        "Change update selection",
        result,
        "updates.queue.selectionChanged",
        None,
        false,
    );
    *next_refresh = Instant::now();
}
