use crate::command_result::AsyncUiResult;
use crate::release_state::ReleaseState;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;

pub fn start_staging(
    root: PathBuf,
    state: Arc<Mutex<ReleaseState>>,
    results: mpsc::Sender<AsyncUiResult>,
    request_id: String,
    wait_pid: u32,
    wait_for_game: bool,
) {
    if !crate::update_check::system_updates_enabled(&root) {
        let _ = results.send(AsyncUiResult {
            request_id,
            source: "Updates".into(),
            action: "Stage update".into(),
            success_message: "updates.stagedToast".into(),
            result: Err("System updates module is disabled".into()),
        });
        return;
    }
    let release = {
        let Ok(mut value) = state.lock() else {
            let _ = results.send(AsyncUiResult {
                request_id,
                source: "Updates".into(),
                action: "Stage update".into(),
                success_message: "updates.stagedToast".into(),
                result: Err("Update status is unavailable.".into()),
            });
            return;
        };
        if value.state == "downloading" || value.staged {
            let error = if value.staged {
                "An update is already staged."
            } else {
                "An update is already downloading."
            };
            let _ = results.send(AsyncUiResult {
                request_id,
                source: "Updates".into(),
                action: "Stage update".into(),
                success_message: "updates.stagedToast".into(),
                result: Err(error.into()),
            });
            return;
        }
        let Some(release) = value.release.clone() else {
            value.state = "error".into();
            value.message = Some("Check for updates before downloading.".into());
            let _ = results.send(AsyncUiResult {
                request_id,
                source: "Updates".into(),
                action: "Stage update".into(),
                success_message: "updates.stagedToast".into(),
                result: Err("Check for updates before downloading.".into()),
            });
            return;
        };
        value.state = "downloading".into();
        value.message = Some("Downloading and verifying update…".into());
        release
    };
    let version = release.version.clone();
    thread::spawn(move || {
        let request = serde_json::json!({"version":release.version,"downloadUrl":release.download_url,"checksum":release.checksum});
        let result = crate::request_system_stage(&root, request, wait_pid, wait_for_game);
        let feedback = match result {
            Ok(()) => Ok(format!(
                "Download of version {version} queued in the independent updater."
            )),
            Err(error) => Err(error),
        };
        if let Ok(mut value) = state.lock() {
            match &feedback {
                Ok(_) => {
                    value.state = "queued".into();
                    value.staged = false;
                    value.message =
                        Some("System update download queued in the independent updater.".into());
                }
                Err(error) => {
                    value.state = "error".into();
                    value.message = Some(error.clone());
                }
            }
        }
        let _ = results.send(AsyncUiResult {
            request_id,
            source: "Updates".into(),
            action: "Queue update".into(),
            success_message: "updates.stagedQueued".into(),
            result: feedback,
        });
    });
}
