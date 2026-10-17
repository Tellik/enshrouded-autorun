use super::choose_storage_folder::choose_folder;
use super::read_host_arguments::Arguments;
use super::report_command_result::report_command_result;
use super::save_loader_setting::save_setting;
use std::time::Instant;

pub(crate) fn change_storage_folder(
    key: String,
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    next_refresh: &mut Instant,
) {
    let result = choose_folder(&key)
        .and_then(|selected| save_setting(&arguments.root, "paths", None, &key, &selected));
    report_command_result(
        &arguments.root,
        &webview,
        &request_id,
        "Settings",
        "Choose storage folder",
        result,
        "settings.pathsSaved",
        Some("settings"),
        false,
    );
    *next_refresh = Instant::now();
}
