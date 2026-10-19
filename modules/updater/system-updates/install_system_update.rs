use crate::apply_release_files::apply;
use crate::read_install_arguments::Arguments;
use crate::read_managed_release_files::{managed_paths, obsolete_managed_paths};
use crate::remove_release_files::{remove_empty_mod_directories, remove_entry};
use crate::restore_release_files::restore;
use crate::validate_update_paths::{installed_path, validate_file_path, validate_roots};
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
            let active_id = queue["run"]["activeItemId"]
                .as_str()
                .map(str::to_owned);
            queue["run"]["installing"] = serde_json::Value::Bool(true);
            if let Some(active_id) = active_id
                && let Some(item) = queue["items"].as_array_mut().and_then(|items| {
                    items
                        .iter_mut()
                        .find(|item| item["id"].as_str() == Some(&active_id))
                })
            {
                item["state"] = serde_json::Value::String("installing".into());
                item["message"] =
                    serde_json::Value::String("Applying the verified update".into());
            }
            Ok(())
        })?;
        Ok(QueueInstallationGuard { root })
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
    let backup = crate::recycle_bin::preserve_update_files(
        root,
        backup_id,
        "System update",
        &transaction_paths,
    )?;
    super::scheduled::write_status(root, "installing", "Applying the verified update");
    let result = apply(source, root, &paths).and_then(|()| {
        for relative in &obsolete_paths {
            let current = installed_path(root, relative);
            remove_entry(&current)?;
            if relative.starts_with("mods/") {
                remove_empty_mod_directories(root, &current)?;
            }
        }
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
                && let Err(error) = shroudforge_loader_config::config::update_loader(
                    root,
                    |config| {
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
                    },
                )
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
            append_log(
                root,
                'E',
                &format!("Update failed: {error}. Restoring backup"),
            );
            if let Err(rollback_error) = restore(&backup, root, &transaction_paths) {
                append_log(root, 'E', &format!("Rollback failed: {rollback_error}"));
                return Err(format!("{error}. Rollback also failed: {rollback_error}"));
            }
            Err(error)
        }
    }
}
