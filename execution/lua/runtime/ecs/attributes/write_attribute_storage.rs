use super::encode_attribute_scalar;
use super::entity_storage;
use super::recalculate_component_update;
use super::storage_range;
use crate::env::AppState;
use crate::env::loader;
use crate::env::runtime_metrics;
use mlua::Lua;
use mlua::Value as LuaValue;
use mod_loader::Mod;

pub(super) fn write_attribute_storage(
    lua: &Lua,
    args: mlua::MultiValue,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    let _timing = runtime_metrics::span("attributes.write_attribute_storage");
    let state = lua.app_data_ref::<AppState>().unwrap();
    let execute = || -> Result<LuaValue, String> {
        let feature = "runtime.ecs.write";
        if let Some(reason) = loader::runtime_denial_reason(&state, &r#mod, feature) {
            return Err(reason);
        }
        if !loader::available(&state, &r#mod, feature) {
            return Err(format!("{feature} unavailable"));
        }
        if args.len() != 3 {
            return Err("expected entity, attribute name/ID, and (for writes) scalar value".into());
        }
        let entity = match &args[0] {
            LuaValue::Integer(v) => u32::try_from(*v).map_err(|_| "entity outside u32")?,
            _ => return Err("expected entity integer".into()),
        };
        let catalog = state.attribute_catalog()?;
        let (descriptor, component, mut bytes) =
            entity_storage(catalog, catalog.get(&args[1])?, entity)?;
        let name = component["qualified_name"].as_str().unwrap();
        let range = storage_range(&descriptor, &component, &bytes)?;
        let scalar = component["storage_scalar_type"]
            .as_str()
            .ok_or("unresolved attribute scalar type")?;

        if descriptor["storage_writable"] != true {
            return Err(descriptor["storage_reason"]
                .as_str()
                .unwrap_or("unverified attribute write layout")
                .into());
        }
        let encoded = encode_attribute_scalar(&descriptor, scalar, &args[2])?;
        let before = bytes.clone();
        let mut mask = vec![0u8; bytes.len()];
        mask[range.clone()].fill(1);
        bytes[range].copy_from_slice(&encoded);
        recalculate_component_update(&state, name, &before, &mut bytes, &mut mask)?;
        // Include the initialized header and root identity in
        // the expected snapshot. A delayed write must not use
        // an association that changed after entity_storage.
        crate::env::runtime_ownership::check_write(
            &state,
            &r#mod,
            &crate::env::runtime_ownership::component_target(entity, name),
        )?;
        loader::runtime_provider::compare_exchange(entity, name, &before, &mask, &bytes)?;
        Ok(LuaValue::Boolean(true))
    };
    match execute() {
        Ok(value) => Ok((value, None)),
        Err(e) => Ok((LuaValue::Boolean(false), Some(e))),
    }
}
