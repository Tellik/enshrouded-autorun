use super::encode_attribute_scalar;
use super::entity_storage;
use super::expect_root_values;
use super::recalculate_bound_component_update;
use super::root_result;
use super::storage_range;
use crate::env::AppState;
use crate::env::loader;
use crate::env::registry;
use crate::env::runtime_metrics;
use crate::runtime_resolution::attributes::uint;
use mlua::Lua;
use mlua::Value as LuaValue;
use mod_loader::Mod;
use serde_json::json;

pub(super) fn update_attribute(
    lua: &Lua,
    args: mlua::MultiValue,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    let _timing = runtime_metrics::span("attributes.update_attribute");
    let execute = || -> Result<LuaValue, String> {
        let state = lua.app_data_ref::<AppState>().unwrap();
        let feature = "runtime.ecs.write";
        if let Some(reason) = loader::runtime_denial_reason(&state, &r#mod, feature) {
            return Err(reason);
        }
        if !loader::available(&state, &r#mod, feature) {
            return Err(format!("{feature} unavailable"));
        }
        if args.len() != 3 && args.len() != 4 {
            return Err("expected entity, attribute name/ID, and (for updates) scalar value plus optional expected_values".into());
        }
        let LuaValue::Integer(entity) = args[0] else {
            return Err("expected entity integer".into());
        };
        let entity = u32::try_from(entity).map_err(|_| "entity outside u32")?;
        let catalog = state.attribute_catalog()?;
        let (descriptor, component, before) =
            entity_storage(catalog, catalog.get(&args[1])?, entity)?;
        let name = component["qualified_name"].as_str().unwrap();
        let scalar = component["storage_scalar_type"].as_str().unwrap();
        let root = uint(&descriptor["root_hash"])?;
        let entries = catalog.root_entries(root)?;
        let start = uint(&component["storage_offset"])? as usize;
        let end = start + uint(&component["storage_size"])? as usize;
        let mut values: Vec<u32> = before[start..end]
            .chunks_exact(4)
            .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
            .collect();
        if entries.len() != values.len() {
            return Err("attribute root storage count changed".into());
        }

        if args.len() == 4 {
            let LuaValue::Table(expected) = &args[3] else {
                return Err(
                    "expected_values must be a table of original attribute names and values".into(),
                );
            };
            expect_root_values(catalog, root, scalar, &values, expected)?;
        }
        let mut after = before.clone();
        let mut mask = vec![0; before.len()];
        let range = storage_range(&descriptor, &component, &before)?;
        after[range.clone()].copy_from_slice(&encode_attribute_scalar(
            &descriptor,
            scalar,
            &args[2],
        )?);
        mask[range].fill(1);
        let functions = loader::runtime_provider::functions()?;
        let trace = recalculate_bound_component_update(
            catalog,
            name,
            &before,
            &mut after,
            &mut mask,
            functions["attribute_calculation_model"]
                .as_str()
                .unwrap_or(""),
        )?;
        values = after[start..end]
            .chunks_exact(4)
            .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
            .collect();
        crate::env::runtime_ownership::check_write(
            &state,
            &r#mod,
            &crate::env::runtime_ownership::component_target(entity, name),
        )?;
        loader::runtime_provider::compare_exchange(entity, name, &before, &mask, &after)?;

        let mut result = root_result(&entries, scalar, &values, trace)?;
        result["entity"] = json!(entity);
        result["component"] = component;
        result["applied"] = json!(true);
        result["effect_scope"] = json!("local process, no automatic client/server RPC");
        registry::json_to_lua(lua, &result).map_err(|e| e.to_string())
    };
    Ok(match execute() {
        Ok(v) => (v, None),
        Err(e) => (LuaValue::Nil, Some(e)),
    })
}
