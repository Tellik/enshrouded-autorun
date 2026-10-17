use crate::env::AppState;
use crate::env::registry;
use mlua::Lua;
use mlua::Value as LuaValue;

pub(super) fn read_attribute_descriptor(
    lua: &Lua,
    selector: LuaValue,
) -> mlua::Result<(LuaValue, Option<String>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    let descriptor = state.attribute_catalog().and_then(|c| c.get(&selector));
    match descriptor {
        Ok(d) => Ok((registry::json_to_lua(lua, d)?, None)),
        Err(e) => Ok((LuaValue::Nil, Some(e))),
    }
}
