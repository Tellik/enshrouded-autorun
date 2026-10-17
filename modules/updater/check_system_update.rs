use crate::command_result::AsyncUiResult;
use crate::compare_release_versions::is_update_available;
use crate::fetch_system_release::fetch_latest;
use crate::provider_config::ProviderConfig;
use crate::release_state::ReleaseState;
use crate::restore_checked_release::restore_persisted_update_result;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub fn start_update_check(
    root: PathBuf,
    provider: ProviderConfig,
    state: Arc<Mutex<ReleaseState>>,
    results: mpsc::Sender<AsyncUiResult>,
    request_id: Option<String>,
) {
    let updater_status_path =
        shroudforge_loader_config::paths::updates_dir(&root).join("updater-status.json");
    let updater_status = std::fs::read(&updater_status_path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|value| value["status"].as_str().map(str::to_owned));
    if let Ok(mut value) = state.lock() {
        if value.state == "queued"
            && matches!(updater_status.as_deref(), Some("error" | "installed"))
        {
            value.state = "ready".into();
        }
        if value.state == "checking" && request_id.is_none() {
            return;
        }
        if value.state == "downloading" || value.state == "queued" {
            if let Some(request_id) = request_id {
                let _ = results.send(AsyncUiResult {
                    request_id,
                    source: "Updates".into(),
                    action: "Check for updates".into(),
                    success_message: "updates.check.upToDate".into(),
                    result: Err("An update operation is already in progress.".into()),
                });
            }
            return;
        }
        if !provider.enabled {
            value.state = "idle".into();
            value.message = Some("GitHub system updates are disabled.".into());
            if let Some(request_id) = request_id {
                let _ = results.send(AsyncUiResult {
                    request_id,
                    source: "Updates".into(),
                    action: "Check for updates".into(),
                    success_message: "updates.check.upToDate".into(),
                    result: Err("System update checks are disabled in settings.".into()),
                });
            }
            return;
        }
        value.state = "checking".into();
        value.message = Some("Checking releases…".into());
    } else {
        if let Some(request_id) = request_id {
            let _ = results.send(AsyncUiResult {
                request_id,
                source: "Updates".into(),
                action: "Check for updates".into(),
                success_message: "updates.check.upToDate".into(),
                result: Err("Update status is unavailable.".into()),
            });
        }
        return;
    }
    let manual = request_id.is_some();
    thread::spawn(move || {
        let repair_status = crate::refresh_cached_repair_status(&root);
        let repair_required = matches!(&repair_status, Ok(true));
        if repair_required
            && !manual
            && shroudforge_loader_config::config::read_loader(&root).is_ok_and(|config| {
                config["modules"]["updates"]["system"]["latestRelease"].is_object()
            })
        {
            restore_persisted_update_result(&root, &state);
            let _ = shroudforge_logging::append(
                &root,
                'D',
                "updates",
                "Required installation files are missing. Reusing the verified release for repair",
            );
            return;
        }
        let check_start = repair_status.and_then(|repair_required| {
            let repair_without_release = repair_required
                    && !shroudforge_loader_config::config::read_loader(&root)?["modules"]
                        ["updates"]["system"]["latestRelease"]
                        .is_object();
            crate::update_check::begin_system_update_check(&root, manual || repair_without_release)
        });
        let _check_lock = match check_start {
            Ok(crate::update_check::SystemUpdateCheckStart::Started(lock)) => lock,
            Ok(crate::update_check::SystemUpdateCheckStart::CoolingDown { remaining_seconds }) => {
                restore_persisted_update_result(&root, &state);
                if let Ok(mut value) = state.lock() {
                    if value.message.is_none() {
                        value.message = Some(format!(
                            "A recent update check was reused. Another automatic check is allowed in {remaining_seconds} seconds."
                        ));
                    }
                }
                let _ = shroudforge_logging::append(
                    &root,
                    'D',
                    "updates",
                    &format!(
                        "Recent persisted update check result reused. {remaining_seconds} seconds remain in the automatic cooldown"
                    ),
                );
                if let Some(request_id) = request_id {
                    let update_available = state
                        .lock()
                        .map(|value| value.update_available)
                        .unwrap_or(false);
                    let detail = if update_available {
                        "A ShroudForge update is available."
                    } else {
                        "ShroudForge is up to date."
                    };
                    let _ = results.send(AsyncUiResult {
                        request_id,
                        source: "Updates".into(),
                        action: "Check for updates".into(),
                        success_message: if update_available {
                            "updates.check.available"
                        } else {
                            "updates.check.upToDate"
                        }
                        .into(),
                        result: Ok(detail.into()),
                    });
                }
                return;
            }
            Ok(crate::update_check::SystemUpdateCheckStart::AlreadyRunning) => {
                restore_persisted_update_result(&root, &state);
                if let Ok(mut value) = state.lock() {
                    if value.state != "ready" {
                        value.state = "idle".into();
                        value.message = Some(
                            "Another ShroudForge process is already checking for updates.".into(),
                        );
                    }
                }
                let _ = shroudforge_logging::append(
                    &root,
                    'D',
                    "updates",
                    "Update check remained active after the coordination wait",
                );
                if let Some(request_id) = request_id {
                    let _ = results.send(AsyncUiResult {
                        request_id,
                        source: "Updates".into(),
                        action: "Check for updates".into(),
                        success_message: "updates.check.upToDate".into(),
                        result: Err("An update check is already running.".into()),
                    });
                }
                return;
            }
            Err(error) => {
                if let Ok(mut value) = state.lock() {
                    value.state = "error".into();
                    value.message = Some(error.clone());
                }
                let _ = shroudforge_logging::append(
                    &root,
                    'E',
                    "updates",
                    &format!("Could not coordinate system update check: {error}"),
                );
                if let Some(request_id) = request_id {
                    let _ = results.send(AsyncUiResult {
                        request_id,
                        source: "Updates".into(),
                        action: "Check for updates".into(),
                        success_message: "updates.check.upToDate".into(),
                        result: Err(error),
                    });
                }
                return;
            }
        };
        let (current, current_build) = state
            .lock()
            .map(|value| (value.current_version.clone(), value.current_build))
            .unwrap_or_default();
        let result = fetch_latest(&provider, &current);
        let mut outcome = Err("Update status is unavailable.".to_owned());
        if let Ok(mut value) = state.lock() {
            match result {
                Ok(Some(mut release)) => {
                    let repair_required = !crate::missing_required_files(&root).is_empty();
                    release.mandatory |= repair_required;
                    value.update_available = repair_required
                        || is_update_available(
                            &current,
                            current_build,
                            &release.base_version,
                            release.build,
                        );
                    value.mandatory = release.mandatory;
                    value.latest_version = Some(release.version.clone());
                    value.prerelease = release.prerelease;
                    value.release_url = release.release_url.clone();
                    value.message = Some(if value.update_available {
                        release.message.clone()
                    } else {
                        "ShroudForge is up to date.".into()
                    });
                    let update_available = value.update_available;
                    let latest_version = release.version.clone();
                    let checked_at = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let persisted_release = serde_json::json!({
                        "version":release.version.clone(),
                        "baseVersion":release.base_version.clone(),
                        "build":release.build,
                        "downloadUrl":release.download_url.clone(),
                        "checksum":release.checksum.clone(),
                        "releaseUrl":release.release_url.clone(),
                        "message":release.message.chars().take(20_000).collect::<String>(),
                        "prerelease":release.prerelease,
                        "mandatory":release.mandatory,
                        "repairRequired":repair_required,
                        "updateAvailable":update_available,
                        "checkedAt":checked_at
                    });
                    let persisted =
                        shroudforge_loader_config::config::update_loader(&root, |config| {
                            config["modules"]["updates"]["system"]["lastCheckAt"] =
                                serde_json::json!(checked_at);
                            config["modules"]["updates"]["system"]["latestRelease"] =
                                persisted_release;
                            Ok(())
                        });
                    value.release = Some(release);
                    match persisted {
                        Ok(()) => {
                            value.state = "ready".into();
                            outcome = Ok(if update_available {
                                format!("Update {latest_version} is available.")
                            } else {
                                "ShroudForge is up to date.".into()
                            });
                        }
                        Err(error) => {
                            let error = format!(
                                "Could not persist checked release in modloader-config.json: {error}"
                            );
                            value.state = "error".into();
                            value.message = Some(error.clone());
                            outcome = Err(error);
                        }
                    }
                }
                Ok(None) => {
                    value.update_available = false;
                    value.mandatory = repair_required;
                    value.prerelease = false;
                    value.latest_version = None;
                    value.release_url = None;
                    value.release = None;
                    value.message =
                        Some("No ShroudForge release has been published on GitHub yet.".into());
                    let checked_at = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    match shroudforge_loader_config::config::update_loader(&root, |config| {
                        config["modules"]["updates"]["system"]["lastCheckAt"] =
                            serde_json::json!(checked_at);
                        config["modules"]["updates"]["system"]["latestRelease"] =
                            serde_json::Value::Null;
                        Ok(())
                    }) {
                        Ok(()) => {
                            if repair_required {
                                let error = "Required ShroudForge files are missing, but no compatible release is available for repair.".to_owned();
                                value.state = "error".into();
                                value.message = Some(error.clone());
                                outcome = Err(error);
                            } else {
                                value.state = "ready".into();
                                outcome =
                                    Ok("No ShroudForge release has been published yet.".into());
                            }
                        }
                        Err(error) => {
                            let error =
                                format!("Could not persist empty update check result: {error}");
                            value.state = "error".into();
                            value.message = Some(error.clone());
                            outcome = Err(error);
                        }
                    }
                }
                Err(error) => {
                    let error = if repair_required {
                        format!(
                            "Required ShroudForge files are missing. Could not check the release needed for repair: {error}"
                        )
                    } else {
                        error
                    };
                    value.mandatory |= repair_required;
                    value.state = "error".into();
                    value.message = Some(error.clone());
                    outcome = Err(error);
                }
            }
        }
        let (level, message) = match &outcome {
            Ok(detail) => (
                if detail == "ShroudForge is up to date." {
                    'D'
                } else {
                    'I'
                },
                format!(
                    "{} update check completed: {detail}",
                    if manual { "Manual" } else { "Automatic" }
                ),
            ),
            Err(error) => (
                'W',
                format!(
                    "{} update check failed: {error}",
                    if manual { "Manual" } else { "Automatic" }
                ),
            ),
        };
        let _ = shroudforge_logging::append(&root, level, "updates", &message);
        if let Some(request_id) = request_id {
            let success_message = match outcome {
                Ok(ref detail) if detail == "ShroudForge is up to date." => {
                    "updates.check.upToDate"
                }
                Ok(ref detail) if detail == "No ShroudForge release has been published yet." => {
                    "updates.check.none"
                }
                Ok(_) => "updates.check.available",
                Err(_) => "updates.check.upToDate",
            };
            let _ = results.send(AsyncUiResult {
                request_id,
                source: "Updates".into(),
                action: "Check for updates".into(),
                success_message: success_message.into(),
                result: outcome,
            });
        }
    });
}
