use crate as shroudforge_updater;
use crate::download_mod_package::download_catalog_version;
use crate::mod_installation::catalog::{
    catalog_project_for_mod, forget_catalog_install, read_catalog_install_registry,
    remember_catalog_install,
};
use crate::mod_package::ResolveContentPlan;
use crate::provider_config::ProviderConfig;
use crate::replace_installed_mod::replace_catalog_mod;
use shroudforge_loader_config::runtime_status::runtime_is_active;
use shroudforge_package::package_lookup::find_mod_package;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub fn install_mod_from_catalog(
    root: &Path,
    provider: &ProviderConfig,
    project_id: &str,
    update_mod_id: Option<&str>,
    queue_item_id: &str,
) -> Result<Vec<String>, String> {
    if !provider.enabled || provider.kind != "shroudedit" {
        return Err("ShroudEdit access is disabled".into());
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .user_agent(format!(
            "ShroudForge/{} (mod-installer)",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .map_err(|error| error.to_string())?;
    let registry = read_catalog_install_registry(root);
    if let Some(mod_id) = update_mod_id {
        if catalog_project_for_mod(root, mod_id).as_deref() != Some(project_id) {
            return Err(
                "catalog update target does not match the installed ShroudEdit project".into(),
            );
        }
        if find_mod_package(root, mod_id).is_none() {
            return Err(format!(
                "installed mod package '{mod_id}' could not be found"
            ));
        }
    }
    let existing_project_ids = registry
        .get("projects")
        .and_then(|value| value.as_object())
        .map(|projects| projects.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
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
    let plan: ResolveContentPlan = response
        .json()
        .map_err(|error| format!("Invalid installation plan: {error}"))?;
    let mut resolved = plan.dependencies;
    resolved.push(plan.primary);
    let mut installed_paths = Vec::new();
    let mut installed = Vec::new();
    let mut staged_update: Option<(String, String, PathBuf)> = None;
    let result = (|| {
        for item in resolved {
            let is_primary_update = update_mod_id.is_some() && item.project_id == project_id;
            if is_primary_update {
                let staged = download_catalog_version(
                    root,
                    provider,
                    &client,
                    &item,
                    update_mod_id,
                    queue_item_id,
                )?;
                staged_update = Some((item.project_id.clone(), item.version_id.clone(), staged.1));
                installed.push(staged.0);
                continue;
            }
            let installed_mod =
                download_catalog_version(root, provider, &client, &item, None, queue_item_id)?;
            installed_paths.push(installed_mod.1.clone());
            remember_catalog_install(
                root,
                &item.project_id,
                &item.version_id,
                &installed_mod.0,
                &installed_mod.1,
            )?;
            installed.push(installed_mod.0);
        }
        if update_mod_id.is_none() && runtime_is_active(root) {
            let runtime_ids = installed
                .iter()
                .filter_map(|id| {
                    let package = shroudforge_loader_config::paths::mods_dir(root).join(id);
                    shroudforge_package::config::read_manifest_path(root, &package)
                        .ok()
                        .filter(|manifest| {
                            manifest
                                .capabilities
                                .iter()
                                .any(|capability| capability.requires_runtime())
                        })
                        .map(|manifest| manifest.id)
                })
                .collect::<Vec<_>>();
            if !runtime_ids.is_empty() {
                shroudforge_updater::request_runtime_mod_reload(root, &runtime_ids)
                    .map_err(|error| format!("catalog package installed, but its runtime modules could not be loaded in the running game: {error}"))?;
            }
        }
        if let Some((updated_project_id, version_id, staging)) = staged_update.take() {
            let mod_id = update_mod_id.expect("staged update has target");
            let destination = shroudforge_loader_config::paths::mods_dir(root).join(mod_id);
            if let Err(error) = replace_catalog_mod(
                root,
                mod_id,
                &staging,
                &destination,
                &updated_project_id,
                &version_id,
            ) {
                let _ = fs::remove_dir_all(&staging);
                return Err(error);
            }
        }
        Ok(())
    })();
    if let Err(error) = result {
        for path in installed_paths {
            // Failed multi-package installs remain recoverable in the loader trash.
            let trash = shroudforge_loader_config::paths::ui_data_dir(root).join("trash");
            let _ = fs::create_dir_all(&trash);
            if let Some(name) = path.file_name() {
                let destination = trash.join(format!(
                    "failed-{}-{}",
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos(),
                    name.to_string_lossy()
                ));
                let _ = fs::rename(&path, destination);
            }
        }
        if let Some((_, _, staging)) = staged_update {
            let _ = fs::remove_dir_all(staging);
        }
        for mod_id in &installed {
            if Some(mod_id.as_str()) != update_mod_id {
                let _ = forget_catalog_install(root, mod_id);
            }
        }
        return Err(error);
    }
    Ok(installed)
}
