use super::create_mod_api::{action_registry_key, valid_identifier};
use mlua::Function;
use std::fs;
pub(crate) fn dispatch_ui_actions(lua: &mlua::Lua, active_mods: &[String]) {
    let Some(game_dir) = lua
        .app_data_ref::<crate::env::AppState>()
        .map(|state| state.env().game_dir().as_std_path().to_path_buf())
    else {
        return;
    };
    let directory = shroudforge_loader_config::paths::ui_data_dir(&game_dir).join("actions");
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten().take(32) {
        let path = entry.path();
        let Some(value) = fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        else {
            let _ = fs::remove_file(path);
            continue;
        };
        let Some(mod_id) = value.get("modId").and_then(|value| value.as_str()) else {
            let _ = fs::remove_file(path);
            continue;
        };
        let Some(action) = value.get("action").and_then(|value| value.as_str()) else {
            let _ = fs::remove_file(path);
            continue;
        };
        let payload = value
            .get("value")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);
        if !active_mods.iter().any(|id| id == mod_id) {
            let _ = fs::remove_file(path);
            continue;
        }
        if !valid_identifier(mod_id) || !valid_identifier(action) {
            let _ = fs::remove_file(path);
            continue;
        }
        match lua.named_registry_value::<Function>(&action_registry_key(mod_id, action)) {
            Ok(callback) => {
                let result = if let Some(payload) = payload {
                    callback.call::<()>(payload)
                } else {
                    callback.call::<()>(())
                };
                if let Err(error) = result {
                    tracing::error!(target: "shroudforge::mod", mod_id, action, "ui action failed: {error}");
                }
                let _ = fs::remove_file(path);
            }
            Err(_) => {
                // Keep the request until the runtime mod has registered its action.
            }
        }
    }
}
