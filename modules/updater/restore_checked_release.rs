use crate::release::Release;
use crate::release_state::ReleaseState;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

pub fn restore_persisted_update_result(root: &Path, state: &Arc<Mutex<ReleaseState>>) {
    let Ok(config) = shroudforge_loader_config::config::read_loader(root) else {
        return;
    };
    restore_persisted_update_result_from_config(&config, state);
}

pub(crate) fn restore_persisted_update_result_from_config(
    config: &serde_json::Value,
    state: &Arc<Mutex<ReleaseState>>,
) {
    let last_check_at = config
        .pointer("/modules/updates/system/lastCheckAt")
        .and_then(serde_json::Value::as_u64);
    let latest = config.pointer("/modules/updates/system/latestRelease");
    let Ok(mut value) = state.lock() else {
        return;
    };
    if let Some(latest) = latest.filter(|latest| latest.is_object())
        && let (Some(version), Some(base_version), Some(build), Some(download_url), Some(checksum)) = (
            latest["version"].as_str(),
            latest["baseVersion"].as_str(),
            latest["build"].as_u64(),
            latest["downloadUrl"].as_str(),
            latest["checksum"].as_str(),
        )
    {
        let update_available = latest["updateAvailable"].as_bool().unwrap_or(false);
        let message = latest["message"].as_str().unwrap_or_default().to_owned();
        let release_url = latest["releaseUrl"].as_str().map(str::to_owned);
        let mandatory = latest["mandatory"].as_bool().unwrap_or(false);
        let prerelease = latest["prerelease"].as_bool().unwrap_or(false);
        value.latest_version = Some(version.to_owned());
        value.update_available = update_available;
        value.mandatory = mandatory;
        value.prerelease = prerelease;
        value.release_url = release_url.clone();
        value.message = Some(if update_available {
            message.clone()
        } else {
            "ShroudForge is up to date.".into()
        });
        value.release = Some(Release {
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
        value.state = "ready".into();
        return;
    }
    if last_check_at.is_some() {
        value.latest_version = None;
        value.update_available = false;
        value.mandatory = false;
        value.prerelease = false;
        value.release_url = None;
        value.release = None;
        value.state = "ready".into();
        value.message = Some("The recent check found no compatible release.".into());
    }
}
