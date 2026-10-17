use std::path::Path;
use std::time::Instant;

pub(crate) fn save_console_preferences(
    root: &Path,
    patch: serde_json::Value,
    next_refresh: &mut Instant,
) {
    let result = shroudforge_loader_config::config::update_loader(root, |value| {
        for key in ["defaultSource", "levelFilter", "autoScroll"] {
            if let Some(setting) = patch.get(key) {
                value["modules"]["debugConsole"][key] = setting.clone();
            }
        }
        Ok(())
    });
    if let Err(error) = result {
        eprintln!("debug preferences: {error}");
        let _ = shroudforge_logging::append(
            root,
            'E',
            "debug-console",
            &format!("Could not save Debug Console preferences: {error}"),
        );
    }
    *next_refresh = Instant::now();
}
