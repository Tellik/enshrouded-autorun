use super::read_host_arguments::Arguments;
use shroudforge_loader_config::runtime_status::runtime_pid;
use shroudforge_updater::start_staging;
use shroudforge_updater::command_result::AsyncUiResult;
use shroudforge_updater::release_state::ReleaseState;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;

pub(crate) fn stage_system_update(
    request_id: String,
    wait_for_game: bool,
    arguments: &Arguments,
    release_state: &Arc<Mutex<ReleaseState>>,
    async_sender: &mpsc::Sender<AsyncUiResult>,
) {
    let wait_pid = if !wait_for_game {
        0
    } else {
        runtime_pid(&arguments.root).unwrap_or(arguments.game_pid)
    };
    start_staging(
        arguments.root.clone(),
        release_state.clone(),
        async_sender.clone(),
        request_id,
        wait_pid,
        wait_for_game,
    )
}
