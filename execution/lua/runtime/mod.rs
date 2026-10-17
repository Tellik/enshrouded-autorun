use kfc::reflection::{PrimitiveType, TypeMetadata, TypeRegistry};
use mod_loader::{Capability, Mod};
use shroudforge_compatibility::Availability;
use std::rc::Rc;

use crate::{
    RuntimePhase,
    type_aliases::MappedValue,
    env::{
        AppFeatures, AppState,
        game::value::{
            convert_lua_to_value, convert_value_to_lua, mapped_source_bytes,
            validate_and_clone_lua_value,
        },
        registration::add_function_with_mod,
    },
    lua::{FunctionArgs, LuaError, LuaValue},
};

pub(crate) use crate::runtime_provider;

#[path = "policy.rs"]
mod policy;
use policy::*;
pub(crate) use policy::{available, runtime_denial_reason};
#[path = "ecs/registry.rs"]
mod ecs_registry;
use ecs_registry::*;
#[path = "ecs/entities.rs"]
mod entities;
use entities::*;
#[path = "ecs/queries.rs"]
mod queries;
use queries::*;
#[path = "ecs/components.rs"]
mod components;
use components::*;
#[path = "ecs/props.rs"]
mod props;
use props::*;
#[path = "voxels.rs"]
mod voxels;
use voxels::*;
#[path = "building.rs"]
mod building;
use building::*;
#[path = "engine_context.rs"]
mod engine_context;
use engine_context::*;
#[path = "player_input/cursor.rs"]
mod cursor;
use cursor::*;
#[path = "patches.rs"]
mod patches;
use patches::*;

fn is_component_type(registry: &TypeRegistry, metadata: &TypeMetadata) -> bool {
    if !metadata.qualified_name.starts_with("keen::ecs::")
        || metadata.qualified_name == "keen::ecs::Component"
    {
        return false;
    }
    let mut current = Some(metadata);
    for _ in 0..=registry.len() {
        let Some(ty) = current else {
            return false;
        };
        if ty.qualified_name == "keen::ecs::Component" {
            return true;
        }
        current = registry.get_inner_type(ty);
    }
    false
}

pub(crate) fn runtime_provider_report() -> serde_json::Value {
    runtime_provider::report()
}

pub fn create(lua: &mlua::Lua, r#mod: Mod) -> mlua::Result<mlua::Table> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();
    if app_state.phase() == RuntimePhase::Ingame && !app_state.runtime_configured.get() {
        let registry = app_state.type_registry();
        let ecs_catalog_types = registry
            .iter()
            .filter(|metadata| metadata.qualified_name.starts_with("keen::ecs::"))
            .count();
        let component_candidates = registry
            .iter()
            .filter(|metadata| metadata.size > 0 && is_component_type(registry, metadata))
            .count();
        let dynamic_candidates = registry
            .iter()
            .filter(|metadata| {
                metadata.size > 0
                    && metadata.qualified_name.starts_with("keen::ecs::Dynamic")
                    && !is_component_type(registry, metadata)
            })
            .count();
        let contract: Vec<_> = registry
            .iter()
            .filter(|metadata| {
                metadata.size > 0 && metadata.qualified_name.starts_with("keen::ecs::")
            })
            .map(|metadata| (metadata.qualified_name.clone(), metadata.size))
            .collect();
        tracing::debug!(
            target: "shroudforge::runtime",
            reflection_types = registry.len(),
            ecs_catalog_types,
            component_candidates,
            dynamic_runtime_struct_candidates = dynamic_candidates,
            index_candidates = contract.len(),
            "Prepared KFC runtime component index candidates"
        );
        let configured = runtime_provider::configure(&contract);
        if configured {
            app_state.runtime_configured.set(true);
        }
        if !configured {
            tracing::error!("KFC Runtime rejected the component contract");
        }
        tracing::debug!(
            target: "shroudforge::runtime",
            configured,
            components = contract.len(),
            "KFC runtime component contract configuration finished"
        );
    }
    let table = lua.create_table()?;
    table.raw_set("phase", app_state.phase().as_str())?;
    table.raw_set("schema_version", 1)?;
    table.raw_set("is_client", app_state.is_client())?;
    table.raw_set("is_server", app_state.is_server())?;
    add_function_with_mod(lua, &table, "has", &r#mod, lua_has)?;
    add_function_with_mod(lua, &table, "require", &r#mod, lua_require)?;
    add_function_with_mod(lua, &table, "status", &r#mod, lua_status)?;
    add_function_with_mod(lua, &table, "get_operations", &r#mod, lua_get_operations)?;
    add_function_with_mod(lua, &table, "report_effect", &r#mod, lua_report_effect)?;

    table.raw_set("ecs", create_ecs(lua, &r#mod)?)?;
    table.raw_set(
        "services",
        crate::env::runtime_services::create(lua, &r#mod)?,
    )?;
    table.raw_set(
        "overrides",
        crate::env::runtime_overrides::create(lua, &r#mod)?,
    )?;
    table.raw_set("world", create_world(lua, &r#mod)?)?;
    table.raw_set(
        "network",
        crate::env::runtime_networking::create(lua, &r#mod)?,
    )?;
    table.raw_set("patch", create_patch(lua, &r#mod)?)?;
    table.raw_set("functions", create_functions(lua, &table, &r#mod)?)?;

    Ok(table)
}

fn create_functions(
    lua: &mlua::Lua,
    runtime: &mlua::Table,
    r#mod: &Mod,
) -> mlua::Result<mlua::Table> {
    let result = lua.create_table()?;
    add_function_with_mod(lua, &result, "get_operations", r#mod, lua_get_operations)?;
    crate::env::runtime_functions::attach(lua, &result, &runtime.raw_get::<mlua::Table>("patch")?)?;
    // Capture existing checked wrappers. No second calling convention/dispatcher.
    let bindings = lua.create_table()?;
    let catalog: serde_json::Value =
        serde_json::from_str(include_str!("../../shroudforge/v1/runtime-operations.json"))
            .map_err(LuaError::external)?;
    for id in catalog["runtimeOperations"].as_array().unwrap() {
        let id = id.as_str().unwrap();
        let path = match id {
            "runtime.world.context.active" => "runtime.world.context_active",
            "runtime.world.voxel.grid_spec" => "runtime.world.voxel.get_grid_spec",
            "runtime.network.status" => "runtime.network.status",
            "runtime.network.send" => "runtime.network.send",
            "runtime.network.send_mod" => "runtime.network.send_mod",
            "runtime.network.receive" => "runtime.network.receive",
            "runtime.network.receive_mod" => "runtime.network.receive_mod",
            "runtime.network.accept" => "runtime.network.accept",
            "runtime.network.connected_peers" => "runtime.network.connected_peers",
            "runtime.lifecycle" | "runtime.gameplay.patch" => continue,
            id => id,
        };
        let mut value = LuaValue::Table(runtime.clone());
        for part in path.split('.').skip(1) {
            value = match value {
                LuaValue::Table(table) => table.raw_get::<LuaValue>(part)?,
                _ => LuaValue::Nil,
            };
        }
        if matches!(value, LuaValue::Function(_)) {
            bindings.raw_set(id, value)?;
        }
    }
    let native_get = result.raw_get::<mlua::Function>("get")?;
    let operation_get = native_get.clone();
    let operation_bindings = bindings.clone();
    let operation_mod = r#mod.clone();
    result.raw_set(
        "get",
        lua.create_function(move |lua, selector: LuaValue| {
            let (descriptor, reason): (Option<mlua::Table>, Option<String>) =
                operation_get.call(selector.clone())?;
            if let (Some(descriptor), LuaValue::String(id)) = (&descriptor, &selector) {
                let id = id.to_str()?;
                if let Some(call) =
                    operation_bindings.raw_get::<Option<mlua::Function>>(id.as_ref())?
                {
                    let state = lua.app_data_ref::<AppState>().unwrap();
                    let ready = runtime_denial_reason(&state, &operation_mod, &id).is_none()
                        && available(&state, &operation_mod, &id);
                    descriptor.raw_set("operation_id", id.as_ref())?;
                    descriptor.raw_set("callable", ready)?;
                    descriptor.raw_set("execution", "checked-runtime-operation")?;
                    descriptor.raw_set("call", call)?;
                    descriptor.raw_set(
                        "reason",
                        if ready {
                            None
                        } else {
                            Some("operation context/capability is currently unavailable")
                        },
                    )?;
                }
            }
            Ok((descriptor, reason))
        })?,
    )?;
    let r#mod = r#mod.clone();
    result.raw_set(
        "bind",
        lua.create_function(move |lua, id: String| {
            let state = lua.app_data_ref::<AppState>().unwrap();
            let function = bindings.raw_get::<Option<mlua::Function>>(id.as_str())?;
            if function.is_none() {
                let (descriptor, reason): (Option<mlua::Table>, Option<String>) =
                    native_get.call(id.clone())?;
                let Some(descriptor) = descriptor else {
                    return Ok((None, reason));
                };
                if descriptor.raw_get::<bool>("callable")? {
                    return Ok((Some(descriptor.raw_get::<mlua::Function>("call")?), None));
                }
                return Ok((None, descriptor.raw_get::<Option<String>>("reason")?));
            }
            if let Some(reason) = runtime_denial_reason(&state, &r#mod, &id) {
                return Ok((None, Some(reason)));
            }
            if !available(&state, &r#mod, &id) {
                return Ok((
                    None,
                    Some(format!("operation is currently unavailable: {id}")),
                ));
            }
            Ok((function, None))
        })?,
    )?;
    Ok(result)
}

fn create_ecs(lua: &mlua::Lua, r#mod: &Mod) -> mlua::Result<mlua::Table> {
    let table = lua.create_table()?;

    add_function_with_mod(lua, &table, "get_components", r#mod, lua_ecs_get_components)?;
    add_function_with_mod(lua, &table, "get_catalog", r#mod, lua_ecs_get_catalog)?;
    add_function_with_mod(lua, &table, "get_registry", r#mod, lua_ecs_get_registry)?;
    add_function_with_mod(lua, &table, "get_component", r#mod, lua_ecs_get_component)?;
    add_function_with_mod(lua, &table, "query", r#mod, lua_ecs_query)?;
    add_function_with_mod(lua, &table, "query_bounds", r#mod, lua_ecs_query_bounds)?;
    add_function_with_mod(lua, &table, "resolve", r#mod, lua_ecs_resolve)?;
    add_function_with_mod(lua, &table, "read", r#mod, lua_ecs_read)?;
    add_function_with_mod(lua, &table, "read_bytes", r#mod, lua_ecs_read_bytes)?;
    add_function_with_mod(lua, &table, "read_fields", r#mod, lua_ecs_read_fields)?;
    add_function_with_mod(lua, &table, "get_template", r#mod, lua_ecs_get_template)?;
    crate::env::runtime_templates::attach(lua, &table, r#mod)?;
    add_function_with_mod(lua, &table, "write_fields", r#mod, lua_ecs_write_fields)?;
    add_function_with_mod(lua, &table, "write", r#mod, lua_ecs_write)?;
    crate::env::runtime_attributes::attach(lua, &table, r#mod)?;

    Ok(table)
}

fn create_world(lua: &mlua::Lua, r#mod: &Mod) -> mlua::Result<mlua::Table> {
    let world = lua.create_table()?;
    add_function_with_mod(
        lua,
        &world,
        "operation_available",
        r#mod,
        lua_world_operation_available,
    )?;
    add_function_with_mod(
        lua,
        &world,
        "context_active",
        r#mod,
        lua_world_context_active,
    )?;
    add_function_with_mod(lua, &world, "context_kind", r#mod, lua_world_context_kind)?;
    add_function_with_mod(lua, &world, "session_id", r#mod, lua_world_session_id)?;
    let voxel = lua.create_table()?;
    add_function_with_mod(lua, &voxel, "read", r#mod, lua_world_voxel_read)?;
    add_function_with_mod(lua, &voxel, "write", r#mod, lua_world_voxel_write)?;
    add_function_with_mod(lua, &voxel, "get_grid_spec", r#mod, lua_world_grid_get_spec)?;
    world.raw_set("voxel", voxel)?;
    let cursor = lua.create_table()?;
    add_function_with_mod(lua, &cursor, "get", r#mod, lua_world_cursor_get)?;
    world.raw_set("cursor", cursor)?;
    let entity = lua.create_table()?;
    add_function_with_mod(
        lua,
        &entity,
        "query_props",
        r#mod,
        lua_world_entity_query_props,
    )?;
    add_function_with_mod(
        lua,
        &entity,
        "query_props_in_bounds",
        r#mod,
        lua_world_entity_query_props_in_bounds,
    )?;
    add_function_with_mod(
        lua,
        &entity,
        "register_prop_recipes",
        r#mod,
        lua_world_entity_register_prop_recipes,
    )?;
    add_function_with_mod(
        lua,
        &entity,
        "get_transform",
        r#mod,
        lua_world_entity_get_transform,
    )?;
    add_function_with_mod(lua, &entity, "set_scale", r#mod, lua_world_entity_set_scale)?;
    add_function_with_mod(lua, &entity, "spawn", r#mod, lua_world_entity_spawn)?;
    add_function_with_mod(lua, &entity, "place", r#mod, lua_world_entity_place)?;
    add_function_with_mod(lua, &entity, "destroy", r#mod, lua_world_entity_destroy)?;
    add_function_with_mod(
        lua,
        &entity,
        "finish_building",
        r#mod,
        lua_world_entity_finish_building,
    )?;
    world.raw_set("entity", entity)?;
    crate::env::runtime_building::attach(lua, &world, r#mod)?;
    Ok(world)
}

fn lua_vec<const N: usize>(table: mlua::Table, label: &str) -> mlua::Result<[f64; N]> {
    if table.raw_len() != N {
        return Err(LuaError::generic(format!(
            "{label} must contain exactly {N} numbers"
        )));
    }
    let mut values = [0.0; N];
    for (index, value) in values.iter_mut().enumerate() {
        *value = table.raw_get(index + 1)?;
    }
    Ok(values)
}

pub(crate) fn runtime_type_name(
    lua: &mlua::Lua,
    args: &FunctionArgs,
    index: usize,
) -> mlua::Result<String> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    let type_index =
        crate::env::registry::resolve(state.type_registry(), args.get::<LuaValue>(index)?)
            .map_err(LuaError::generic)?;
    let r#type = state.type_registry().get(type_index).unwrap();

    if !r#type.qualified_name.starts_with("keen::ecs::") {
        return Err(LuaError::generic(format!(
            "runtime ECS operation requires a keen::ecs::* type, got {}",
            r#type.qualified_name
        )));
    }

    Ok(runtime_provider::resolve_type(&r#type.qualified_name)
        .unwrap_or_else(|| r#type.qualified_name.clone()))
}
