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
pub(crate) fn read_catalog_provider(
    root: &std::path::Path,
) -> Result<serde_json::Value, String> {
    let config = shroudforge_loader_config::config::read_loader(root)?;
    let provider = config.pointer("/catalog/provider").cloned();
    let provider = match provider {
        Some(provider) if provider.is_object() => {
            serde_json::from_value::<crate::provider_config::CatalogProviderConfig>(provider)
                .map_err(|error| format!("invalid catalog provider: {error}"))?
        }
        _ => crate::provider_config::CatalogProviderConfig::default(),
    };
    if provider.kind != "shroudedit" {
        return Err("unsupported catalog provider".into());
    }
    serde_json::to_value(provider).map_err(|error| error.to_string())
}
