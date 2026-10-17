use super::read_host_arguments::Arguments;
use super::report_command_result::report_command_result;
use super::save_language::save_language;
use std::time::Instant;

pub(crate) fn apply_language_change(
    locale: String,
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    next_refresh: &mut Instant,
) {
    let result = save_language(&arguments.root, &locale);
    report_command_result(
        &arguments.root,
        &webview,
        &request_id,
        "Modloader",
        "Save language",
        result,
        "settings.languageSaved",
        None,
        false,
    );
    *next_refresh = Instant::now();
}
