use super::read_host_arguments::Arguments;
use super::report_command_result::report_command_result;
use std::time::Instant;

pub(crate) fn capture_game_file_originals(
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    next_refresh: &mut Instant,
) {
    let target = if arguments.server { "server" } else { "client" };
    let result = shroudforge_game_assets::originals::capture_verified_originals(&arguments.root, target)
        .map(|_| ());
    report_command_result(
        &arguments.root,
        &webview,
        &request_id,
        "Backup Management",
        "Capture verified original gamefiles",
        result,
        "backups.originals.captured",
        None,
        false,
    );
    *next_refresh = Instant::now();
}
