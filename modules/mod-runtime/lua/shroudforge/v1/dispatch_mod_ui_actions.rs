use super::create_mod_api::{action_registry_key, valid_identifier};
use mlua::Function;
use std::fs;

enum ActionUnavailable {
    InactiveMod,
    InvalidIdentifier,
    UnregisteredCallback,
}

impl ActionUnavailable {
    fn reason(&self) -> &'static str {
        match self {
            Self::InactiveMod => "mod action requires an active runtime mod",
            Self::InvalidIdentifier => "invalid mod action identifier",
            Self::UnregisteredCallback => "mod action callback is not registered",
        }
    }
}

fn resolve_mod_action(
    lua: &mlua::Lua,
    active_mods: &[String],
    mod_id: &str,
    action: &str,
) -> Result<Function, ActionUnavailable> {
    if !active_mods.iter().any(|id| id == mod_id) {
        return Err(ActionUnavailable::InactiveMod);
    }
    if !valid_identifier(mod_id) || !valid_identifier(action) {
        return Err(ActionUnavailable::InvalidIdentifier);
    }
    lua.named_registry_value::<Function>(&action_registry_key(mod_id, action))
        .map_err(|_| ActionUnavailable::UnregisteredCallback)
}

pub(crate) fn invoke_mod_action(
    lua: &mlua::Lua,
    active_mods: &[String],
    mod_id: &str,
    action: &str,
    payload: Option<&str>,
) -> Result<(), String> {
    let callback = resolve_mod_action(lua, active_mods, mod_id, action)
        .map_err(|error| error.reason().to_owned())?;
    let result = if let Some(payload) = payload {
        callback.call::<()>(payload)
    } else {
        callback.call::<()>(())
    };
    result.map_err(|error| error.to_string())
}

pub(crate) fn dispatch_ui_actions(lua: &mlua::Lua, active_mods: &[String]) {
    let Some(game_dir) = lua
        .app_data_ref::<crate::env::AppState>()
        .map(|state| state.env().game_dir().as_std_path().to_path_buf())
    else {
        return;
    };
    let directory = shroudforge_loader_config::paths::ui_data_dir(&game_dir).join("actions");
    dispatch_queued_actions(lua, active_mods, &directory);
}

fn dispatch_queued_actions(lua: &mlua::Lua, active_mods: &[String], directory: &std::path::Path) {
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
        match resolve_mod_action(lua, active_mods, mod_id, action) {
            Ok(_) => {
                if let Err(error) =
                    invoke_mod_action(lua, active_mods, mod_id, action, payload.as_deref())
                {
                    tracing::error!(target: "shroudforge::mod", mod_id, action, "ui action failed: {error}");
                }
                let _ = fs::remove_file(path);
            }
            Err(ActionUnavailable::UnregisteredCallback) => {
                // Keep the request until the runtime mod has registered its action.
            }
            Err(_) => {
                let _ = fs::remove_file(path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::{Lua, Value};
    use serde_json::json;

    #[test]
    fn mod_action_passes_text_and_absent_payload_to_its_registered_callback() -> mlua::Result<()> {
        let lua = Lua::new();
        let callback = lua
            .load("return function(value) received = value; calls = (calls or 0) + 1 end")
            .eval::<Function>()?;
        lua.set_named_registry_value(&action_registry_key("world-editor", "copy"), callback)?;
        let active = vec!["world-editor".to_owned()];

        invoke_mod_action(&lua, &active, "world-editor", "copy", Some("Block äß"))
            .map_err(mlua::Error::runtime)?;
        assert_eq!(lua.globals().get::<String>("received")?, "Block äß");
        invoke_mod_action(&lua, &active, "world-editor", "copy", None)
            .map_err(mlua::Error::runtime)?;
        assert!(matches!(
            lua.globals().get::<Value>("received")?,
            Value::Nil
        ));
        assert_eq!(lua.globals().get::<u64>("calls")?, 2);
        Ok(())
    }

    #[test]
    fn mod_actions_reject_inactive_invalid_unregistered_and_failed_callbacks() -> mlua::Result<()> {
        let lua = Lua::new();
        let callback = lua
            .load("return function() error('callback sentinel') end")
            .eval::<Function>()?;
        lua.set_named_registry_value(&action_registry_key("world-editor", "failure"), callback)?;
        let active = vec!["world-editor".to_owned(), "invalid/id".to_owned()];

        assert!(
            invoke_mod_action(&lua, &[], "world-editor", "failure", None)
                .unwrap_err()
                .contains("active runtime mod")
        );
        for (mod_id, action) in [("invalid/id", "failure"), ("world-editor", "../failure")] {
            assert!(
                invoke_mod_action(&lua, &active, mod_id, action, None)
                    .unwrap_err()
                    .contains("invalid mod action identifier")
            );
        }
        assert!(
            invoke_mod_action(&lua, &active, "world-editor", "missing", None)
                .unwrap_err()
                .contains("not registered")
        );
        assert!(
            invoke_mod_action(&lua, &active, "world-editor", "failure", None)
                .unwrap_err()
                .contains("callback sentinel")
        );
        Ok(())
    }

    #[test]
    fn ui_queue_retains_only_requests_waiting_for_callback_registration() -> anyhow::Result<()> {
        let lua = Lua::new();
        let directory =
            std::env::temp_dir().join(format!("shroudforge-mod-actions-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&directory)?;
        for (file, mod_id, action) in [
            ("success", "world-editor", "copy"),
            ("failure", "world-editor", "failure"),
            ("waiting", "world-editor", "later"),
            ("inactive", "disabled-editor", "copy"),
            ("invalid", "world-editor", "../copy"),
        ] {
            fs::write(
                directory.join(file),
                json!({"modId":mod_id,"action":action,"value":"from UI"}).to_string(),
            )?;
        }
        fs::write(directory.join("malformed"), "not JSON")?;
        lua.set_named_registry_value(
            &action_registry_key("world-editor", "copy"),
            lua.load("return function(value) received = value end")
                .eval::<Function>()?,
        )?;
        lua.set_named_registry_value(
            &action_registry_key("world-editor", "failure"),
            lua.load("return function() error('UI failure sentinel') end")
                .eval::<Function>()?,
        )?;
        let active = vec!["world-editor".to_owned()];
        dispatch_queued_actions(&lua, &active, &directory);
        assert_eq!(lua.globals().get::<String>("received")?, "from UI");
        let remaining = fs::read_dir(&directory)?
            .map(|entry| entry.map(|entry| entry.file_name()))
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(remaining, vec![std::ffi::OsString::from("waiting")]);

        lua.set_named_registry_value(
            &action_registry_key("world-editor", "later"),
            lua.load("return function(value) late = value end")
                .eval::<Function>()?,
        )?;
        dispatch_queued_actions(&lua, &active, &directory);
        assert_eq!(lua.globals().get::<String>("late")?, "from UI");
        assert_eq!(fs::read_dir(&directory)?.count(), 0);
        fs::remove_dir(directory)?;
        Ok(())
    }
}
