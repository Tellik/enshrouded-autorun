#[cfg(windows)]
use crate::update_control_action::HeadlessControlAction;

#[cfg(windows)]
pub(crate) fn execute_headless_control_action(
    root: &std::path::Path,
    action: &HeadlessControlAction,
) -> Result<String, String> {
    let parameters = &action.parameters;
    let request = match action.name.as_str() {
        "checkUpdates" => serde_json::json!({"command":"check-system-update","manual":true}),
        "queueSystemUpdate" => serde_json::json!({"command":"enqueue-system-update"}),
        "stageUpdate" => serde_json::json!({
            "command":"stage-system-update",
            "waitForGame":parameters["waitForGame"].as_bool().unwrap_or(true)
        }),
        "selectUpdateQueueItems" => serde_json::json!({
            "command":"select-update-queue-items",
            "ids":crate::read_action_parameters::read_control_action_ids(parameters)?,
            "selected":parameters["selected"]
                .as_bool()
                .ok_or("control action parameter 'selected' must be a boolean")?
        }),
        "removeUpdateQueueItem" => serde_json::json!({
            "command":"remove-update-queue-item",
            "id":parameters["key"]
                .as_str()
                .ok_or("control action parameter 'key' is required")?
        }),
        "startUpdateQueue" => serde_json::json!({
            "command":"start-update-queue",
            "ids":crate::read_action_parameters::read_control_action_ids(parameters)?,
            "waitForGame":parameters["waitForGame"].as_bool().unwrap_or(true),
            "showWindow":false
        }),
        "clearUpdateQueue" => serde_json::json!({"command":"clear-update-queue"}),
        "cancelUpdate" => serde_json::json!({"command":"cancel-update"}),
        "restoreGamefiles" => serde_json::json!({"command":"restore-gamefiles"}),
        "captureGamefileOriginals" => {
            serde_json::json!({"command":"capture-gamefile-originals"})
        }
        "installMod" => serde_json::json!({
            "command":"install-mod",
            "projectId":parameters["projectId"],
            "title":parameters["title"],
            "provider":crate::read_action_parameters::read_catalog_provider(root)?
        }),
        "updateMod" => serde_json::json!({
            "command":"update-mod",
            "modId":parameters["modId"],
            "title":parameters["title"],
            "provider":crate::read_action_parameters::read_catalog_provider(root)?
        }),
        _ => serde_json::json!({"command":action.name}),
    };
    let result = crate::run_request(root, &request)?;
    Ok(result["message"]
        .as_str()
        .unwrap_or("Updater request completed")
        .to_owned())
}
