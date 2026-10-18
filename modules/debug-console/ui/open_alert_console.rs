use serde_json::Value;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;

pub(crate) fn cooldown(preferences: &Value) -> Duration {
    Duration::from_secs(
        preferences["popupCooldownSeconds"]
            .as_u64()
            .unwrap_or(30)
            .min(3600),
    )
}

pub(crate) fn suppress_reopen_until(preferences: &Value) -> Option<Instant> {
    let cooldown = cooldown(preferences);
    (!cooldown.is_zero()).then(|| Instant::now() + cooldown)
}

pub(crate) fn open_for_alert(
    window: &tao::window::Window,
    root: &Path,
    shown: &mut bool,
    auto_open_alert: &mut bool,
    suppressed_until: Option<Instant>,
) {
    if *shown || suppressed_until.is_some_and(|deadline| Instant::now() < deadline) {
        return;
    }
    *shown = true;
    *auto_open_alert = true;
    window.set_visible(true);
    window.set_focus();
    let _ =
        shroudforge_loader_config::config::publish_window_visibility(root, "debugConsole", true);
}
