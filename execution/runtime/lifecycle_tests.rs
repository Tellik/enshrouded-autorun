
use super::{Lifecycle, settings_only_manifest_change};
use serde_json::json;

#[test]
fn failed_start_or_update_retains_cleanup_scope_until_unload_succeeds() -> mlua::Result<()> {
    for failed_callback in ["on_load", "on_update"] {
        let lua = mlua::Lua::new();
        let module: mlua::Table = lua
            .load(
                r#"
                value, cleanup_attempts = 1, 0
                return {
                    on_load=function() value = 2; error("startup failed after write") end,
                    on_update=function() value = 2; error("update failed after write") end,
                    on_unload=function()
                        cleanup_attempts = cleanup_attempts + 1
                        if cleanup_attempts == 1 then error("Scene gate is busy") end
                        value = 1
                    end,
                }
            "#,
            )
            .eval()?;
        let callback: mlua::Function = module.get(failed_callback)?;
        assert!(callback.call::<()>(()).is_err());
        let mut item = Lifecycle {
            id: "independent-mod".into(),
            load: None,
            update: None,
            unload: Some(lua.create_registry_value(module.get::<mlua::Function>("on_unload")?)?),
            update_interval: std::time::Duration::from_millis(100),
            next_update: std::time::Instant::now(),
            active: true,
            cleanup_pending: false,
        };
        assert!(item.try_unload(&lua).is_err());
        assert!(
            item.active && item.cleanup_pending,
            "busy cleanup retains its writable scope"
        );
        assert_eq!(lua.globals().get::<i64>("value")?, 2);
        item.try_unload(&lua)?;
        assert!(!item.active && !item.cleanup_pending);
        assert_eq!(lua.globals().get::<i64>("value")?, 1);
        assert_eq!(lua.globals().get::<i64>("cleanup_attempts")?, 2);
    }
    Ok(())
}

#[test]
fn setting_value_change_does_not_look_like_package_metadata_change() {
    let applied = json!({
        "enabled": true,
        "settingValues": {"maximumCopyableProps": 60000},
        "settings": [{
            "key": "maximumCopyableProps",
            "type": "number",
            "default": 60000,
            "label": "Maximum captured props"
        }]
    });
    let desired = json!({
        "enabled": true,
        "settingValues": {"maximumCopyableProps": 120000},
        "settings": [{
            "key": "maximumCopyableProps",
            "type": "number",
            "default": 60000,
            "label": "Maximum captured props"
        }]
    });

    assert!(settings_only_manifest_change(&desired, &applied));
}

#[test]
fn setting_definition_change_still_requires_restart() {
    let applied = json!({
        "enabled": true,
        "settingValues": {"maximumCopyableProps": 60000},
        "settings": [{
            "key": "maximumCopyableProps",
            "type": "number",
            "default": 60000,
            "label": "Maximum captured props"
        }]
    });
    let desired = json!({
        "enabled": true,
        "settingValues": {"maximumCopyableProps": 120000},
        "settings": [{
            "key": "maximumCopyableProps",
            "type": "number",
            "default": 120000,
            "label": "Maximum captured props (changed)"
        }]
    });

    assert!(!settings_only_manifest_change(&desired, &applied));
}
