use crate::validate_update_paths::validate_release_path;
use std::{fs, path::Path};

pub(crate) fn managed_paths(source: &Path) -> Result<Vec<String>, String> {
    let bytes = fs::read(source.join("shroudforge/version.json")).map_err(|e| e.to_string())?;
    let release: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let entries = release["managedPaths"]
        .as_array()
        .ok_or("release has no managedPaths")?;
    let mut paths = Vec::new();
    for entry in entries {
        let relative = entry.as_str().ok_or("invalid managed path")?;
        let path = validate_release_path(relative)?;
        // New releases list files, never whole mod/config directories.
        if !source.join(path).is_file() {
            return Err(format!("managed path is not a file: {relative}"));
        }
        paths.push(relative.to_owned());
    }
    paths.sort();
    paths.dedup();
    if !paths.iter().any(|p| p == "shroudforge/version.json") {
        return Err("release must manage shroudforge/version.json".into());
    }
    Ok(paths)
}

pub(crate) fn obsolete_managed_mod_paths(
    target: &Path,
    current: &[String],
) -> Result<Vec<String>, String> {
    let version_path = target.join("shroudforge/version.json");
    if !version_path.is_file() {
        return Ok(Vec::new());
    }
    let release: serde_json::Value =
        serde_json::from_slice(&fs::read(&version_path).map_err(|error| error.to_string())?)
            .map_err(|error| format!("installed version manifest is invalid: {error}"))?;
    let entries = release["managedPaths"]
        .as_array()
        .ok_or("installed version has no managedPaths")?;
    let mut obsolete = Vec::new();
    for entry in entries {
        let Some(relative) = entry.as_str() else {
            return Err("installed version has an invalid managed path".into());
        };
        if !relative.starts_with("mods/")
            || current
                .iter()
                .any(|path| path.eq_ignore_ascii_case(relative))
        {
            continue;
        }
        let mod_relative = relative.strip_prefix("mods/").unwrap_or_default();
        let path = Path::new(mod_relative);
        if mod_relative.is_empty()
            || relative.contains('\\')
            || relative.contains(':')
            || !path
                .components()
                .all(|part| matches!(part, std::path::Component::Normal(_)))
        {
            return Err(format!("unsafe installed managed path: {relative}"));
        }
        obsolete.push(relative.to_owned());
    }
    obsolete.sort();
    obsolete.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    Ok(obsolete)
}
