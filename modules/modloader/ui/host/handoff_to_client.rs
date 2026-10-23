use super::read_client_ui_process::active_client_ui_pid;
use std::path::Path;

pub(crate) fn handoff_to_client(root: &Path) -> Result<Option<u32>, String> {
    let Some(pid) = active_client_ui_pid(root) else {
        return Ok(None);
    };
    shroudforge_loader_config::config::request_window_visibility(root, "modloaderUi", true)?;
    Ok(Some(pid))
}
