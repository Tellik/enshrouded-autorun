use super::entity_storage;
use super::storage_range;
use crate::env::AppState;
use crate::env::loader;
use mlua::Lua;
use mlua::Value as LuaValue;
use mod_loader::Mod;

pub(super) fn read_attribute(
    lua: &Lua,
    args: mlua::MultiValue,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    let _timing =
        shroudforge_runtime_diagnostics::operation_timings::span("attributes.read_attribute");
    let state = lua.app_data_ref::<AppState>().unwrap();
    let execute = || -> Result<LuaValue, String> {
        let feature = "runtime.ecs.read";
        if let Some(reason) = loader::runtime_denial_reason(&state, &r#mod, feature) {
            return Err(reason);
        }
        if !loader::available(&state, &r#mod, feature) {
            return Err(format!("{feature} unavailable"));
        }
        if args.len() != 2 {
            return Err("expected entity, attribute name/ID, and (for writes) scalar value".into());
        }
        let entity = match &args[0] {
            LuaValue::Integer(v) => u32::try_from(*v).map_err(|_| "entity outside u32")?,
            _ => return Err("expected entity integer".into()),
        };
        let catalog = state.attribute_catalog()?;
        let (descriptor, component, bytes) =
            entity_storage(catalog, catalog.get(&args[1])?, entity)?;
        let _name = component["qualified_name"].as_str().unwrap();
        let range = storage_range(&descriptor, &component, &bytes)?;
        let scalar = component["storage_scalar_type"]
            .as_str()
            .ok_or("unresolved attribute scalar type")?;

        let data: [u8; 4] = bytes[range].try_into().unwrap();
        match scalar {
            "keen::sint32" => Ok(LuaValue::Integer(i32::from_le_bytes(data) as i64)),
            "keen::uint32" => Ok(LuaValue::Integer(u32::from_le_bytes(data) as i64)),
            "keen::float32" => Ok(LuaValue::Number(f32::from_le_bytes(data) as f64)),
            _ => Err("unsupported attribute scalar".into()),
        }
    };
    match execute() {
        Ok(value) => Ok((value, None)),
        Err(e) => Ok((LuaValue::Nil, Some(e))),
    }
}
