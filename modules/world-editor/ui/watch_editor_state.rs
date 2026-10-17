use crate::blueprint_library::ViewState;
use crate::blueprint_library::view_state;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

pub(crate) fn watch_editor_state(
    state_root: PathBuf,
    refresh_ms: u64,
) -> mpsc::Receiver<ViewState> {
    let (state_sender, state_updates) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        loop {
            if let Ok(state) = view_state(&state_root) {
                match state_sender.try_send(state) {
                    Ok(()) | Err(mpsc::TrySendError::Full(_)) => {}
                    Err(mpsc::TrySendError::Disconnected(_)) => break,
                }
            }
            std::thread::sleep(Duration::from_millis(refresh_ms));
        }
    });
    state_updates
}
