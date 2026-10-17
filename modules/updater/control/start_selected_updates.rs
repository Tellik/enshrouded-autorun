#[cfg(windows)]
use crate::start_update_queue::start_update_queue;
#[cfg(windows)]
use crate::update_queue_storage::read_update_queue_value;

#[cfg(windows)]
pub(crate) fn start_selected_updates(
    root: &std::path::Path,
    parameters: &serde_json::Value,
) -> Result<String, String> {
    let mut selected_ids = crate::read_action_parameters::read_control_action_ids(parameters)?;
    if selected_ids.is_empty() {
        selected_ids = read_update_queue_value(root)["items"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|item| item["selected"].as_bool() == Some(true))
            .filter_map(|item| item["id"].as_str().map(str::to_owned))
            .collect();
    }
    let wait_for_game = parameters
        .get("waitForGame")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    start_update_queue(root, &selected_ids, wait_for_game, false)?;
    Ok("Headless update worker started. Progress is in the shared update queue".into())
}
