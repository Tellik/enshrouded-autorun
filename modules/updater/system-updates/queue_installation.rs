use std::path::Path;

pub fn queue_latest_installation(root: &Path) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        let config = shroudforge_loader_config::config::read_loader(root)?;
        let mut provider = crate::provider_config::default_system_provider();
        provider.include_prereleases = config
            .pointer("/modules/updates/system/includePrereleases")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let installed = crate::installed_release(root);
        let release = crate::fetch_system_release::fetch_latest(&provider, &installed.version)?
            .ok_or("No compatible ShroudForge release is available for installation.")?;
        let version = release.version;
        let payload = serde_json::json!({
            "version": version.clone(),
            "installerAction": true,
            "baseVersion": release.base_version,
            "build": release.build,
            "downloadUrl": release.download_url,
            "checksum": release.checksum,
            "releaseUrl": release.release_url,
            "prerelease": release.prerelease,
            "mandatory": release.mandatory,
            "message": release.message
        });
        let queue = crate::read_update_queue(root);
        if queue["items"].as_array().is_some_and(|items| {
            items.iter().any(|item| {
                matches!(item["kind"].as_str(), Some("system" | "installer"))
                    && item["version"].as_str() == Some(version.as_str())
                    && !matches!(
                        item["state"].as_str(),
                        Some("complete" | "cancelled" | "error")
                    )
            })
        }) {
            return Err(
                "A ShroudForge installation task for this release is already queued.".into(),
            );
        }
        let id = format!("installer:install-latest:{version}");
        crate::enqueue_update_item(
            root,
            serde_json::json!({
                "id":id,
                "kind":"installer",
                "operation":"install-latest",
                "title":"Install or repair ShroudForge",
                "version":version,
                "state":"queued",
                "selected":true,
                "payload":payload
            }),
        )?;
        Ok(serde_json::json!({
            "queued":true,
            "successMessage":"updater.installation.queued"
        }))
    }
    #[cfg(not(windows))]
    {
        let _ = root;
        Err("ShroudForge installation is available on Windows only".into())
    }
}

pub fn queue_uninstall(
    root: &Path,
    remove_mods: bool,
    reset_user_settings: bool,
    purge_data: bool,
) -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        if !shroudforge_loader_config::paths::shroudforge_program_files_present(root) {
            return Err("No ShroudForge program files were found in this game folder.".into());
        }
        let id = "installer:uninstall";
        crate::enqueue_update_item(
            root,
            serde_json::json!({
                "id":id,
                "kind":"installer",
                "operation":"uninstall",
                "title":"Uninstall ShroudForge",
                "state":"queued",
                "selected":true,
                "payload":{
                    "removeMods":remove_mods,
                    "resetUserSettings":reset_user_settings && !purge_data,
                    "purgeData":purge_data
                }
            }),
        )?;
        Ok(serde_json::json!({
            "queued":true,
            "successMessage":"updater.uninstall.queued"
        }))
    }
    #[cfg(not(windows))]
    {
        let _ = (root, remove_mods, reset_user_settings, purge_data);
        Err("ShroudForge uninstallation is available on Windows only".into())
    }
}
