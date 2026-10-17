#[cfg(windows)]
use crate::headless_check_system_updates;

#[cfg(windows)]
pub(crate) fn check_due_system_update(root: &std::path::Path) -> std::time::Instant {
    let settings = shroudforge_loader_config::config::read_loader(root).ok();
    let enabled = settings
        .as_ref()
        .and_then(|value| value.pointer("/modules/updates/system/enabled"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let interval_minutes = settings
        .as_ref()
        .and_then(|value| value.pointer("/modules/updates/system/checkMinutes"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(60)
        .clamp(5, 1440);
    let last_check_at = settings
        .as_ref()
        .and_then(|value| value.pointer("/modules/updates/system/lastCheckAt"))
        .and_then(serde_json::Value::as_u64);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let interval_seconds = interval_minutes * 60;
    let seconds_until_check = last_check_at
        .map(|checked| interval_seconds.saturating_sub(now.saturating_sub(checked)))
        .unwrap_or(0);
    if enabled && seconds_until_check == 0 {
        match headless_check_system_updates(root, false) {
            Ok(message) => {
                let _ = shroudforge_logging::append(
                    root,
                    if message == "ShroudForge is up to date." {
                        'D'
                    } else {
                        'I'
                    },
                    "updates",
                    &format!("Headless update check: {message}"),
                );
            }
            Err(error) => {
                let _ = shroudforge_logging::append(
                    root,
                    'W',
                    "updates",
                    &format!("Headless update check failed: {error}"),
                );
            }
        }
    }
    std::time::Instant::now()
        + std::time::Duration::from_secs(if enabled {
            if seconds_until_check == 0 {
                interval_seconds
            } else {
                seconds_until_check
            }
        } else {
            60
        })
}
