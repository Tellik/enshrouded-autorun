use super::encode_attribute_scalar;
use super::root_result;
use super::validate_attribute_bits;
use crate::env::AppState;
use crate::runtime_resolution::attributes::uint;
use mlua::Lua;
use mlua::Table;
use mlua::Value as LuaValue;

pub(super) fn evaluate_attributes(
    lua: &Lua,
    (selector, values): (LuaValue, Table),
) -> mlua::Result<(LuaValue, Option<String>)> {
    let _timing =
        shroudforge_runtime_diagnostics::operation_timings::span("attributes.evaluate_attributes");
    let execute = || -> Result<LuaValue, String> {
        let state = lua.app_data_ref::<AppState>().unwrap();
        let catalog = state.attribute_catalog()?;
        let descriptor = catalog.get(&selector)?;
        let root = uint(&descriptor["root_hash"])?;
        let entries = catalog.root_entries(root)?;
        let scalar = descriptor["calculation_scalar_type"]
            .as_str()
            .ok_or("unknown scalar type")?;
        let mut input = Vec::new();
        for entry in &entries {
            if entry["calculation_encoding_verified"] != true
                || entry["calculation"]["decoded"] != true
            {
                return Err("attribute root has an unverified calculation program".into());
            }
            if entry["calculation_scalar_type"].as_str() != Some(scalar) {
                return Err("attribute root has conflicting scalar interpretations".into());
            }
            let name = entry["name"].as_str().unwrap();
            let value = values
                .raw_get::<LuaValue>(name)
                .map_err(|e| e.to_string())?;
            input.push(u32::from_le_bytes(
                encode_attribute_scalar(entry, scalar, &value)
                    .map_err(|e| format!("{name}: {e}"))?,
            ));
        }
        if values.pairs::<LuaValue, LuaValue>().count() != entries.len() {
            return Err("unexpected attribute names in root values".into());
        }
        let (result, trace) = catalog.recalculate(root, &input)?;
        for (entry, bits) in entries.iter().zip(&result) {
            validate_attribute_bits(entry, scalar, *bits)?;
        }
        crate::lua::convert_json::json_to_lua(lua, &root_result(&entries, scalar, &result, trace)?)
            .map_err(|e| e.to_string())
    };
    Ok(match execute() {
        Ok(v) => (v, None),
        Err(e) => (LuaValue::Nil, Some(e)),
    })
}
