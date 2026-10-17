use super::{
    create_default_value::create_default_value, decode_owned_value::decode_selected_value,
    encode_owned_value::encode_owned_value, normalize_field::normalize_field,
};
use crate::lua::LuaValue;
use kfc::reflection::TypeRegistry;
use std::rc::Rc;

pub(crate) fn create(lua: &mlua::Lua, registry: Rc<TypeRegistry>) -> mlua::Result<mlua::Table> {
    let table = lua.create_table()?;
    let registry_field = registry.clone();
    table.set(
        "normalize_field",
        lua.create_function(
            move |lua, (selector, path, value): (LuaValue, String, LuaValue)| {
                normalize_field(lua, &registry_field, selector, path, value)
            },
        )?,
    )?;
    let registry_decode = registry.clone();
    table.set(
        "decode",
        lua.create_function(move |lua, (selector, bytes): (LuaValue, mlua::String)| {
            decode_selected_value(lua, &registry_decode, selector, bytes)
        })?,
    )?;
    let registry_new = registry.clone();
    table.set(
        "new",
        lua.create_function(move |lua, selector: LuaValue| {
            create_default_value(lua, &registry_new, selector)
        })?,
    )?;
    table.set(
        "encode",
        lua.create_function(move |lua, (selector, value): (LuaValue, LuaValue)| {
            encode_owned_value(lua, &registry, selector, value)
        })?,
    )?;
    Ok(table)
}
