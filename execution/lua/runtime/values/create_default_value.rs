use super::{decode, handle, outcome};
use crate::lua::LuaValue;
use kfc::reflection::TypeRegistry;
use std::rc::Rc;

pub(super) fn create_default_value(
    lua: &mlua::Lua,
    registry: &Rc<TypeRegistry>,
    selector: LuaValue,
) -> mlua::Result<(Option<LuaValue>, Option<String>)> {
    Ok(outcome(handle(registry, &selector).and_then(|ty| {
        let bytes = ty
            .default_value
            .as_ref()
            .ok_or_else(|| format!("no reflected default for {}", ty.qualified_name))?;
        decode(lua, &ty, bytes)
    })))
}
