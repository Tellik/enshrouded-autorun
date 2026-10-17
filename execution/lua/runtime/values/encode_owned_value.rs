use super::{handle, outcome};
use crate::{
    env::game::value::{convert_lua_to_value, validate_and_clone_lua_value},
    lua::LuaValue,
};
use kfc::reflection::TypeRegistry;
use std::rc::Rc;

pub(super) fn encode_owned_value(
    lua: &mlua::Lua,
    registry: &Rc<TypeRegistry>,
    selector: LuaValue,
    value: LuaValue,
) -> mlua::Result<(Option<LuaValue>, Option<String>)> {
    let result = handle(registry, &selector).and_then(|ty| {
        let checked = validate_and_clone_lua_value(&value, &ty, lua).map_err(|e| e.to_string())?;
        let value = convert_lua_to_value(&checked, &ty).map_err(|e| e.to_string())?;
        let bytes = value.to_bytes(registry, &ty).map_err(|e| e.to_string())?;
        lua.create_string(bytes)
            .map(LuaValue::String)
            .map_err(|e| e.to_string())
    });
    Ok(outcome(result))
}
