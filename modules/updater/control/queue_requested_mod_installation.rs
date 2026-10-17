#[cfg(windows)]
use crate::request_mod_install;

#[cfg(windows)]
pub(crate) fn queue_requested_mod_installation(
    root: &std::path::Path,
    parameters: &serde_json::Value,
) -> Result<String, String> {
    let project_id = parameters
        .get("projectId")
        .and_then(serde_json::Value::as_str)
        .ok_or("control action parameter 'projectId' is required")?;
    let title = parameters
        .get("title")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(project_id);
    request_mod_install(
        root,
        project_id,
        title,
        crate::read_action_parameters::read_enabled_catalog_provider(root)?,
    )?;
    Ok(format!(
        "Mod installation for {project_id} added to the shared update queue"
    ))
}
