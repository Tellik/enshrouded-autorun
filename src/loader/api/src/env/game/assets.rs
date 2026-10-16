use kfc::guid::{ContentHash, Guid};
use mlua::Table;
use shroudforge_parser::kfc_format;
use tracing::debug;

use crate::{
    env::{
        AppState, Buffer,
        util::{add_function, add_function_with_mod, get_type},
    },
    lua::{FunctionArgs, LuaError, LuaValue},
};

mod content;
pub(crate) mod loaded_index;
mod resource;
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
    add_function(lua, &table, "get_loaded_resource_by_field", loaded_index::get)?;
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

fn lua_get_loaded_resource(lua: &mlua::Lua, args: FunctionArgs) -> mlua::Result<Option<Resource>> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();
    if app_state.phase() != crate::RuntimePhase::Ingame {
        return Err(LuaError::generic(
            "loaded ResourceService access is available only in the ingame phase",
        ));
    }
    let session = crate::env::loader::runtime_provider::world_session_id();
    let guid = args.get::<Guid>(0)?;
    let r#type = get_type(&args, 1, app_state.type_registry().as_ref())?;
    let part = args.get::<Option<u32>>(2)?.unwrap_or(0);
    let requested = kfc_format::resource_id(
        app_state.type_registry(),
        &r#type.qualified_name,
        guid,
        part,
    )
    .ok_or_else(|| LuaError::generic(format!("type not found: {}", r#type.qualified_name)))?;
    let found = crate::env::loader::runtime_provider::resource_exists(&requested)
        .map_err(LuaError::generic)?;
    if crate::env::loader::runtime_provider::world_session_id() != session {
        return Err(LuaError::generic("world session changed during loaded resource lookup"));
    }
    Ok(found.then(|| Resource::new(app_state.get_loaded_resource_info_for_session(&requested, session))))
}

fn lua_get_loaded_resources_by_type(lua: &mlua::Lua, args: FunctionArgs) -> mlua::Result<Table> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();
    if app_state.phase() != crate::RuntimePhase::Ingame {
        return Err(LuaError::generic(
            "loaded ResourceService access is available only in the ingame phase",
        ));
    }
    let session = crate::env::loader::runtime_provider::world_session_id();
    let r#type = get_type(&args, 0, app_state.type_registry().as_ref())?;
    let type_id = kfc_format::resource_id(
        app_state.type_registry(),
        &r#type.qualified_name,
        Guid::NONE,
        0,
    )
    .ok_or_else(|| LuaError::generic(format!("type not found: {}", r#type.qualified_name)))?;
    let resources = crate::env::loader::runtime_provider::resources_by_type(type_id.type_hash())
        .map_err(LuaError::generic)?;
    let result = lua.create_table_with_capacity(resources.len(), 0)?;
    for resource_id in resources {
        result.push(Resource::new(
            app_state.get_loaded_resource_info_for_session(&resource_id, session),
        ))?;
    }
    if crate::env::loader::runtime_provider::world_session_id() != session {
        return Err(LuaError::generic("world session changed during loaded resource listing"));
    }
    Ok(result)
}

/// Read one scalar and its exact reflected type from immutable snapshot bytes.
/// No live process read, unlocked post-write observation or Lua path parser.
fn lua_read_loaded_field(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<(LuaValue, Option<String>, LuaValue)> {
    let read = || -> Result<(LuaValue, LuaValue), String> {
        if args.len() != 2 {
            return Err("read_loaded_field expects a loaded Resource and reflected field path".into());
        }
        let state = lua.app_data_ref::<AppState>().unwrap();
        let resource = args.get::<&Resource>(0).map_err(|error| error.to_string())?;
        let path = args.get::<String>(1).map_err(|error| error.to_string())?;
        let bytes = resource.info().loaded_snapshot_bytes()?;
        let metadata = kfc_format::type_for_resource(state.type_registry(), &resource.info().resource_id)
            .ok_or("resource type is absent from active reflection")?;
        let field = crate::env::runtime_values::resource_field(state.type_registry(), metadata.index, &path, bytes)?;
        let value = field.read(lua, bytes)?;
        let ty = state.get_type(lua, field.type_index()).map_err(|error| error.to_string())?
            .ok_or("resource field type is unavailable")?;
        Ok((value, ty))
    };
    Ok(match read() {
        Ok((value, ty)) => (value, None, ty),
        Err(reason) => (LuaValue::Nil, Some(reason), LuaValue::Nil),
    })
}

/// Resolve the selected value through the same reflection/parser graph as reads.
/// A world-bound snapshot and original ECS authority are required. Native success
/// includes independent readback, preserved in the returned Resource.
fn lua_write_loaded_field(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &mod_loader::Mod,
) -> mlua::Result<(Option<Resource>, Option<String>)> {
    let apply = || -> Result<Resource, String> {
        let state = lua.app_data_ref::<AppState>().unwrap();
        for operation in ["runtime.assets.write", "runtime.ecs.write"] {
            if let Some(reason) = crate::env::loader::runtime_denial_reason(&state, r#mod, operation) {
                return Err(reason);
            }
            if !crate::env::loader::available(&state, r#mod, operation) {
                return Err(format!("{operation} is currently unavailable"));
            }
        }
        if args.len() != 3 && args.len() != 4 {
            return Err("write_loaded_field expects a loaded Resource, field path, value, and optional expected value".into());
        }
        let resource = args.get::<&Resource>(0).map_err(|e| e.to_string())?;
        let session = resource.info().require_runtime_write_session(
            crate::env::loader::runtime_provider::world_session_id())?;
        let path = args.get::<String>(1).map_err(|e| e.to_string())?;
        let value = args.get::<LuaValue>(2).map_err(|e| e.to_string())?;
        let id = resource.info().resource_id;
        let registry = state.type_registry();
        let metadata = kfc_format::type_for_resource(registry, &id)
            .ok_or("resource type is absent from the active reflection")?;
        let before = crate::env::loader::runtime_provider::resource_read(&id)?;
        let field =
            crate::env::runtime_values::resource_field(registry, metadata.index, &path, &before)?;
        let range = field.byte_range();
        if args.len() == 4 {
            let expected = args.get::<LuaValue>(3).map_err(|e| e.to_string())?;
            let mut candidate = before.clone();
            field.write_into(lua, &expected, &mut candidate, &mut vec![0; before.len()])?;
            if candidate[range.clone()] != before[range.clone()] {
                return Err(
                    "Resource scalar snapshot changed before update. Read current values and retry"
                        .into(),
                );
            }
        } else {
            // A prior Resource.data read must not silently turn into a blind
            // assignment. Require the full immutable source snapshot unless
            // the caller supplies an explicit expected scalar.
            let source = resource.info().loaded_snapshot_bytes()?;
            let source_field = crate::env::runtime_values::resource_field(
                registry, metadata.index, &path, source)?;
            if source.as_ref() != before.as_slice() || source_field.byte_range() != range
                || source_field.type_index() != field.type_index()
                || source[range.clone()] != before[range.clone()]
            {
                return Err("Resource source snapshot changed before update. Get a fresh resource or supply the observed expected scalar value".into());
            }
        }
        let mut after = before.clone();
        let mut mask = vec![0; before.len()];
        field.write_into(lua, &value, &mut after, &mut mask)?;
        let offset = range.start;
        if mask.iter().filter(|byte| **byte != 0).count() != 4
            || mask
                .get(offset..offset + 4)
                .is_none_or(|span| span.iter().any(|byte| *byte == 0))
        {
            return Err("live resource writes support one contiguous 32-bit value per call".into());
        }
        resource.info().require_runtime_write_session(
            crate::env::loader::runtime_provider::world_session_id())?;
        crate::env::runtime_ownership::check_write(&state, r#mod,
            &crate::env::runtime_ownership::Target::Resource(id))?;
        if let Err(reason) = state.compare_exchange_loaded_resource_scalar(
            &id,
            before.len(),
            offset,
            &before,
            &after[offset..offset + 4],
            session,
        ) {
            if reason.to_lowercase().contains("uncertain") {
                state.record_uncertain_runtime_write(session, &reason);
                resource.info().stop_runtime_writes(&reason);
            }
            return Err(reason);
        }
        // Native success independently read back the complete expected+desired
        // payload under both original gates. Preserve that exact proven result,
        // even if a later observer sees another state or world session.
        Ok(Resource::new(state.get_loaded_resource_info_from_snapshot(&id, session, after)))
    };
    Ok(match apply() {
        Ok(resource) => (Some(resource), None),
        Err(reason) => (None, Some(reason)),
    })
}

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
