use super::open_storage_folder::open_storage_path;
use super::read_host_arguments::Arguments;
use super::report_command_result::report_command_result;

pub(crate) fn open_requested_folder(
    key: String,
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
) {
    let result = open_storage_path(&arguments.root, &key);
    report_command_result(
        &arguments.root,
        &webview,
        &request_id,
        "Settings",
        "Open storage folder",
        result,
        "settings.pathsOpened",
        None,
        false,
    );
}
