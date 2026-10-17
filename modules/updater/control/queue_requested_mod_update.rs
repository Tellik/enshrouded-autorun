#[cfg(windows)]
use crate::request_mod_update;

#[cfg(windows)]
pub(crate) fn queue_requested_mod_update(
    root: &std::path::Path,
    parameters: &serde_json::Value,
) -> Result<String, String> {
    let mod_id = parameters
        .get("modId")
        .and_then(serde_json::Value::as_str)
        .ok_or("control action parameter 'modId' is required")?;
    let catalog =
        shroudforge_loader_config::config::read_document(root, crate::documents::CATALOG_STATE)?;
    let project_id = catalog
        .get("projects")
        .and_then(serde_json::Value::as_object)
        .and_then(|projects| {
            projects.iter().find_map(|(project_id, item)| {
                (item.get("modId").and_then(serde_json::Value::as_str) == Some(mod_id))
                    .then(|| project_id.clone())
            })
        })
        .ok_or("this mod is not registered as a ShroudEdit installation")?;
    let title = parameters
        .get("title")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(mod_id);
    request_mod_update(
        root,
        &project_id,
        mod_id,
        title,
        crate::read_action_parameters::read_enabled_catalog_provider(root)?,
    )?;
    Ok(format!(
        "Mod update for {mod_id} added to the shared update queue"
    ))
}
