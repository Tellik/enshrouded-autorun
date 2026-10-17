use crate::apply_release_files::{apply, copy_entry};
use crate::read_install_arguments::Arguments;
use crate::read_managed_release_files::{managed_paths, obsolete_managed_mod_paths};
use crate::remove_release_files::{remove_empty_mod_directories, remove_entry};
use crate::restore_release_files::restore;
use crate::validate_update_paths::{installed_path, validate_file_path, validate_roots};
use crate::windows::append_log;
use std::{fs, time::SystemTime};

pub(crate) fn run_install() -> Result<(), String> {
    let arguments = Arguments::read()?;
    super::scheduled::write_status(
        &arguments.root,
        "waitingForGame",
        "Update is verified. Waiting for Enshrouded to exit",
    );
    append_log(
        &arguments.root,
        'D',
        "Independent updater is waiting for Enshrouded processes to exit",
    );
    if let Err(error) = super::scheduled::wait_for_game_processes(&arguments.root) {
        append_log(
            &arguments.root,
            'E',
            &format!("Game process scan failed. Update was not installed: {error}"),
        );
        return Err(error);
    }
    super::scheduled::check_update_cancelled(&arguments.root)?;
    super::scheduled::write_status(
        &arguments.root,
        "installing",
        "Validating the verified ShroudForge update",
    );
    validate_roots(&arguments.root, &arguments.staged)?;
    let source = arguments.staged.clone();
    if !source.join("shroudforge/version.json").is_file() {
        return Err("staged release does not contain shroudforge/version.json".into());
    }

    let stamp = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let backup = shroudforge_loader_config::paths::updates_dir(&arguments.root)
        .join("backups")
        .join(stamp.to_string());
    fs::create_dir_all(&backup).map_err(|error| error.to_string())?;
    super::scheduled::write_status(
        &arguments.root,
        "installing",
        "Preparing rollback backup before replacing files",
    );
    append_log(
        &arguments.root,
        'D',
        &format!("Starting update from {}", arguments.staged.display()),
    );

    let paths = managed_paths(&source)?;
    let obsolete_mod_paths = match obsolete_managed_mod_paths(&arguments.root, &paths) {
        Ok(paths) => paths,
        Err(error) => {
            append_log(
                &arguments.root,
                'W',
                &format!(
                    "Old managed-file metadata is invalid. Preserving unknown mod files during repair: {error}"
                ),
            );
            Vec::new()
        }
    };
    let mut transaction_paths = paths.clone();
    transaction_paths.extend(obsolete_mod_paths.iter().cloned());
    transaction_paths.sort();
    transaction_paths.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    // Complete and verify backup before the first installed file changes.
    for relative in &transaction_paths {
        let is_incoming = paths
            .iter()
            .any(|incoming| incoming.eq_ignore_ascii_case(relative));
        if is_incoming {
            validate_file_path(&source, relative)?;
        }
        validate_file_path(&arguments.root, relative)?;
        let incoming = source.join(relative);
        if is_incoming && !incoming.is_file() {
            return Err(format!("missing release file: {relative}"));
        }
        let current = installed_path(&arguments.root, relative);
        if current.exists() {
            copy_entry(&current, &backup.join(relative))?;
        }
    }
    super::scheduled::write_status(
        &arguments.root,
        "installing",
        "Applying the verified update",
    );
    let result = apply(&source, &arguments.root, &paths).and_then(|()| {
        for relative in &obsolete_mod_paths {
            let current = installed_path(&arguments.root, relative);
            remove_entry(&current)?;
            remove_empty_mod_directories(&arguments.root, &current)?;
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
                "build": release["build"], "installedAt": stamp, "backup": backup
            });
            let repair_required = !super::missing_required_files(&arguments.root).is_empty();
            let installed_version = release["version"]
                .as_str()
                .map(|version| version.trim_start_matches('v').to_owned());
            let installed_build = release["build"].as_u64().or_else(|| {
                release["build"]
                    .as_str()
                    .and_then(|build| build.parse::<u64>().ok())
            });
            if !repair_required {
                if let Err(error) =
                    shroudforge_loader_config::config::update_loader(&arguments.root, |config| {
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
                        &arguments.root,
                        'W',
                        &format!(
                            "Installed update, but could not refresh cached release status: {error}"
                        ),
                    );
                }
            }
            if let Err(error) = shroudforge_loader_config::config::write_document(
                &arguments.root,
                crate::documents::STATE,
                &state,
            ) {
                append_log(
                    &arguments.root,
                    'E',
                    &format!("Installed files, but could not save update state: {error}"),
                );
            }
            let _ = fs::remove_file(
                shroudforge_loader_config::paths::updates_dir(&arguments.root)
                    .join("pending.ready"),
            );
            let status = serde_json::json!({"schemaVersion":1,"operation":"systemStage","status":"installed","step":"complete","message":"System update installed successfully","version":release["version"],"updatedAt":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()});
            let _ = shroudforge_loader_config::config::write_json(
                &shroudforge_loader_config::paths::updates_dir(&arguments.root)
                    .join("updater-status.json"),
                &status,
            );
            append_log(
                &arguments.root,
                'I',
                &format!("Update installed. Backup={}", backup.display()),
            );
            Ok(())
        }
        Err(error) => {
            append_log(
                &arguments.root,
                'E',
                &format!("Update failed: {error}. Restoring backup"),
            );
            if let Err(rollback_error) = restore(&backup, &arguments.root, &transaction_paths) {
                append_log(
                    &arguments.root,
                    'E',
                    &format!("Rollback failed: {rollback_error}"),
                );
                return Err(format!("{error}. Rollback also failed: {rollback_error}"));
            }
            Err(error)
        }
    }
}
