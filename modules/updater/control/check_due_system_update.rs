#[cfg(windows)]
use crate::headless_check_system_updates;

pub fn system_update_check_delay(root: &std::path::Path) -> std::time::Duration {
    let settings = shroudforge_loader_config::config::read_loader(root).ok();
    let module_enabled = settings
        .as_ref()
        .and_then(|value| value.pointer("/modules/updates/enabled"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let enabled = module_enabled
        && settings
            .as_ref()
            .and_then(|value| value.pointer("/modules/updates/system/enabled"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
    if !enabled {
        return std::time::Duration::from_secs(86_400);
    }
    let interval_seconds =
        crate::update_check::system_update_check_interval_seconds(settings.as_ref());
    let last_check_at = crate::update_check::last_system_update_check_at(settings.as_ref());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    std::time::Duration::from_secs(
        last_check_at
            .map(|checked| interval_seconds.saturating_sub(now.saturating_sub(checked)))
            .unwrap_or(0),
    )
}

#[cfg(windows)]
pub(crate) fn check_due_system_update(root: &std::path::Path) -> std::time::Instant {
    if !crate::update_check::system_updates_enabled(root) {
        return std::time::Instant::now() + std::time::Duration::from_secs(1);
    }
    let delay = system_update_check_delay(root);
    if delay.is_zero() {
        match headless_check_system_updates(
            root,
            crate::update_check::SystemUpdateCheckMode::Automatic,
        ) {
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
    let next_delay = if delay.is_zero() {
        let next = system_update_check_delay(root);
        if next.is_zero() {
            std::time::Duration::from_secs(60)
        } else {
            next
        }
    } else {
        delay
    };
    std::time::Instant::now() + next_delay
}
