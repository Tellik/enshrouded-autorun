use shroudforge_updater::installed_release;
use shroudforge_updater::installed_version;
use shroudforge_updater::release::Release;
use shroudforge_updater::release_state::ReleaseState;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

pub(crate) fn read_release_status(
    root: &Path,
    updater_window: bool,
    central: &serde_json::Value,
    release: &Arc<Mutex<ReleaseState>>,
) -> ReleaseState {
    let mut release_snapshot = release
        .lock()
        .map(|value| value.clone())
        .unwrap_or_else(|_| ReleaseState::idle(installed_release(root), false));
    if let Some(latest) = central.pointer("/modules/updates/system/latestRelease") {
        if latest.is_object() {
            if let (
                Some(version),
                Some(base_version),
                Some(build),
                Some(download_url),
                Some(checksum),
            ) = (
                latest["version"].as_str(),
                latest["baseVersion"].as_str(),
                latest["build"].as_u64(),
                latest["downloadUrl"].as_str(),
                latest["checksum"].as_str(),
            ) {
                let update_available = latest["updateAvailable"].as_bool().unwrap_or(false);
                let mandatory = latest["mandatory"].as_bool().unwrap_or(false);
                let prerelease = latest["prerelease"].as_bool().unwrap_or(false);
                let release_url = latest["releaseUrl"].as_str().map(str::to_owned);
                let message = latest["message"].as_str().unwrap_or_default().to_owned();
                release_snapshot.latest_version = Some(version.to_owned());
                release_snapshot.update_available = update_available;
                release_snapshot.mandatory = mandatory;
                release_snapshot.prerelease = prerelease;
                release_snapshot.release_url = release_url.clone();
                if release_snapshot.state == "idle" || release_snapshot.state == "ready" {
                    release_snapshot.state = "ready".into();
                    release_snapshot.message = Some(if update_available {
                        message.clone()
                    } else {
                        "ShroudForge is up to date.".into()
                    });
                }
                release_snapshot.release = Some(Release {
                    version: version.to_owned(),
                    base_version: base_version.to_owned(),
                    build,
                    download_url: download_url.to_owned(),
                    checksum: checksum.to_owned(),
                    release_url,
                    message,
                    mandatory,
                    prerelease,
                });
            }
        }
    }
    if let Ok(bytes) =
        fs::read(shroudforge_loader_config::paths::updates_dir(root).join("updater-status.json"))
    {
        if let Ok(status) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            let terminal = matches!(
                status["status"].as_str(),
                Some("installed" | "error" | "cancelled")
            );
            let last_check = central["modules"]["updates"]["system"]["lastCheckAt"].as_u64();
            let superseded = terminal
                && (release_snapshot.update_available
                    || matches!(release_snapshot.state.as_str(), "checking" | "error")
                    || status["updatedAt"]
                        .as_u64()
                        .zip(last_check)
                        .is_some_and(|(updated, checked)| updated < checked));
            if status["operation"] == "systemStage" && !superseded {
                if let Some(state) = status["status"].as_str() {
                    let staged = shroudforge_loader_config::paths::updates_dir(root)
                        .join("pending.ready")
                        .is_file();
                    release_snapshot.state = if state == "installed" && !updater_window
                        || (state == "staged" && !staged)
                    {
                        "idle".into()
                    } else {
                        state.to_owned()
                    };
                    release_snapshot.message = status["message"].as_str().map(str::to_owned);
                    release_snapshot.staged = state == "staged" && staged;
                    release_snapshot.latest_version = status["version"]
                        .as_str()
                        .map(str::to_owned)
                        .or(release_snapshot.latest_version);
                    release_snapshot.downloaded_bytes =
                        status["downloadedBytes"].as_u64().unwrap_or(0);
                    release_snapshot.total_bytes = status["totalBytes"].as_u64();
                    release_snapshot.bytes_per_second = status["bytesPerSecond"].as_u64();
                    if state == "installed" {
                        release_snapshot.current_version = installed_version(root);
                    }
                }
            }
        }
    }
    release_snapshot
}
