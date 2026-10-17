use crate::env::AppState;
use mlua::Lua;
use mlua::Value as LuaValue;
use serde_json::json;

pub(super) fn read_attribute_catalog(
    lua: &Lua,
    (): (),
) -> mlua::Result<(LuaValue, Option<String>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    let catalog = match state.attribute_catalog() {
        Ok(c) => c,
        Err(reason) => return Ok((LuaValue::Nil, Some(reason))),
    };
    let result = json!({"version":state.type_registry().version,"count":catalog.entries.len(),
            "entries":catalog.entries.values().collect::<Vec<_>>(),"component_layouts":catalog.layouts,
            "storage_resolution":"per-entity-initialized-root-id"});
    Ok((crate::lua::convert_json::json_to_lua(lua, &result)?, None))
}
