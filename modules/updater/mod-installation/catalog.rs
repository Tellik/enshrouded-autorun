use std::{path::Path, time::Duration};
use crate::{catalog_item::CatalogItem, provider_config::CatalogProviderConfig};

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
            version: optional_string_field(
                value,
                &["latest_version", "latestVersion", "version"],
            ),
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
        shroudforge_loader_config::config::validate_document(root, crate::documents::CATALOG_STATE, &registry)?;
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
            .retain(|_, value| {
                value.get("modId").and_then(|value| value.as_str()) != Some(mod_id)
            });
        shroudforge_loader_config::config::validate_document(root, crate::documents::CATALOG_STATE, &registry)?;
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
