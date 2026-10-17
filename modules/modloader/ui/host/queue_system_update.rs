use std::time::SystemTime;
use super::read_host_arguments::Arguments;
use super::read_ui_configuration::read_central_config;
use super::report_command_result::report_command_result;
use shroudforge_updater::release_state::ReleaseState;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;
use std::time::UNIX_EPOCH;

pub(crate) fn queue_system_update(
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    release_state: &Arc<Mutex<ReleaseState>>,
    next_refresh: &mut Instant,
) {
    let persisted = read_central_config(&arguments.root)
        .pointer("/modules/updates/system/latestRelease")
        .filter(|release| release["updateAvailable"].as_bool() == Some(true))
        .cloned();
    let release = persisted.or_else(|| {
        release_state
        .lock()
        .ok()
        .and_then(|state| {
            state.update_available.then(|| state.release.clone()).flatten()
        })
        .map(|release| serde_json::json!({
            "version":release.version.clone(),
            "baseVersion":release.base_version.clone(),
            "build":release.build,
            "downloadUrl":release.download_url.clone(),
            "checksum":release.checksum.clone(),
            "releaseUrl":release.release_url.clone(),
            "message":release.message.clone(),
            "prerelease":release.prerelease,
            "mandatory":release.mandatory,
            "updateAvailable":true,
            "checkedAt":SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
        }))
    });
    let result = release
        .ok_or_else(|| "Check for system updates before adding one to the queue.".to_owned())
        .and_then(|release| {
            let version = release["version"]
                .as_str()
                .ok_or("release has no version")?
                .to_owned();
            shroudforge_updater::enqueue_system_update(&arguments.root, release)?;
            if arguments.updater_window {
                shroudforge_updater::start_update_queue(
                    &arguments.root,
                    &[format!("system:{version}")],
                    false,
                    false,
                )?;
            }
            Ok(())
        });
    report_command_result(
        &arguments.root,
        &webview,
        &request_id,
        "Updates",
        "Add system update to queue",
        result,
        "updates.queue.added",
        None,
        false,
    );
    *next_refresh = Instant::now();
}
