use super::read_host_arguments::Arguments;
use super::remove_installed_mod::remove_mod;
use super::report_command_result::report_command_result;
use std::time::Instant;

pub(crate) fn remove_requested_mod(
    id: String,
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    next_refresh: &mut Instant,
) {
    let result = remove_mod(&arguments.root, &id);
    report_command_result(
        &arguments.root,
        &webview,
        &request_id,
        &id,
        "Remove mod",
        result,
        "mod.removedToast",
        None,
        false,
    );
    *next_refresh = Instant::now();
}
