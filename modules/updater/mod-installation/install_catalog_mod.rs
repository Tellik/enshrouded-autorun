use crate as shroudforge_updater;
use crate::download_mod_package::download_catalog_version;
use crate::mod_installation::catalog::{
    catalog_project_for_mod, forget_catalog_install, read_catalog_install_registry,
    remember_catalog_install, resolve_catalog_project,
};
use crate::provider_config::CatalogProviderConfig;
use crate::replace_installed_mod::replace_catalog_mod_with_runtime_dependencies;
use shroudforge_loader_config::runtime_status::runtime_is_active;
use shroudforge_package::package_lookup::find_mod_package;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub fn install_mod_from_catalog(
    root: &Path,
    provider: &CatalogProviderConfig,
    project_id: &str,
    update_mod_id: Option<&str>,
    queue_item_id: &str,
) -> Result<Vec<String>, String> {
    if provider.kind != "shroudedit" {
        return Err("unsupported catalog provider".into());
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
    let plan = resolve_catalog_project(&client, provider, project_id, &existing_project_ids)?;
    let mut resolved = plan.dependencies;
    resolved.push(plan.primary);
    let mut installed_paths = Vec::new();
    let mut installed = Vec::new();
    let mut staged_update: Option<(String, String, PathBuf)> = None;
    let mut runtime_dependency_ids = Vec::new();
    let mut refresh_runtime_after_cleanup = false;
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
        runtime_dependency_ids = installed
            .iter()
            .filter(|id| Some(id.as_str()) != update_mod_id)
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
            .collect();
        if let Some((updated_project_id, version_id, staging)) = staged_update.as_ref() {
            refresh_runtime_after_cleanup =
                runtime_is_active(root) && !runtime_dependency_ids.is_empty();
            let mod_id = update_mod_id.expect("staged update has target");
            let destination = shroudforge_loader_config::paths::mods_dir(root).join(mod_id);
            replace_catalog_mod_with_runtime_dependencies(
                root,
                mod_id,
                staging,
                &destination,
                updated_project_id,
                version_id,
                &runtime_dependency_ids,
            )?;
        } else if runtime_is_active(root) && !runtime_dependency_ids.is_empty() {
            shroudforge_updater::request_runtime_mod_reload(root, &runtime_dependency_ids)
                .map_err(|error| format!("catalog package installed, but its runtime modules could not be loaded in the running game: {error}"))?;
        }
        Ok(())
    })();
    if let Err(error) = result {
        let mut cleanup_errors = Vec::new();
        let mut removed_mod_ids = Vec::new();
        for path in installed_paths {
            let mod_id = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned());
            match move_directory_to_recycle_bin(root, &path, "Failed mod installation") {
                Ok(()) => {
                    if let Some(mod_id) = mod_id {
                        removed_mod_ids.push(mod_id);
                    }
                }
                Err(cleanup_error) => cleanup_errors.push(cleanup_error),
            }
        }
        if let Some((_, _, staging)) = staged_update
            && let Err(cleanup_error) =
                move_directory_to_recycle_bin(root, &staging, "Failed mod update staging files")
        {
            cleanup_errors.push(cleanup_error);
        }
        for mod_id in removed_mod_ids {
            if Some(mod_id.as_str()) != update_mod_id
                && let Err(cleanup_error) = forget_catalog_install(root, &mod_id)
            {
                cleanup_errors.push(format!(
                    "could not clear catalog record for recycled mod {mod_id}: {cleanup_error}"
                ));
            }
        }
        if refresh_runtime_after_cleanup
            && let Err(refresh_error) =
                shroudforge_updater::request_runtime_mod_reload(root, &runtime_dependency_ids)
        {
            cleanup_errors.push(format!(
                "runtime recovery after package cleanup also failed: {refresh_error}"
            ));
        }
        return Err(if cleanup_errors.is_empty() {
            error
        } else {
            format!(
                "{error}. Cleanup also reported: {}",
                cleanup_errors.join("; ")
            )
        });
    }
    Ok(installed)
}

fn move_directory_to_recycle_bin(root: &Path, source: &Path, reason: &str) -> Result<(), String> {
    if !source.exists() {
        return Ok(());
    }
    let id = format!(
        "mod-cleanup-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let mods = shroudforge_loader_config::paths::mods_dir(root);
    let mod_id = source
        .parent()
        .filter(|parent| *parent == mods)
        .and_then(|_| source.file_name())
        .and_then(|name| name.to_str())
        .filter(|name| shroudforge_package::package_lookup::valid_identifier(name));
    if let Some(mod_id) = mod_id {
        crate::recycle_bin::preserve_mod_directory(root, source, &id, reason, mod_id, None)?;
    } else {
        crate::recycle_bin::preserve_directory(root, source, &id, reason)?;
    }
    fs::remove_dir_all(source).map_err(|error| {
        format!(
            "could not remove recycled package {}: {error}",
            source.display()
        )
    })
}
