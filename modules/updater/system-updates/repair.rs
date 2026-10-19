use std::path::Path;

/// Checks the installed ShroudForge files listed in its version manifest for the repair updater.
/// EML startup and EML asset handling do not use this ShroudForge-specific check.
pub fn missing_required_files(root: &Path) -> Vec<String> {
    let version_manifest = "shroudforge/version.json";
    let manifest = std::fs::read(root.join(version_manifest))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
    let Some(entries) = manifest
        .as_ref()
        .and_then(|value| value["managedPaths"].as_array())
    else {
        return vec![version_manifest.into()];
    };
    let Some(mut paths) = entries
        .iter()
        .map(serde_json::Value::as_str)
        .collect::<Option<Vec<_>>>()
    else {
        return vec![version_manifest.into()];
    };
    if paths.is_empty() {
        return vec![version_manifest.into()];
    }

    if !paths.contains(&version_manifest) {
        paths.push(version_manifest);
    }
    if paths
        .iter()
        .any(|path| crate::validate_update_paths::validate_relative_path(path).is_err())
    {
        return vec![version_manifest.into()];
    }

    paths
        .into_iter()
        .filter(|path| !crate::validate_update_paths::installed_path(root, path).is_file())
        .map(str::to_owned)
        .collect()
}

/// Reuses verified release metadata for repairs without another network check.
/// Returns whether the installed files currently require repair.
pub fn refresh_cached_repair_status(root: &std::path::Path) -> Result<bool, String> {
    if missing_required_files(root).is_empty() {
        return Ok(false);
    }
    let config = shroudforge_loader_config::config::read_loader(root)?;
    let latest = &config["modules"]["updates"]["system"]["latestRelease"];
    if latest.is_object()
        && (latest["repairRequired"].as_bool() != Some(true)
            || latest["mandatory"].as_bool() != Some(true)
            || latest["updateAvailable"].as_bool() != Some(true))
    {
        shroudforge_loader_config::config::update_loader(root, |config| {
            let latest = &mut config["modules"]["updates"]["system"]["latestRelease"];
            if latest.is_object() {
                latest["repairRequired"] = serde_json::Value::Bool(true);
                latest["mandatory"] = serde_json::Value::Bool(true);
                latest["updateAvailable"] = serde_json::Value::Bool(true);
            }
            Ok(())
        })?;
    }
    Ok(true)
}
