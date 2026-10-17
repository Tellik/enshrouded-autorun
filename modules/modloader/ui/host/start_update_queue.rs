use super::read_host_arguments::Arguments;
use super::report_command_result::report_command_result;
use shroudforge_updater::release_state::ReleaseState;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

pub(crate) fn start_update_queue(
    ids: Vec<String>,
    wait_for_game: bool,
    show_window: bool,
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    release_state: &Arc<Mutex<ReleaseState>>,
    next_refresh: &mut Instant,
) {
    let result =
        shroudforge_updater::start_update_queue(&arguments.root, &ids, wait_for_game, show_window);
    if result.is_ok() {
        if let Ok(mut release) = release_state.lock() {
            release.state = "queued".into();
            release.message = Some("Selected updates are queued.".into());
        }
    }
    report_command_result(
        &arguments.root,
        &webview,
        &request_id,
        "Updates",
        "Start update queue",
        result,
        "updates.queue.started",
        None,
        false,
    );
    *next_refresh = Instant::now();
}
