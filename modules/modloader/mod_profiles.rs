//! Data-only compatibility adapters. Reading the catalog never loads a DLL.
use crate::dll_manager::{DllProfileFile, DllProfileTarget};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Component, Path},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModCompatibilityProfile {
    pub schema_version: u32,
    pub id: String,
    #[serde(default)]
    pub mod_id: String,
    pub package_directory: String,
    pub name: String,
    #[serde(default)]
    pub target: DllProfileTarget,
    #[serde(default)]
    pub mod_json: Option<Value>,
    #[serde(default)]
    pub extended_mod_json: Option<Value>,
    #[serde(default)]
    pub package_files: HashMap<String, String>,
    #[serde(default)]
    pub import_files: HashMap<String, String>,
    #[serde(default)]
    pub files: Vec<DllProfileFile>,
}

pub fn read_profiles(root: &Path) -> Result<Vec<ModCompatibilityProfile>, Vec<String>> {
    let directory = crate::paths::mod_profiles_dir(root);
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(vec![format!("{}: {error}", directory.display())]),
    };
    let mut paths: Vec<_> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        })
        .collect();
    paths.sort();
    let mut profiles = Vec::new();
    let mut ids = HashSet::new();
    for path in paths {
        let result = fs::read(&path)
            .map_err(|error| error.to_string())
            .and_then(|bytes| {
                serde_json::from_slice::<ModCompatibilityProfile>(&bytes)
                    .map_err(|error| error.to_string())
            })
            .and_then(|profile| {
                validate_profile(&profile)?;
                Ok(profile)
            });
        match result {
            Ok(profile) if ids.insert(profile.id.clone()) => profiles.push(profile),
            Ok(_) => {
                tracing::warn!(target: "shroudforge::dll_manager", profile = %path.display(), "Duplicate compatibility profile ID. File skipped")
            }
            Err(error) => {
                tracing::warn!(target: "shroudforge::dll_manager", profile = %path.display(), %error, "Invalid compatibility profile. File skipped")
            }
        }
    }
    Ok(profiles)
}

fn validate_profile(profile: &ModCompatibilityProfile) -> Result<(), String> {
    if profile.schema_version != 1
        || profile.id.trim().is_empty()
        || profile.id == "none"
        || profile.id.len() > 128
        || profile.name.trim().is_empty()
        || profile.name.len() > 160
        || !single_folder(&profile.package_directory)
    {
        return Err("invalid compatibility profile identity".into());
    }
    for value in [&profile.mod_json, &profile.extended_mod_json]
        .into_iter()
        .flatten()
    {
        if !value.is_object() {
            return Err("manifest additions must be JSON objects".into());
        }
    }
    if profile
        .mod_json
        .as_ref()
        .and_then(Value::as_object)
        .is_some_and(|value| value.contains_key("id") || value.contains_key("version"))
    {
        return Err("a profile cannot replace the mod ID or version".into());
    }
    if let Some(capabilities) = profile
        .mod_json
        .as_ref()
        .and_then(|value| value.get("capabilities"))
    {
        serde_json::from_value::<Vec<crate::Capability>>(capabilities.clone())
            .map_err(|error| format!("invalid profile capabilities: {error}"))?;
    }
    if profile.files.len() > 16
        || profile.package_files.len() > 64
        || profile.import_files.len() > 64
    {
        return Err("compatibility profile has too many files".into());
    }
    let mut seen = HashSet::new();
    for file in &profile.files {
        if !relative_file(&file.path)
            || !file.path.to_ascii_lowercase().ends_with(".dll")
            || !seen.insert(file.path.to_ascii_lowercase())
        {
            return Err(format!("invalid or duplicate DLL path: {}", file.path));
        }
    }
    let mut bytes = 0usize;
    for (path, contents) in &profile.package_files {
        bytes = bytes.saturating_add(contents.len());
        if !relative_file(path)
            || matches!(path.as_str(), "mod.json" | "extended.mod.json")
            || contents.len() > 1024 * 1024
        {
            return Err(format!("invalid virtual package file: {path}"));
        }
    }
    if bytes > 4 * 1024 * 1024 {
        return Err("virtual package files exceed 4 MiB".into());
    }
    for (source, destination) in &profile.import_files {
        if !relative_file(source) || !relative_file(destination) {
            return Err("invalid installation file mapping".into());
        }
    }
    Ok(())
}

pub(crate) fn single_folder(value: &str) -> bool {
    relative_file(value) && Path::new(value).components().count() == 1
}

pub(crate) fn relative_file(value: &str) -> bool {
    !value.is_empty()
        && !value.contains(['\\', ':'])
        && Path::new(value)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

pub fn select_profile<'a>(
    profiles: &'a [ModCompatibilityProfile],
    directory: &str,
    mod_id: Option<&str>,
    selected: Option<&str>,
) -> Option<&'a ModCompatibilityProfile> {
    if let Some(selected) = selected.filter(|selected| !selected.is_empty()) {
        return profiles.iter().find(|profile| profile.id == selected);
    }
    profiles.iter().find(|profile| match mod_id {
        Some(id) if !profile.mod_id.is_empty() => profile.mod_id == id,
        _ => profile.package_directory.eq_ignore_ascii_case(directory),
    })
}

pub fn profile_for_package(
    root: &Path,
    directory: &str,
    mod_id: Option<&str>,
) -> Option<ModCompatibilityProfile> {
    let profiles = read_profiles(root).ok()?;
    let selected = crate::dll_manager::read_selections(root)
        .into_iter()
        .find(|selection| selection.package_directory.eq_ignore_ascii_case(directory))
        .and_then(|selection| selection.compatibility_profile);
    select_profile(&profiles, directory, mod_id, selected.as_deref()).cloned()
}
