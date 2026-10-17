#[cfg(windows)]
pub(crate) fn read_control_action_ids(
    parameters: &serde_json::Value,
) -> Result<Vec<String>, String> {
    parameters
        .get("ids")
        .and_then(serde_json::Value::as_array)
        .ok_or("control action parameter 'ids' must be an array")?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or("control action item IDs must be strings".into())
        })
        .collect()
}

#[cfg(windows)]
pub(crate) fn read_enabled_catalog_provider(
    root: &std::path::Path,
) -> Result<serde_json::Value, String> {
    let provider =
        shroudforge_loader_config::config::read_loader(root)?["catalog"]["provider"].clone();
    if !provider.is_object()
        || provider["enabled"].as_bool() != Some(true)
        || provider["kind"].as_str() != Some("shroudedit")
    {
        return Err("ShroudEdit catalog access is not enabled in modloader-config.json".into());
    }
    Ok(provider)
}
