#[cfg(windows)]
use crate::update_control_action::HeadlessControlAction;

#[cfg(windows)]
pub(crate) fn claim_headless_control_actions(
    root: &std::path::Path,
) -> Result<Vec<HeadlessControlAction>, String> {
    const SUPPORTED: &[&str] = &[
        "checkUpdates",
        "installMod",
        "updateMod",
        "queueSystemUpdate",
        "selectUpdateQueueItems",
        "removeUpdateQueueItem",
        "clearUpdateQueue",
        "cancelUpdate",
        "startUpdateQueue",
        "stageUpdate",
        "restoreGamefiles",
        "captureGamefileOriginals",
    ];
    let current = shroudforge_loader_config::config::read_loader(root)?;
    if !SUPPORTED.iter().any(|name| {
        current
            .pointer(&format!("/control/actions/{name}"))
            .and_then(serde_json::Value::as_bool)
            == Some(true)
    }) {
        return Ok(Vec::new());
    }
    let mut claimed = Vec::new();
    shroudforge_loader_config::config::update_loader(root, |config| {
        // Queue additions run before startUpdateQueue, so a config can request
        // an install/update and start the resulting selected item in one save.
        for name in SUPPORTED {
            if config
                .pointer(&format!("/control/actions/{name}"))
                .and_then(serde_json::Value::as_bool)
                != Some(true)
            {
                continue;
            }
            let request_id = format!(
                "headless-{name}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            );
            let parameters = config
                .pointer(&format!("/control/parameters/{name}"))
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            config["control"]["actions"][*name] = serde_json::Value::Bool(false);
            config["control"]["results"][*name] = serde_json::json!({
                "requestId": request_id,
                "state": "running",
                "result": "Accepted by headless update controller"
            });
            claimed.push(HeadlessControlAction {
                name: (*name).into(),
                request_id,
                parameters,
            });
        }
        Ok(())
    })?;
    Ok(claimed)
}
