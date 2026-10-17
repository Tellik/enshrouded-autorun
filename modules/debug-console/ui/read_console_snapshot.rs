use super::read_console_arguments::Arguments;
use super::read_console_settings::Config;
use crate::read_log_tail::read_tail;

pub(crate) fn read_console_snapshot(
    arguments: &Arguments,
    config: &Config,
    preferences: &serde_json::Value,
    minimum_level: String,
    auto_open_alert: bool,
) -> serde_json::Value {
    let payload = serde_json::json!({
        "game": read_tail(&arguments.root.join("enshrouded.log"), config.tail_bytes),
        "loader": read_tail(&shroudforge_loader_config::paths::current_log(&arguments.root), config.tail_bytes),
        "paths": {
            "game": arguments.root.join("enshrouded.log").display().to_string(),
            "loader": shroudforge_loader_config::paths::current_log(&arguments.root).display().to_string(),
        },
        "connected": !arguments.desktop,
        "minimumLevel": minimum_level,
        "autoOpen": auto_open_alert,
        "preferences": preferences,
    });
    payload
}
