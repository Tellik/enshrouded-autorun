use crate::apply_release_files::apply_with_progress_and_cancel;
use crate::read_install_arguments::Arguments;
use crate::read_managed_release_files::{managed_paths, obsolete_managed_paths};
use crate::remove_release_files::{remove_empty_mod_directories, remove_entry};
use crate::restore_release_files::restore_with_progress;
use crate::validate_update_paths::{
    installed_path, mod_relative_path, validate_file_path, validate_roots,
};
use crate::windows::append_log;
use std::{fs, path::Path, time::SystemTime};

struct QueueInstallationGuard<'a> {
    root: &'a Path,
}

impl QueueInstallationGuard<'_> {
    fn begin(root: &Path) -> Result<QueueInstallationGuard<'_>, String> {
        crate::update_queue_storage::update_update_queue(root, |queue| {
            if queue["run"]["state"] != "running" {
                return Err("the update queue stopped before installation could begin".into());
            }
            crate::check_update_cancelled::check_update_cancelled(root)?;
            let active_id = queue["run"]["activeItemId"].as_str().map(str::to_owned);
            queue["run"]["installing"] = serde_json::Value::Bool(true);
            queue["run"]["cancelable"] = serde_json::Value::Bool(true);
            if let Some(active_id) = active_id
                && let Some(item) = queue["items"].as_array_mut().and_then(|items| {
                    items
                        .iter_mut()
                        .find(|item| item["id"].as_str() == Some(&active_id))
                })
            {
                item["state"] = serde_json::Value::String("installing".into());
                item["message"] = serde_json::Value::String("Applying the verified update".into());
            }
            Ok(())
        })?;
        Ok(QueueInstallationGuard { root })
    }

    fn ensure_installation_can_continue(&self) -> Result<(), String> {
        crate::update_queue_storage::update_update_queue(self.root, |queue| {
            if queue["run"]["state"] != "running" || queue["run"]["installing"] != true {
                return Err("the update queue stopped before installation could begin".into());
            }
            crate::check_update_cancelled::check_update_cancelled(self.root)?;
            // Applying files is cancellable because every interruption rolls
            // back from the verified backup before the queue worker exits.
            queue["run"]["cancelable"] = serde_json::Value::Bool(true);
            Ok(())
        })
    }
}

impl Drop for QueueInstallationGuard<'_> {
    fn drop(&mut self) {
        if let Err(error) = crate::update_queue_storage::update_update_queue(self.root, |queue| {
            queue["run"]["installing"] = serde_json::Value::Bool(false);
            Ok(())
        }) {
            append_log(
                self.root,
                'W',
                &format!("Could not release the update installation state: {error}"),
            );
        }
    }
}

pub(crate) fn run_install() -> Result<(), String> {
    let arguments = Arguments::read()?;
    install_staged(&arguments.root, &arguments.staged)
}
pub(crate) fn install_staged(root: &Path, staged: &Path) -> Result<(), String> {
    super::scheduled::write_status(
        root,
        "waitingForGame",
        "Update is verified. Waiting for Enshrouded to exit",
    );
    append_log(
        root,
        'D',
        "Independent updater is waiting for Enshrouded processes to exit",
    );
    if let Err(error) = super::scheduled::wait_for_game_processes(root) {
        append_log(
            root,
            'E',
            &format!("Game process scan failed. Update was not installed: {error}"),
        );
        return Err(error);
    }
    let _installation = QueueInstallationGuard::begin(root)?;
    super::scheduled::write_status(
        root,
        "installing",
        "Validating the verified ShroudForge update",
    );
    validate_roots(root, staged)?;
    let source = staged;
    if !source.join("shroudforge/version.json").is_file() {
        return Err("staged release does not contain shroudforge/version.json".into());
    }

    let stamp = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    super::scheduled::write_status(
        root,
        "installing",
        "Preparing rollback backup before replacing files",
    );
    append_log(
        root,
        'D',
        &format!("Starting update from {}", staged.display()),
    );

    let paths = managed_paths(source)?;
    let obsolete_paths = match obsolete_managed_paths(root, &paths) {
        Ok(paths) => paths,
        Err(error) => {
            append_log(
                root,
                'W',
                &format!(
                    "Old managed-file metadata is invalid. Preserving unknown mod files during repair: {error}"
                ),
            );
            Vec::new()
        }
    };
    let mut transaction_paths = paths.clone();
    transaction_paths.extend(obsolete_paths.iter().cloned());
    transaction_paths.sort();
    transaction_paths.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    // Validate every path before preserving or changing any installed file.
    for relative in &transaction_paths {
        let is_incoming = paths
            .iter()
            .any(|incoming| incoming.eq_ignore_ascii_case(relative));
        if is_incoming {
            validate_file_path(source, relative)?;
        }
        validate_file_path(root, relative)?;
        let incoming = source.join(relative);
        if is_incoming && !incoming.is_file() {
            return Err(format!("missing release file: {relative}"));
        }
    }
    super::scheduled::prepare_release_file_locks(root, &transaction_paths)?;
    // A single recycle-bin entry is both the rollback snapshot and the
    // recoverable copy of replaced or removed files.
    let backup_entry = shroudforge_loader_config::paths::recycle_bin_dir(root)
        .join(format!("system-update-{stamp}-{}", std::process::id()));
    let backup_id = backup_entry
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("invalid recycle-bin entry path")?;
    let backup = match crate::recycle_bin::preserve_update_files_with_progress_and_cancel(
        root,
        backup_id,
        "System update",
        &transaction_paths,
        Some(root),
        |completed, total, relative, copied, file_total| {
            crate::check_update_cancelled::check_update_cancelled(root)?;
            crate::write_update_status::write_file_progress(
                root,
                "installing",
                "Preparing rollback backup",
                completed,
                total,
                relative,
                copied,
                file_total,
            );
            Ok(())
        },
    ) {
        Ok(backup) => backup,
        Err(error) => {
            if let Err(cancel_error) = crate::check_update_cancelled::check_update_cancelled(root)
                && cancel_error == "UPDATE_CANCELLED"
            {
                return Err(cancel_error);
            }
            return Err(error);
        }
    };
    if let Err(error) = _installation.ensure_installation_can_continue() {
        return match fs::remove_dir_all(&backup_entry) {
            Ok(()) => Err(error),
            Err(cleanup_error) if cleanup_error.kind() == std::io::ErrorKind::NotFound => {
                Err(error)
            }
            Err(cleanup_error) => {
                let detail = format!(
                    "{error}. Could not remove the unused rollback copy at {}: {cleanup_error}",
                    backup_entry.display()
                );
                append_log(root, 'E', &detail);
                Err(detail)
            }
        };
    }
    super::scheduled::write_status(root, "installing", "Applying the verified update");
    let result = apply_with_progress_and_cancel(
        source,
        root,
        &paths,
        Some(root),
        |completed, total, relative, copied, file_total| {
            crate::check_update_cancelled::check_update_cancelled(root)?;
            crate::write_update_status::write_file_progress(
                root,
                "installing",
                "Installing update",
                completed,
                total,
                relative,
                copied,
                file_total,
            );
            Ok(())
        },
    )
    .and_then(|()| {
        for relative in &obsolete_paths {
            crate::check_update_cancelled::check_update_cancelled(root)?;
            let current = installed_path(root, relative);
            remove_entry(&current)?;
            if mod_relative_path(relative).is_some() {
                remove_empty_mod_directories(root, &current)?;
            }
        }
        crate::check_update_cancelled::check_update_cancelled(root)?;
        Ok(())
    });
    match result {
        Ok(()) => {
            let release: serde_json::Value = serde_json::from_slice(
                &fs::read(source.join("shroudforge/version.json"))
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
            let state = serde_json::json!({
                "schemaVersion": 1, "status": "installed", "version": release["version"],
                "build": release["build"], "installedAt": stamp, "backup": backup_entry
            });
            let repair_required = !super::missing_required_files(root).is_empty();
            let installed_version = release["version"]
                .as_str()
                .map(|version| version.trim_start_matches('v').to_owned());
            let installed_build = release["build"].as_u64().or_else(|| {
                release["build"]
                    .as_str()
                    .and_then(|build| build.parse::<u64>().ok())
            });
            if !repair_required
                && let Err(error) =
                    shroudforge_loader_config::config::update_loader(root, |config| {
                        let latest = &mut config["modules"]["updates"]["system"]["latestRelease"];
                        let same_release = latest["baseVersion"].as_str()
                            == installed_version.as_deref()
                            && latest["build"].as_u64() == installed_build;
                        if same_release {
                            latest["updateAvailable"] = serde_json::Value::Bool(false);
                            latest["mandatory"] = serde_json::Value::Bool(false);
                            latest["repairRequired"] = serde_json::Value::Bool(false);
                        }
                        Ok(())
                    })
            {
                append_log(
                    root,
                    'W',
                    &format!(
                        "Installed update, but could not refresh cached release status: {error}"
                    ),
                );
            }
            if let Err(error) = shroudforge_loader_config::config::write_document(
                root,
                crate::documents::STATE,
                &state,
            ) {
                append_log(
                    root,
                    'E',
                    &format!("Installed files, but could not save update state: {error}"),
                );
            }
            let _ = fs::remove_file(
                shroudforge_loader_config::paths::updates_dir(root).join("pending.ready"),
            );
            let status = serde_json::json!({"schemaVersion":1,"operation":"systemStage","status":"installed","step":"complete","message":"System update installed successfully","version":release["version"],"updatedAt":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()});
            let _ = shroudforge_loader_config::config::write_json(
                &shroudforge_loader_config::paths::updates_dir(root).join("updater-status.json"),
                &status,
            );
            append_log(
                root,
                'I',
                &format!(
                    "Update installed. Recycle-bin entry={}",
                    backup_entry.display()
                ),
            );
            Ok(())
        }
        Err(error) => {
            rollback_failed_installation(root, &backup, &transaction_paths, &backup_entry, error)
        }
    }
}

fn rollback_failed_installation(
    root: &Path,
    backup: &Path,
    transaction_paths: &[String],
    backup_entry: &Path,
    error: String,
) -> Result<(), String> {
    append_log(
        root,
        'E',
        &format!("Update failed: {error}. Restoring backup"),
    );
    super::scheduled::write_status(
        root,
        "restoring",
        &format!("Update failed. Restoring the previous files: {error}"),
    );
    if let Err(rollback_error) = restore_with_progress(
        backup,
        root,
        transaction_paths,
        |completed, total, relative, copied, file_total| {
            crate::write_update_status::write_file_progress(
                root,
                "restoring",
                "Restoring previous files",
                completed,
                total,
                relative,
                copied,
                file_total,
            );
            Ok(())
        },
    ) {
        let detail = format!(
            "{error}. Rollback also failed: {rollback_error}. The recovery copy remains at {}",
            backup_entry.display()
        );
        append_log(root, 'E', &detail);
        return Err(detail);
    }
    if crate::check_update_cancelled::check_update_cancelled(root)
        .is_err_and(|cancel_error| cancel_error == "UPDATE_CANCELLED")
    {
        return Err("UPDATE_CANCELLED".into());
    }
    Err(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupted_installation_restores_its_backup_before_cancelling() {
        let root = tempfile::tempdir().unwrap();
        let program_file = root.path().join("shroudforge/example.dll");
        fs::create_dir_all(program_file.parent().unwrap()).unwrap();
        let version_file = root.path().join("shroudforge/version.json");
        let old_manifest = serde_json::json!({
            "version": "1.0.0",
            "build": 1,
            "managedPaths": ["shroudforge/example.dll", "shroudforge/version.json"]
        });
        let new_manifest = serde_json::json!({
            "version": "2.0.0",
            "build": 2,
            "managedPaths": ["shroudforge/example.dll", "shroudforge/version.json"]
        });
        fs::write(&program_file, b"original program file").unwrap();
        fs::write(&version_file, serde_json::to_vec(&old_manifest).unwrap()).unwrap();
        let paths = vec![
            "shroudforge/example.dll".to_owned(),
            "shroudforge/version.json".to_owned(),
        ];
        let backup_entry = shroudforge_loader_config::paths::recycle_bin_dir(root.path())
            .join("system-update-test");
        let backup = crate::recycle_bin::preserve_update_files(
            root.path(),
            "system-update-test",
            "System update test",
            &paths,
        )
        .unwrap();

        fs::write(&program_file, b"partially installed file").unwrap();
        fs::write(&version_file, serde_json::to_vec(&new_manifest).unwrap()).unwrap();
        let updates = shroudforge_loader_config::paths::updates_dir(root.path());
        fs::create_dir_all(&updates).unwrap();
        fs::write(updates.join("cancel-system-update"), b"cancel").unwrap();

        let result = rollback_failed_installation(
            root.path(),
            &backup,
            &paths,
            &backup_entry,
            "UPDATE_CANCELLED".into(),
        );

        assert_eq!(result.unwrap_err(), "UPDATE_CANCELLED");
        assert_eq!(fs::read(&program_file).unwrap(), b"original program file");
        assert_eq!(
            fs::read(&version_file).unwrap(),
            serde_json::to_vec(&old_manifest).unwrap()
        );
    }
}
