use super::{cached_inline_field, handle, outcome};
use crate::lua::LuaValue;
use kfc::reflection::TypeRegistry;
use std::rc::Rc;

pub(super) fn normalize_field(
    lua: &mlua::Lua,
    registry: &Rc<TypeRegistry>,
    selector: LuaValue,
    path: String,
    value: LuaValue,
) -> mlua::Result<(Option<LuaValue>, Option<String>)> {
    let normalize = || -> Result<LuaValue, String> {
        let root = handle(registry, &selector)?;
        let field = cached_inline_field(lua, registry, root.index, &path)?;
        let mut bytes = vec![0; root.size as usize];
        let mut mask = vec![0; bytes.len()];
        field.write_into(lua, &value, &mut bytes, &mut mask)?;
        field.read(lua, &bytes)
    };
    Ok(outcome(normalize()))
}
