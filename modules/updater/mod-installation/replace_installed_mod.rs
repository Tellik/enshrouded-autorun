use crate as shroudforge_updater;
use crate::mod_installation::catalog::remember_catalog_install;
use shroudforge_loader_config::runtime_status::runtime_is_active;
use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn replace_catalog_mod(
    root: &Path,
    mod_id: &str,
    staging: &Path,
    destination: &Path,
    project_id: &str,
    version_id: &str,
) -> Result<(), String> {
    let old_manifest = shroudforge_package::config::read_manifest_path(root, destination)
        .map_err(|error| format!("could not inspect installed package before update: {error}"))?;
    let new_manifest = shroudforge_package::config::read_manifest_path(root, staging)
        .map_err(|error| format!("could not inspect downloaded package: {error}"))?;
    if old_manifest.id != mod_id || new_manifest.id != mod_id {
        return Err("mod package identity changed during update".into());
    }
    let has_runtime = old_manifest
        .capabilities
        .iter()
        .any(|capability| capability.requires_runtime())
        && new_manifest
            .capabilities
            .iter()
            .any(|capability| capability.requires_runtime());
    let status_path = shroudforge_loader_config::paths::runtime_dir(root).join("mod-status.json");
    let active = fs::read(&status_path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|value| value["active"].as_array().cloned())
        .is_some_and(|ids| ids.iter().any(|value| value.as_str() == Some(mod_id)));
    let live_reload = has_runtime && runtime_is_active(root);
    let mod_ids = vec![mod_id.to_owned()];
    if live_reload && active {
        shroudforge_updater::request_runtime_mod_unload(root, &mod_ids).map_err(|error| {
            format!("could not safely unload '{mod_id}' before package replacement: {error}")
        })?;
    }

    let transaction = format!(
        "{}-{}",
        mod_id,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let backup_root = shroudforge_loader_config::paths::updates_dir(root).join("mod-backups");
    fs::create_dir_all(&backup_root).map_err(|error| error.to_string())?;
    let backup = backup_root.join(&transaction);
    fs::rename(destination, &backup).map_err(|error| {
        format!("could not move old mod package into rollback storage: {error}")
    })?;
    if let Err(error) = fs::rename(staging, destination) {
        let rollback = fs::rename(&backup, destination);
        if let Err(rollback_error) = rollback {
            return Err(format!(
                "Could not install updated package: {error}. Rollback also failed: {rollback_error}"
            ));
        }
        if live_reload {
            let _ = shroudforge_updater::request_runtime_mod_reload(root, &mod_ids);
        }
        return Err(format!("could not install updated package: {error}"));
    }

    if live_reload {
        if let Err(error) = shroudforge_updater::request_runtime_mod_reload(root, &mod_ids) {
            let failed_new = backup_root.join(format!("{}-failed", transaction));
            let _ = fs::rename(destination, &failed_new);
            fs::rename(&backup, destination).map_err(|rollback_error| {
                format!(
                    "live reload failed ({error}) and package rollback failed: {rollback_error}"
                )
            })?;
            let restore = shroudforge_updater::request_runtime_mod_reload(root, &mod_ids);
            return Err(match restore {
                Ok(()) => format!(
                    "Updated runtime mod failed to load. Previous package restored: {error}"
                ),
                Err(restore_error) => format!(
                    "Updated runtime mod failed to load ({error}). Previous package restored, but runtime recovery also failed: {restore_error}"
                ),
            });
        }
    }
    if let Err(error) = remember_catalog_install(root, project_id, version_id, mod_id, destination)
    {
        let failed_new = backup_root.join(format!("{}-registry-failed", transaction));
        let _ = fs::rename(destination, &failed_new);
        fs::rename(&backup, destination).map_err(|rollback_error| format!("catalog registry update failed ({error}) and package rollback failed: {rollback_error}"))?;
        let restore = if live_reload {
            shroudforge_updater::request_runtime_mod_reload(root, &mod_ids)
        } else {
            Ok(())
        };
        return Err(match restore {
            Ok(()) => {
                format!("Could not update catalog registry. Previous package restored: {error}")
            }
            Err(restore_error) => format!(
                "Catalog registry update failed ({error}). Previous package restored, but runtime recovery failed: {restore_error}"
            ),
        });
    }
    Ok(())
}
