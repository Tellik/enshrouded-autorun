use super::notify_command_result::notify_command_result;
use super::open_external_link::open_url;
use super::read_host_arguments::Arguments;
use shroudforge_logging::write_activity;

pub(crate) fn open_requested_link(
    url: String,
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
) {
    match open_url(&url) {
        Ok(()) => write_activity(
            &arguments.root,
            "modloader-ui",
            "Open external link",
            "Succeeded",
            None,
            "info",
        ),
        Err(error) => {
            write_activity(
                &arguments.root,
                "modloader-ui",
                "Open external link",
                "Failed",
                Some(&error),
                "error",
            );
            notify_command_result(&webview, &request_id, Err(error), "", None, false, None);
        }
    }
}
