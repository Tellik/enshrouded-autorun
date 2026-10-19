use super::ModUpdateCheckLock;
use crate::{
    catalog_item::CatalogItem, mod_package::ResolveContentPlan,
    provider_config::CatalogProviderConfig,
};
use serde::Deserialize;
use std::{path::Path, time::Duration};

pub fn resolve_catalog_project(
    client: &reqwest::blocking::Client,
    provider: &CatalogProviderConfig,
    project_id: &str,
    existing_project_ids: &[String],
) -> Result<ResolveContentPlan, String> {
    let response = client
        .post(format!(
            "{}/v3/content/resolve",
            provider.base_url.trim_end_matches('/')
        ))
        .json(&serde_json::json!({
            "project_id": project_id,
            "version_id": null,
            "content_type": "mod",
            "selected": { "game_versions": [], "loaders": [provider.loader] },
            "target": { "game_versions": [], "loaders": [provider.loader] },
            "existing_project_ids": existing_project_ids
        }))
        .send()
        .map_err(|error| format!("ShroudEdit is unreachable: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "ShroudEdit resolution returned HTTP {}",
            response.status()
        ));
    }
    response
        .json()
        .map_err(|error| format!("Invalid installation plan: {error}"))
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstalledCatalogProject {
    mod_id: String,
    version_id: String,
}

pub fn check_catalog_mod_updates(root: &Path, manual: bool) -> Result<serde_json::Value, String> {
    let _check_lock = ModUpdateCheckLock::try_acquire(root)?
        .ok_or("A mod update check is already running for this installation")?;
    let config = shroudforge_loader_config::config::read_loader(root)?;
    if config
        .pointer("/modules/updates/enabled")
        .and_then(serde_json::Value::as_bool)
        == Some(false)
    {
        return Err(crate::UPDATE_MODULE_DISABLED_MESSAGE.into());
    }
    if !manual
        && config
            .pointer("/modules/updates/automatic/enabled")
            .and_then(serde_json::Value::as_bool)
            == Some(false)
    {
        return Err("Automatic update checks are disabled".into());
    }
    let queue_updates = config
        .pointer("/modules/updates/automatic/queueUpdates")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let provider = config
        .pointer("/catalog/provider")
        .cloned()
        .map(serde_json::from_value::<CatalogProviderConfig>)
        .transpose()
        .map_err(|error| format!("invalid catalog provider: {error}"))?
        .unwrap_or_default();
    if provider.kind != "shroudedit" {
        return Err("unsupported catalog provider".into());
    }
    let registry = read_catalog_install_registry(root);
    let projects = registry
        .get("projects")
        .and_then(serde_json::Value::as_object)
        .ok_or("catalog install registry has no project list")?;
    let existing_project_ids = projects.keys().cloned().collect::<Vec<_>>();
    let projects = projects
        .iter()
        .filter_map(|(project_id, value)| {
            serde_json::from_value::<InstalledCatalogProject>(value.clone())
                .ok()
                .map(|installed| (project_id.clone(), installed))
        })
        .collect::<Vec<_>>();
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent(format!(
            "ShroudForge/{} (mod-update-check)",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .map_err(|error| error.to_string())?;
    let mut latest_updates = Vec::new();
    let mut failures = Vec::new();
    let mut checked = 0usize;
    for (project_id, installed) in projects {
        if shroudforge_package::package_lookup::find_mod_package(root, &installed.mod_id).is_none()
        {
            continue;
        }
        let result =
            resolve_catalog_project(&client, &provider, &project_id, &existing_project_ids)
                .and_then(|plan| {
                    if plan.primary.project_id != project_id {
                        return Err(format!(
                            "ShroudEdit returned an unexpected project for {project_id}"
                        ));
                    }
                    Ok(plan.primary.version_id)
                });
        match result {
            Ok(version_id) => {
                checked += 1;
                if version_id != installed.version_id {
                    let manifest = shroudforge_package::package_lookup::find_mod_package(
                        root,
                        &installed.mod_id,
                    )
                    .and_then(|package| {
                        shroudforge_package::config::read_manifest_path(root, &package).ok()
                    });
                    let version_number = client
                        .get(format!(
                            "{}/v3/version/{version_id}",
                            provider.base_url.trim_end_matches('/')
                        ))
                        .send()
                        .ok()
                        .filter(|response| response.status().is_success())
                        .and_then(|response| response.json::<crate::mod_package::ApiVersion>().ok())
                        .filter(|version| {
                            version.id == version_id && version.project_id == project_id
                        });
                    latest_updates.push(serde_json::json!({
                        "projectId": project_id,
                        "modId": installed.mod_id,
                        "name": manifest.as_ref().map(|manifest| manifest.name.as_str()).unwrap_or(&installed.mod_id),
                        "currentVersion": manifest.map(|manifest| manifest.version),
                        "latestVersion": version_number.map(|version| version._version_number),
                        "latestVersionId": version_id
                    }));
                }
            }
            Err(error) => failures.push(format!("{}: {error}", installed.mod_id)),
        }
    }
    let checked_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut queued = 0usize;
    if queue_updates {
        let provider_value = serde_json::to_value(&provider).map_err(|error| error.to_string())?;
        for update in &latest_updates {
            let Some(project_id) = update["projectId"].as_str() else {
                continue;
            };
            let Some(mod_id) = update["modId"].as_str() else {
                continue;
            };
            let title = update["name"].as_str().unwrap_or(mod_id);
            match crate::request_mod_update_if_needed(
                root,
                project_id,
                mod_id,
                title,
                provider_value.clone(),
            ) {
                Ok(true) => queued += 1,
                Ok(false) => {}
                Err(error) => failures.push(format!("Could not queue {mod_id}: {error}")),
            }
        }
    }
    let result = serde_json::json!({
        "checked": checked,
        "available": latest_updates.len(),
        "queued": queued,
        "failed": failures.len(),
        "checkedAt": checked_at,
        "latestUpdates": latest_updates.clone(),
        "errors": failures.clone()
    });
    shroudforge_loader_config::config::update_loader(root, |config| {
        config["modules"]["updates"]["mods"]["lastCheckAt"] = checked_at.into();
        config["modules"]["updates"]["mods"]["latestUpdates"] = result["latestUpdates"].clone();
        config["modules"]["updates"]["mods"]["lastCheckResult"] = serde_json::json!({
            "checked": checked,
            "available": latest_updates.len(),
            "failed": failures.len(),
            "checkedAt": checked_at,
            "errors": failures
        });
        Ok(())
    })?;
    shroudforge_logging::write_activity(
        root,
        "ShroudEdit",
        "Check mod updates",
        if failures.is_empty() {
            "Succeeded"
        } else {
            "Completed with errors"
        },
        Some(&format!(
            "Checked {checked} installed mod(s), found {} update(s), queued {queued}, {} failure(s).",
            latest_updates.len(),
            failures.len()
        )),
        if failures.is_empty() { "info" } else { "warn" },
    );
    Ok(result)
}

pub fn fetch_catalog(
    provider: &CatalogProviderConfig,
    query: &str,
) -> Result<Vec<CatalogItem>, String> {
    if provider.kind != "shroudedit" {
        return Err(format!("unsupported catalog provider: {}", provider.kind));
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(12))
        .user_agent(format!(
            "ShroudForge/{} (catalog)",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .map_err(|error| error.to_string())?;
    let response = client
        .get(format!(
            "{}/v3/search",
            provider.base_url.trim_end_matches('/')
        ))
        .query(&[("query", query), ("limit", "30"), ("loader", "shroudforge")])
        .send()
        .map_err(|error| format!("ShroudEdit is unreachable: {error}"))?;
    if !response.status().is_success() {
        return Err(format!("ShroudEdit returned HTTP {}", response.status()));
    }
    let payload: serde_json::Value = response
        .json()
        .map_err(|error| format!("Invalid ShroudEdit response: {error}"))?;
    let values = payload
        .as_array()
        .or_else(|| payload.get("hits").and_then(|value| value.as_array()))
        .or_else(|| payload.get("projects").and_then(|value| value.as_array()))
        .or_else(|| payload.get("data").and_then(|value| value.as_array()))
        .ok_or("ShroudEdit search response contains no result list")?;
    let mut items = Vec::new();
    for value in values {
        let loaders = value
            .get("loaders")
            .and_then(|value| value.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if !loaders.is_empty()
            && !loaders
                .iter()
                .any(|loader| loader.eq_ignore_ascii_case("shroudforge"))
        {
            continue;
        }
        let id = string_field(value, &["project_id", "projectId", "id"]);
        let slug = string_field(value, &["slug", "project_slug"]);
        let name = string_field(value, &["name", "title"]);
        if id.is_empty() || name.is_empty() {
            continue;
        }
        items.push(CatalogItem {
            id,
            slug: if slug.is_empty() {
                string_field(value, &["project_id", "projectId", "id"])
            } else {
                slug
            },
            name,
            summary: string_field(value, &["summary", "description"]),
            icon_url: optional_string_field(value, &["icon_url", "iconUrl"]),
            author: optional_string_field(value, &["author", "owner_name", "ownerName"]),
            downloads: value
                .get("downloads")
                .or_else(|| value.get("download_count"))
                .and_then(|value| value.as_u64()),
            version: optional_string_field(value, &["latest_version", "latestVersion", "version"]),
        });
    }
    Ok(items)
}

pub fn read_catalog_install_registry(root: &Path) -> serde_json::Value {
    shroudforge_loader_config::config::read_document(root, crate::documents::CATALOG_STATE)
        .unwrap_or_else(|_| serde_json::json!({ "schemaVersion": 1, "projects": {} }))
}

pub fn catalog_installed_mod_ids(root: &Path) -> std::collections::HashSet<String> {
    read_catalog_install_registry(root)
        .get("projects")
        .and_then(|value| value.as_object())
        .into_iter()
        .flat_map(|projects| projects.values())
        .filter_map(|value| value.get("modId").and_then(|value| value.as_str()))
        .map(str::to_owned)
        .collect()
}

pub fn catalog_project_for_mod(root: &Path, mod_id: &str) -> Option<String> {
    read_catalog_install_registry(root)
        .get("projects")?
        .as_object()?
        .iter()
        .find_map(|(project_id, value)| {
            (value.get("modId").and_then(|value| value.as_str()) == Some(mod_id))
                .then(|| project_id.clone())
        })
}

pub fn remember_catalog_install(
    root: &Path,
    project_id: &str,
    version_id: &str,
    mod_id: &str,
    package: &Path,
) -> Result<(), String> {
    shroudforge_loader_config::config::update_state_section(root, "catalog", |stored| {
        let mut registry = stored
            .cloned()
            .unwrap_or_else(|| serde_json::json!({"schemaVersion":1,"projects":{}}));
        registry["projects"][project_id] = serde_json::json!({"modId":mod_id,"versionId":version_id,"file":package.file_name().and_then(|value|value.to_str()).unwrap_or_default()});
        shroudforge_loader_config::config::validate_document(
            root,
            crate::documents::CATALOG_STATE,
            &registry,
        )?;
        Ok(registry)
    })
}

pub fn forget_catalog_install(root: &Path, mod_id: &str) -> Result<(), String> {
    shroudforge_loader_config::config::update_state_section(root, "catalog", |stored| {
        let mut registry = stored
            .cloned()
            .unwrap_or_else(|| serde_json::json!({"schemaVersion":1,"projects":{}}));
        registry["projects"]
            .as_object_mut()
            .ok_or("invalid catalog install registry")?
            .retain(|_, value| value.get("modId").and_then(|value| value.as_str()) != Some(mod_id));
        shroudforge_loader_config::config::validate_document(
            root,
            crate::documents::CATALOG_STATE,
            &registry,
        )?;
        Ok(registry)
    })
}

pub fn string_field(value: &serde_json::Value, names: &[&str]) -> String {
    optional_string_field(value, names).unwrap_or_default()
}

pub fn optional_string_field(value: &serde_json::Value, names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        value
            .get(name)
            .and_then(|value| value.as_str())
            .map(str::to_owned)
    })
}
