use super::read_host_arguments::Arguments;
use super::report_command_result::report_command_result;
use shroudforge_updater::release_state::ReleaseState;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;
use tao::event_loop::ControlFlow;

pub(crate) fn clear_update_queue(
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    release_state: &Arc<Mutex<ReleaseState>>,
    next_refresh: &mut Instant,
    control_flow: &mut ControlFlow,
) -> bool {
    let result = shroudforge_updater::clear_update_queue(&arguments.root);
    let clear_succeeded = result.is_ok();
    if result.is_ok() {
        if let Ok(mut release) = release_state.lock() {
            release.state = "ready".into();
            release.staged = false;
            release.message = Some("Update queue cleared.".into());
        }
    }
    report_command_result(
        &arguments.root,
        &webview,
        &request_id,
        "Updates",
        "Clear update queue",
        result,
        "updates.queue.cleared",
        None,
        false,
    );
    *next_refresh = Instant::now();
    if arguments.updater_window && clear_succeeded {
        *control_flow = ControlFlow::Exit;
        return true;
    }
    false
}
