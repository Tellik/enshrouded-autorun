#[cfg(windows)]
use crate::cancel_update_queue::cancel_update_queue;
#[cfg(windows)]
use crate::clear_update_queue::clear_update_queue;
#[cfg(windows)]
use crate::headless_check_system_updates;
#[cfg(windows)]
use crate::remove_update_item::remove_update_queue_item;
#[cfg(windows)]
use crate::request_gamefiles_restore;
#[cfg(windows)]
use crate::select_update_items::select_update_queue_items;
#[cfg(windows)]
use crate::update_control_action::HeadlessControlAction;

#[cfg(windows)]
pub(crate) fn execute_headless_control_action(
    root: &std::path::Path,
    action: &HeadlessControlAction,
) -> Result<String, String> {
    let parameters = &action.parameters;
    match action.name.as_str() {
        "checkUpdates" => {
            headless_check_system_updates(root, crate::update_check::SystemUpdateCheckMode::Manual)
        }
        "queueSystemUpdate" => {
            crate::queue_stored_system_update::queue_stored_system_update(root, parameters)
        }
        "stageUpdate" => {
            crate::stage_stored_system_update::stage_stored_system_update(root, parameters)
        }
        "selectUpdateQueueItems" => {
            let selected = parameters
                .get("selected")
                .and_then(serde_json::Value::as_bool)
                .ok_or("control action parameter 'selected' must be a boolean")?;
            select_update_queue_items(
                root,
                &crate::read_action_parameters::read_control_action_ids(parameters)?,
                selected,
            )?;
            Ok("Update queue selection saved".into())
        }
        "removeUpdateQueueItem" => {
            let key = parameters
                .get("key")
                .and_then(serde_json::Value::as_str)
                .ok_or("control action parameter 'key' is required")?;
            remove_update_queue_item(root, key)?;
            Ok("Update removed from the shared queue".into())
        }
        "startUpdateQueue" => {
            crate::start_selected_updates::start_selected_updates(root, parameters)
        }
        "clearUpdateQueue" => {
            clear_update_queue(root)?;
            Ok("Update queue cleared".into())
        }
        "cancelUpdate" => {
            cancel_update_queue(root)?;
            Ok("Update queue cancellation requested".into())
        }
        "restoreGamefiles" => {
            request_gamefiles_restore(root)?;
            Ok("Gamefile restoration was handed to the updater".into())
        }
        "captureGamefileOriginals" => {
            crate::capture_original_game_files::capture_original_game_files(root, parameters)
        }
        "installMod" => crate::queue_requested_mod_installation::queue_requested_mod_installation(
            root, parameters,
        ),
        "updateMod" => {
            crate::queue_requested_mod_update::queue_requested_mod_update(root, parameters)
        }
        _ => Err(format!(
            "unsupported headless control action '{}': use the Modloader UI",
            action.name
        )),
    }
}
