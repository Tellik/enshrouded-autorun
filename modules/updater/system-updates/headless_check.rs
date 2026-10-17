use crate::*;
#[cfg(windows)]
pub(crate) fn headless_check_system_updates(
    root: &std::path::Path,
    mode: crate::update_check::SystemUpdateCheckMode,
) -> Result<String, String> {
    super::refresh_cached_repair_status(root)?;
    let config = shroudforge_loader_config::config::read_loader(root)?;
    let _check_lock = match crate::update_check::begin_system_update_check(root, mode)? {
        crate::update_check::SystemUpdateCheckStart::Started(lock) => lock,
        crate::update_check::SystemUpdateCheckStart::CoolingDown { remaining_seconds } => {
            return Ok(format!(
                "A system update check was attempted recently. The next automatic check is allowed in {remaining_seconds} seconds."
            ));
        }
        crate::update_check::SystemUpdateCheckStart::AlreadyRunning => {
            return Ok("System update check is already running in another process.".into());
        }
        crate::update_check::SystemUpdateCheckStart::Disabled => {
            return Ok("Automatic system update checks are disabled.".into());
        }
    };

    let mut provider = crate::provider_config::default_system_provider();
    provider.include_prereleases = config
        .pointer("/modules/updates/system/includePrereleases")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let current = std::fs::read(shroudforge_loader_config::paths::version_file(root))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
    let current_version = current
        .as_ref()
        .and_then(|value| value["version"].as_str())
        .unwrap_or(env!("CARGO_PKG_VERSION"));
    let current_build = current
        .as_ref()
        .and_then(|value| value["build"].as_u64())
        .or_else(|| {
            current
                .as_ref()
                .and_then(|value| value["build"].as_str()?.parse().ok())
        })
        .unwrap_or(0);
    let result = crate::fetch_system_release::fetch_latest(&provider, current_version)?;
    let Some(release) = result else {
        let checked_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        shroudforge_loader_config::config::update_loader(root, |value| {
            value["modules"]["updates"]["system"]["latestRelease"] = serde_json::Value::Null;
            value["modules"]["updates"]["system"]["lastCheckAt"] = serde_json::json!(checked_at);
            Ok(())
        })?;
        return Ok("No compatible ShroudForge release was found".into());
    };
    let repair_required = !super::missing_required_files(root).is_empty();
    let update_available = repair_required
        || crate::is_update_available(
            current_version,
            current_build,
            &release.base_version,
            release.build,
        );
    let mandatory = release.mandatory || repair_required;
    let checked_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    shroudforge_loader_config::config::update_loader(root, |value| {
        value["modules"]["updates"]["system"]["lastCheckAt"] = serde_json::json!(checked_at);
        value["modules"]["updates"]["system"]["latestRelease"] = serde_json::json!({
            "version": release.version,
            "baseVersion": release.base_version,
            "build": release.build,
            "downloadUrl": release.download_url,
            "checksum": release.checksum,
            "releaseUrl": release.release_url,
            "message": release.message.chars().take(20_000).collect::<String>(),
            "prerelease": release.prerelease,
            "mandatory": mandatory,
            "repairRequired": repair_required,
            "updateAvailable": update_available,
            "checkedAt": checked_at
        });
        Ok(())
    })?;
    Ok(if update_available {
        format!("ShroudForge update {} is available", release.version)
    } else {
        "ShroudForge is up to date".into()
    })
}

#[cfg(windows)]
pub fn startup_check(root: &std::path::Path) -> Result<bool, String> {
    let missing = missing_required_files(root);
    if !missing.is_empty() {
        let detail = format!(
            "Installation requires repair. Missing or invalid files: {}",
            missing.join(", ")
        );
        let _ = shroudforge_logging::append(root, 'E', "updater", &detail);
        scheduled::open_update_window(root).map_err(|error| {
            let error = format!("{detail}. Could not open the repair updater: {error}");
            let _ = shroudforge_logging::append(root, 'E', "updater", &error);
            error
        })?;
        return Ok(true);
    }
    if let Err(error) =
        headless_check_system_updates(root, crate::update_check::SystemUpdateCheckMode::Automatic)
    {
        let _ = shroudforge_logging::append(
            root,
            'W',
            "updater",
            &format!("Startup update check did not complete. Game startup continues: {error}"),
        );
        return Ok(false);
    }

    let release = match shroudforge_loader_config::config::read_loader(root)
        .ok()
        .and_then(|config| config.pointer("/modules/updates/system/latestRelease").cloned())
    {
        Some(release) if release["updateAvailable"].as_bool() == Some(true) => release,
        _ => return Ok(false),
    };
    let Some(version) = release["version"].as_str() else {
        return Ok(false);
    };
    let mandatory = release["mandatory"].as_bool() == Some(true);
    let previous_prompt = shroudforge_loader_config::config::read_loader(root)
        .ok()
        .and_then(|config| {
            config
                .pointer("/modules/updates/system/startupPromptedVersion")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        });
    if !mandatory && previous_prompt.as_deref() == Some(version) {
        return Ok(false);
    }

    match scheduled::open_update_window(root) {
        Ok(()) => {
            if !mandatory {
                let version = version.to_owned();
                if let Err(error) = shroudforge_loader_config::config::update_loader(root, |config| {
                    config["modules"]["updates"]["system"]["startupPromptedVersion"] =
                        serde_json::Value::String(version);
                    Ok(())
                }) {
                    let _ = shroudforge_logging::append(root, 'W', "updater",
                        &format!("Could not save the startup update prompt: {error}"));
                }
            }
            Ok(mandatory)
        }
        Err(error) => {
            let _ = shroudforge_logging::append(
                root,
                'E',
                "updater",
                &format!("Startup update window could not be opened: {error}"),
            );
            if mandatory { Err(error) } else { Ok(false) }
        }
    }
}
#[cfg(not(windows))]
pub fn startup_check(_: &std::path::Path) -> Result<bool, String> {
    Ok(false)
}
