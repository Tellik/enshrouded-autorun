use kfc::guid::{ContentHash, Guid};
use kfc::kfc_format;
use mlua::Table;
use tracing::debug;

use crate::{
    env::{
        AppState, Buffer,
        registration::{add_function, add_function_with_mod},
        type_lookup::get_type,
    },
    lua::{FunctionArgs, LuaError, LuaValue},
};

#[path = "../bindings/game/assets/content.rs"]
mod content;
#[path = "../runtime/loaded_resources/index.rs"]
pub(crate) mod loaded_index;
#[path = "../bindings/game/assets/resource.rs"]
mod resource;
#[path = "../bindings/game/assets/value.rs"]
pub mod value;

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
    add_function(lua, &table, "get_loaded_resource", lua_get_loaded_resource)?;
    add_function_with_mod(
        lua,
        &table,
        "get_loaded_resource_by_field",
        r#mod,
        loaded_index::get,
    )?;
    add_function_with_mod(
        lua,
        &table,
        "prepare_loaded_index",
        r#mod,
        loaded_index::prepare,
    )?;
    add_function_with_mod(
        lua,
        &table,
        "resolve_loaded_resource",
        r#mod,
        lua_resolve_loaded_resource,
    )?;
    add_function(lua, &table, "read_loaded_field", lua_read_loaded_field)?;
    add_function_with_mod(
        lua,
        &table,
        "write_loaded_field",
        r#mod,
        lua_write_loaded_field,
    )?;
    add_function(
        lua,
        &table,
        "get_loaded_resources_by_type",
        lua_get_loaded_resources_by_type,
    )?;
    add_function(lua, &table, "get_all_resources", lua_get_all_resources)?;
    add_function(lua, &table, "get_resource_types", lua_get_resource_types)?;
    add_function(lua, &table, "create_resource", lua_create_resource)?;
    add_function(lua, &table, "get_content", lua_get_content)?;
    add_function(lua, &table, "get_all_contents", lua_get_all_contents)?;
    add_function(lua, &table, "create_content", lua_create_content)?;

    Ok(table)
}

fn lua_get_resource(lua: &mlua::Lua, args: FunctionArgs) -> mlua::Result<Option<Resource>> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let guid = args.get::<Guid>(0)?;
    let r#type = get_type(&args, 1, app_state.type_registry().as_ref())?;
    let part = args.get::<Option<u32>>(2)?;

    let guid = kfc_format::resource_id(
        app_state.type_registry(),
        &r#type.qualified_name,
        guid,
        part.unwrap_or(0),
    )
    .ok_or_else(|| LuaError::generic(format!("type not found: {}", r#type.qualified_name)))?;

    Ok(app_state.get_resource_info(&guid).map(Resource::new))
}

fn lua_get_resource_parts(lua: &mlua::Lua, args: FunctionArgs) -> mlua::Result<Table> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let guid = args.get::<Guid>(0)?;
    let r#type = get_type(&args, 1, app_state.type_registry().as_ref())?;

    let target_guid =
        kfc_format::resource_id(app_state.type_registry(), &r#type.qualified_name, guid, 0)
            .ok_or_else(|| {
                LuaError::generic(format!("type not found: {}", r#type.qualified_name))
            })?;

    let file = app_state.kfc_file();
    let result = lua.create_table()?;

    for guid in file.resources().keys() {
        if guid.guid() != target_guid.guid() || !kfc_format::same_type(guid, &target_guid) {
            continue;
        }

        let resource = match app_state.get_resource_info(guid) {
            Some(info) => Resource::new(info),
            None => {
                debug!("Resource info not found for GUID: {}", guid);
                continue;
            }
        };

        result.push(resource)?;
    }

    Ok(result)
}

fn lua_get_resources_by_type(lua: &mlua::Lua, args: FunctionArgs) -> mlua::Result<Table> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let r#type = get_type(&args, 0, app_state.type_registry().as_ref())?;

    let file = app_state.kfc_file();
    let result = lua.create_table_with_capacity(file.resources().len(), 0)?;

    for guid in
        kfc_format::resources_by_type(file, app_state.type_registry(), &r#type.qualified_name)
    {
        let resource = match app_state.get_resource_info(&guid) {
            Some(info) => Resource::new(info),
            None => {
                debug!("Resource info not found for GUID: {}", guid);
                continue;
            }
        };

        result.push(resource)?;
    }

    Ok(result)
}

/// Resolve an explicitly supported original engine relation. Unlike by_field,
/// this selects the canonical registry entry, not uniqueness over all payloads.

/// Read one scalar and its exact reflected type from immutable snapshot bytes.
/// No live process read, unlocked post-write observation or Lua path parser.

/// Resolve the selected value through the same reflection/parser graph as reads.
/// A world-bound snapshot and original ECS authority are required. Native success
/// includes independent readback, preserved in the returned Resource.

fn lua_get_all_resources(lua: &mlua::Lua, _args: FunctionArgs) -> mlua::Result<Table> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let file = app_state.kfc_file();
    let result = lua.create_table_with_capacity(file.resources().len(), 0)?;

    for guid in file.resources().keys() {
        let resource = match app_state.get_resource_info(guid) {
            Some(info) => Resource::new(info),
            None => {
                debug!("Resource info not found for GUID: {}", guid);
                continue;
            }
        };

        result.push(resource)?;
    }

    Ok(result)
}

fn lua_get_resource_types(lua: &mlua::Lua, _args: FunctionArgs) -> mlua::Result<Table> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let file = app_state.kfc_file();
    let result = lua.create_table_with_capacity(file.resource_bundles().len(), 0)?;

    for type_index in kfc_format::resource_type_indices(file, app_state.type_registry()) {
        let value = app_state
            .get_type(lua, type_index)?
            .expect("type not found in context");

        result.push(value)?;
    }

    Ok(result)
}

fn lua_create_resource(lua: &mlua::Lua, args: FunctionArgs) -> mlua::Result<Resource> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let value = args.get::<LuaValue>(0)?;
    let r#type = get_type(&args, 1, app_state.type_registry().as_ref())?;

    let guid = if args.len() > 2 {
        let guid = args.get::<Guid>(2)?;
        let part = args.get::<u32>(3)?;

        let guid = kfc_format::resource_id(
            app_state.type_registry(),
            &r#type.qualified_name,
            guid,
            part,
        )
        .ok_or_else(|| LuaError::generic(format!("type not found: {}", r#type.qualified_name)))?;

        app_state.add_resource(value, &guid, lua)?;

        guid
    } else {
        app_state.create_resource(value, r#type.index(), lua)?
    };

    Ok(app_state
        .get_resource_info(&guid)
        .map(Resource::new)
        .expect("just created resource info must exist"))
}

fn lua_get_content(lua: &mlua::Lua, args: FunctionArgs) -> mlua::Result<Option<Content>> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let guid = args.get::<ContentHash>(0)?;

    if !app_state.kfc_file().contents().contains_key(&guid) {
        return Ok(None);
    }

    Ok(Some(Content::new(guid)))
}

fn lua_get_all_contents(lua: &mlua::Lua, _args: FunctionArgs) -> mlua::Result<Table> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let file = app_state.kfc_file();
    let result = lua.create_table_with_capacity(file.contents().len(), 0)?;

    for guid in file.contents().keys() {
        result.push(Content::new(*guid))?;
    }

    Ok(result)
}

fn lua_create_content(lua: &mlua::Lua, args: FunctionArgs) -> mlua::Result<Content> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let buffer = args.get::<&Buffer>(0)?;

    let data = buffer.data()?;

    if data.len() > u32::MAX as usize {
        return Err(LuaError::generic("content size exceeds maximum limit"));
    }

    app_state.create_content(data).map(Content::new)
}

#[path = "../runtime/loaded_resources.rs"]
mod loaded_resources;
use loaded_resources::*;
