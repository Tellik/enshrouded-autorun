use super::entity_storage;
use super::root_result;
use crate::env::AppState;
use crate::env::loader;
use crate::runtime_resolution::attributes::uint;
use mlua::Lua;
use mlua::Value as LuaValue;
use mod_loader::Mod;
use serde_json::json;

pub(super) fn read_attributes(
    lua: &Lua,
    args: mlua::MultiValue,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    let _timing =
        shroudforge_runtime_diagnostics::operation_timings::span("attributes.read_attributes");
    let execute = || -> Result<LuaValue, String> {
        let state = lua.app_data_ref::<AppState>().unwrap();
        let feature = "runtime.ecs.read";
        if let Some(reason) = loader::runtime_denial_reason(&state, &r#mod, feature) {
            return Err(reason);
        }
        if !loader::available(&state, &r#mod, feature) {
            return Err(format!("{feature} unavailable"));
        }
        if args.len() != 2 {
            return Err("expected entity, attribute name/ID, and (for updates) scalar value plus optional expected_values".into());
        }
        let LuaValue::Integer(entity) = args[0] else {
            return Err("expected entity integer".into());
        };
        let entity = u32::try_from(entity).map_err(|_| "entity outside u32")?;
        let catalog = state.attribute_catalog()?;
        let (descriptor, component, before) =
            entity_storage(catalog, catalog.get(&args[1])?, entity)?;
        let _name = component["qualified_name"].as_str().unwrap();
        let scalar = component["storage_scalar_type"].as_str().unwrap();
        let root = uint(&descriptor["root_hash"])?;
        let entries = catalog.root_entries(root)?;
        let start = uint(&component["storage_offset"])? as usize;
        let end = start + uint(&component["storage_size"])? as usize;
        let values: Vec<u32> = before[start..end]
            .chunks_exact(4)
            .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
            .collect();
        if entries.len() != values.len() {
            return Err("attribute root storage count changed".into());
        }
        let trace = Vec::new();

        let mut result = root_result(&entries, scalar, &values, trace)?;
        result["entity"] = json!(entity);
        result["component"] = component;
        result["applied"] = json!(false);
        result["effect_scope"] = json!("local process, no automatic client/server RPC");
        crate::lua::convert_json::json_to_lua(lua, &result).map_err(|e| e.to_string())
    };
    Ok(match execute() {
        Ok(v) => (v, None),
        Err(e) => (LuaValue::Nil, Some(e)),
    })
}
