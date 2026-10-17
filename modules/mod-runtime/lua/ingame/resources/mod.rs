use crate::env::{
    AppState,
    game::Resource,
    registration::{add_function, add_function_with_mod},
    type_lookup::get_type,
};
use crate::lua::resource_values as value;
use crate::lua::{FunctionArgs, LuaError, LuaValue};
use kfc::{guid::Guid, kfc_format};
use mlua::Table;
mod index_loaded_resources;
mod read_loaded_resources;
mod write_loaded_field;
pub(crate) use index_loaded_resources::tick;
use read_loaded_resources::*;
use write_loaded_field::*;
pub(crate) fn attach(lua: &mlua::Lua, table: &Table, r#mod: &mod_loader::Mod) -> mlua::Result<()> {
    add_function(lua, table, "get_loaded_resource", lua_get_loaded_resource)?;
    add_function_with_mod(
        lua,
        table,
        "get_loaded_resource_by_field",
        r#mod,
        index_loaded_resources::get,
    )?;
    add_function_with_mod(
        lua,
        table,
        "prepare_loaded_index",
        r#mod,
        index_loaded_resources::prepare,
    )?;
    add_function_with_mod(
        lua,
        table,
        "resolve_loaded_resource",
        r#mod,
        lua_resolve_loaded_resource,
    )?;
    add_function(lua, table, "read_loaded_field", lua_read_loaded_field)?;
    add_function_with_mod(
        lua,
        table,
        "write_loaded_field",
        r#mod,
        lua_write_loaded_field,
    )?;
    add_function(
        lua,
        table,
        "get_loaded_resources_by_type",
        lua_get_loaded_resources_by_type,
    )?;
    Ok(())
}
