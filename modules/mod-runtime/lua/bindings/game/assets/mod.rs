use kfc::guid::{ContentHash, Guid};
use kfc::kfc_format;
use mlua::Table;
use tracing::debug;

use crate::{
    env::{AppState, Buffer, registration::add_function, type_lookup::get_type},
    lua::{FunctionArgs, LuaError, LuaValue},
};

mod access_file_content;
mod access_file_resources;
mod content;
mod resource;
use access_file_content::*;
use access_file_resources::*;
pub use content::*;
pub use resource::*;

pub fn create(lua: &mlua::Lua, r#mod: &mod_loader::Mod) -> mlua::Result<mlua::Table> {
    let table = lua.create_table()?;

    add_function(lua, &table, "get_resource", lua_get_resource)?;
    add_function(lua, &table, "get_resource_parts", lua_get_resource_parts)?;
    add_function(
        lua,
        &table,
        "get_resources_by_type",
        lua_get_resources_by_type,
    )?;
    crate::lua::ingame::resources::attach(lua, &table, r#mod)?;
    add_function(lua, &table, "get_all_resources", lua_get_all_resources)?;
    add_function(lua, &table, "get_resource_types", lua_get_resource_types)?;
    add_function(lua, &table, "create_resource", lua_create_resource)?;
    add_function(lua, &table, "get_content", lua_get_content)?;
    add_function(lua, &table, "get_all_contents", lua_get_all_contents)?;
    add_function(lua, &table, "create_content", lua_create_content)?;

    Ok(table)
}
