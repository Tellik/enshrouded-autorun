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
    replace_catalog_mod_with_runtime_dependencies(
        root,
        mod_id,
        staging,
        destination,
        project_id,
        version_id,
        &[],
    )
}

pub(crate) fn replace_catalog_mod_with_runtime_dependencies(
    root: &Path,
    mod_id: &str,
    staging: &Path,
    destination: &Path,
    project_id: &str,
    version_id: &str,
    additional_runtime_mod_ids: &[String],
) -> Result<(), String> {
    let old_manifest = shroudforge_package::config::read_manifest_path(root, destination)
        .map_err(|error| format!("could not inspect installed package before update: {error}"))?;
    let new_manifest = shroudforge_package::config::read_manifest_path(root, staging)
        .map_err(|error| format!("could not inspect downloaded package: {error}"))?;
    if old_manifest.id != mod_id || new_manifest.id != mod_id {
        return Err("mod package identity changed during update".into());
    }
    let local_staging = crate::stage_mod_in_storage::stage_mod_in_storage(root, staging, mod_id)?;
    let old_has_runtime = old_manifest
        .capabilities
        .iter()
        .any(|capability| capability.requires_runtime());
    let new_has_runtime = new_manifest
        .capabilities
        .iter()
        .any(|capability| capability.requires_runtime());
    let runtime_active = runtime_is_active(root);
    let unload_old_runtime = runtime_active && old_has_runtime;
    let reload_new_runtime =
        runtime_active && (new_has_runtime || !additional_runtime_mod_ids.is_empty());
    let mod_ids = vec![mod_id.to_owned()];
    let mut runtime_mod_ids = Vec::with_capacity(additional_runtime_mod_ids.len() + 1);
    if old_has_runtime || new_has_runtime {
        runtime_mod_ids.push(mod_id.to_owned());
    }
    for runtime_mod_id in additional_runtime_mod_ids {
        if !runtime_mod_ids.contains(runtime_mod_id) {
            runtime_mod_ids.push(runtime_mod_id.clone());
        }
    }
    let runtime_recovery_needed = runtime_active && !runtime_mod_ids.is_empty();
    if unload_old_runtime {
        shroudforge_updater::request_runtime_mod_unload(root, &mod_ids).map_err(|error| {
            format!("could not safely unload '{mod_id}' before package replacement: {error}")
        })?;
    }

    let transaction = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .to_string();
    let backup = match crate::recycle_bin::preserve_mod_directory(
        root,
        destination,
        &format!("mod-update-{transaction}"),
        "Previous mod package",
        mod_id,
        crate::recycle_bin::catalog_entry_for_mod(root, mod_id),
    ) {
        Ok(backup) => backup,
        Err(error) => {
            return Err(reload_after_failed_prepare(
                root,
                &runtime_mod_ids,
                runtime_recovery_needed,
                error,
            ));
        }
    };
    if let Err(error) = fs::remove_dir_all(destination) {
        if let Err(rollback_error) = restore_previous_package(
            root,
            mod_id,
            destination,
            &backup,
            &transaction,
            &runtime_mod_ids,
            runtime_recovery_needed,
        ) {
            return Err(format!(
                "Could not prepare mod package replacement: {error}. Rollback also failed: {rollback_error}"
            ));
        }
        return Err(format!(
            "could not prepare mod package replacement: {error}"
        ));
    }
    if let Err(error) = local_staging.commit(destination) {
        if let Err(rollback_error) = restore_previous_package(
            root,
            mod_id,
            destination,
            &backup,
            &transaction,
            &runtime_mod_ids,
            runtime_recovery_needed,
        ) {
            return Err(format!(
                "Could not install updated package: {error}. Rollback also failed: {rollback_error}"
            ));
        }
        return Err(format!("could not install updated package: {error}"));
    }

    if reload_new_runtime
        && let Err(error) = shroudforge_updater::request_runtime_mod_reload(root, &runtime_mod_ids)
    {
        restore_previous_package(
            root,
            mod_id,
            destination,
            &backup,
            &transaction,
            &runtime_mod_ids,
            runtime_recovery_needed,
        )
        .map_err(|rollback_error| {
            format!("live reload failed ({error}) and package rollback failed: {rollback_error}")
        })?;
        return Err(format!(
            "Updated runtime mod failed to load. Previous package restored: {error}"
        ));
    }
    if let Err(error) = remember_catalog_install(root, project_id, version_id, mod_id, destination)
    {
        restore_previous_package(
            root,
            mod_id,
            destination,
            &backup,
            &transaction,
            &runtime_mod_ids,
            runtime_recovery_needed,
        )
        .map_err(|rollback_error| {
            format!("catalog registry update failed ({error}) and package rollback failed: {rollback_error}")
        })?;
        return Err(format!(
            "Could not update catalog registry. Previous package restored: {error}"
        ));
    }
    if let Err(error) = fs::remove_dir_all(staging) {
        let _ = shroudforge_logging::append(
            root,
            'W',
            "updater",
            &format!(
                "Updated mod {mod_id}, but could not remove its verified staging copy: {error}"
            ),
        );
    }
    Ok(())
}

fn reload_after_failed_prepare(
    root: &Path,
    runtime_mod_ids: &[String],
    reload_runtime: bool,
    error: String,
) -> String {
    if !reload_runtime {
        return error;
    }
    match shroudforge_updater::request_runtime_mod_reload(root, runtime_mod_ids) {
        Ok(()) => error,
        Err(reload_error) => format!(
            "{error}. The previous package is still installed, but its runtime reload also failed: {reload_error}"
        ),
    }
}

fn restore_previous_package(
    root: &Path,
    mod_id: &str,
    destination: &Path,
    backup: &Path,
    transaction: &str,
    runtime_mod_ids: &[String],
    reload_runtime: bool,
) -> Result<(), String> {
    if reload_runtime {
        shroudforge_updater::request_runtime_mod_unload(root, &[mod_id.to_owned()]).map_err(
            |error| format!("could not unload the failed package before rollback: {error}"),
        )?;
    }
    let mut preservation_error = None;
    if destination.exists() {
        if let Err(error) = crate::recycle_bin::preserve_mod_directory(
            root,
            destination,
            &format!("failed-mod-update-{transaction}"),
            "Failed mod update package",
            mod_id,
            None,
        ) {
            preservation_error = Some(error);
        }
        fs::remove_dir_all(destination)
            .map_err(|error| format!("could not remove failed package before rollback: {error}"))?;
    }
    crate::recycle_bin::restore_directory(backup, destination)?;
    if reload_runtime {
        shroudforge_updater::request_runtime_mod_reload(root, runtime_mod_ids).map_err(
            |error| format!("previous package was restored but runtime recovery failed: {error}"),
        )?;
    }
    if let Some(error) = preservation_error {
        return Err(format!(
            "previous package was restored, but the failed package could not be retained: {error}"
        ));
    }
    Ok(())
}
