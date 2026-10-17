use super::{handle, outcome};
use crate::{
    env::game::value::convert_value_to_lua,
    lua::LuaValue,
    type_aliases::{MappedValue, TypeHandle},
};
use kfc::reflection::TypeRegistry;
use std::rc::Rc;

pub(super) fn decode(lua: &mlua::Lua, ty: &TypeHandle, bytes: &[u8]) -> Result<LuaValue, String> {
    if bytes.len() < ty.size as usize {
        return Err(format!(
            "truncated {}: expected at least {} bytes",
            ty.qualified_name, ty.size
        ));
    }
    let owned: Rc<[u8]> = bytes.into();
    let value =
        MappedValue::from_bytes(ty.type_registry(), ty, &owned).map_err(|e| e.to_string())?;
    convert_value_to_lua(&value, lua).map_err(|e| e.to_string())
}

pub(super) fn decode_selected_value(
    lua: &mlua::Lua,
    registry: &Rc<TypeRegistry>,
    selector: LuaValue,
    bytes: mlua::String,
) -> mlua::Result<(Option<LuaValue>, Option<String>)> {
    Ok(outcome(
        handle(registry, &selector).and_then(|ty| decode(lua, &ty, &bytes.as_bytes())),
    ))
}
