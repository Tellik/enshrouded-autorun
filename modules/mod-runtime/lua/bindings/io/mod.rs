use std::io::BufReader;

use mod_loader::Mod;

use crate::{
    env::{
        AppFeatures, AppState, Buffer,
        registration::{add_function, add_function_with_mod},
    },
    logging::{warn, warn_span},
    lua::{Either, FunctionArgs, LuaError},
    type_aliases::{Path, PathBuf},
};

mod manage_exports;
mod path_names;
mod read_mod_files;
use manage_exports::*;
use path_names::*;
use read_mod_files::*;

pub fn create(lua: &mlua::Lua, r#mod: Mod) -> mlua::Result<mlua::Table> {
    let table = lua.create_table()?;

    add_function_with_mod(lua, &table, "read", &r#mod, lua_read)?;
    add_function_with_mod(lua, &table, "read_to_string", &r#mod, lua_read_to_string)?;
    add_function_with_mod(lua, &table, "list_files", &r#mod, lua_list_files)?;
    add_function_with_mod(lua, &table, "exists", &r#mod, lua_exists)?;
    add_function_with_mod(lua, &table, "is_file", &r#mod, lua_is_file)?;
    add_function_with_mod(lua, &table, "is_directory", &r#mod, lua_is_directory)?;
    add_function(lua, &table, "name", lua_name)?;
    add_function(
        lua,
        &table,
        "name_without_extension",
        lua_name_without_extension,
    )?;
    add_function(lua, &table, "extension", lua_extension)?;
    add_function(lua, &table, "parent", lua_parent)?;
    add_function(lua, &table, "join", lua_join)?;
    add_function(lua, &table, "export", lua_export)?;
    add_function(lua, &table, "export_exists", lua_export_exists)?;
    add_function(lua, &table, "export_list", lua_export_list)?;
    add_function(lua, &table, "export_rename", lua_export_rename)?;
    add_function(lua, &table, "export_copy", lua_export_copy)?;
    add_function(lua, &table, "export_delete", lua_export_delete)?;
    add_function(
        lua,
        &table,
        "read_export_to_string",
        lua_read_export_to_string,
    )?;

    Ok(table)
}

fn into_forward_slash(pb: PathBuf) -> String {
    pb.into_string().replace('\\', "/")
}
