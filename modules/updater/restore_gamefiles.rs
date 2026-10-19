use std::path::Path;

pub fn request_gamefiles_restore(root: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        let _operation_lock = crate::acquire_queue_worker_mutex(root)?
            .ok_or("an update or recovery action is already running")?;
        crate::start_update_queue::ensure_gamefiles_restore_idle(root)?;
        let queue = crate::read_update_queue_value(root);
        if let Some(error) = queue["error"].as_str() {
            return Err(error.to_owned());
        }
        if queue["run"]["state"] == "running" {
            return Err("game files cannot be restored while an update is running".into());
        }
        let target = if root.join("enshrouded_server.exe").is_file() {
            "server"
        } else {
            "client"
        };
        if shroudforge_game_assets::originals::originals_status(root, target)["status"] != "ready" {
            return Err(
                "verified original game files are not available for this game build".into(),
            );
        }
        disable_all_mods_for_gamefile_restore(root)?;
        shroudforge_game_assets::request_gamefiles_restore(root)
    }

    #[cfg(not(windows))]
    {
        shroudforge_game_assets::request_gamefiles_restore(root)
    }
}

#[cfg(windows)]
fn disable_all_mods_for_gamefile_restore(root: &Path) -> Result<(), String> {
    let enabled = read_installed_mod_configurations(root)
        .into_iter()
        .filter(|mod_config| mod_config.enabled)
        .collect::<Vec<_>>();
    let mut errors = Vec::new();
    let mut disabled = Vec::new();
    for mod_config in &enabled {
        match shroudforge_package::set_mod_enabled(
            root,
            &mod_config.id,
            false,
            &mod_config.revision,
        ) {
            Ok(()) => disabled.push(mod_config.id.clone()),
            Err(error) => errors.push(format!("{}: {error}", mod_config.id)),
        }
    }
    if errors.is_empty() {
        return Ok(());
    }

    let current = read_installed_mod_configurations(root);
    for id in &disabled {
        if let Some(mod_config) = current.iter().find(|mod_config| &mod_config.id == id)
            && let Err(error) =
                shroudforge_package::set_mod_enabled(root, id, true, &mod_config.revision)
        {
            errors.push(format!(
                "could not restore {id} to its previous enabled state: {error}"
            ));
        }
    }
    Err(format!(
        "Could not disable every mod. Game files were not queued for restoration. {}",
        errors.join(", ")
    ))
}

#[cfg(windows)]
struct InstalledModConfiguration {
    id: String,
    enabled: bool,
    revision: String,
}

#[cfg(windows)]
fn read_installed_mod_configurations(root: &Path) -> Vec<InstalledModConfiguration> {
    let Ok(entries) = std::fs::read_dir(shroudforge_loader_config::paths::mods_dir(root)) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let manifest =
                shroudforge_package::config::read_manifest_path(root, &entry.path()).ok()?;
            let enabled = manifest.enabled;
            let id = manifest.id;
            let state = serde_json::json!({
                "enabled": enabled,
                "settings": manifest.setting_values
            });
            Some(InstalledModConfiguration {
                id,
                enabled,
                revision: shroudforge_package::config_revision(&state),
            })
        })
        .collect()
}
