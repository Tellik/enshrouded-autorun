use std::path::Path;
use std::time::Duration;
use std::time::Instant;

pub(crate) const ALERT_REOPEN_COOLDOWN: Duration = Duration::from_secs(30);

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
