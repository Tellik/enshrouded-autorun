//! Semantic adapters for original registered Impact operations, selected by type.
//! Registration does not grant a general native-call or game-write capability.
use super::{
    AppState, FunctionArgs, LuaValue, Mod, available, registry, runtime_denial_reason,
    runtime_provider,
};
use crate::env::registration::add_function_with_mod;
use kfc::reflection::{PrimitiveType, TypeRegistry};

const IS_SAME_ENTITY: &str = "keen::impact_nodes::IsSameEntity";

pub(super) fn create(lua: &mlua::Lua, owner: &Mod) -> mlua::Result<mlua::Table> {
    let result = lua.create_table()?;
    add_function_with_mod(lua, &result, "call", owner, call)?;
    Ok(result)
}

fn argument_ids(
    types: &TypeRegistry,
    selector: &LuaValue,
    value: &LuaValue,
) -> Result<[u32; 2], String> {
    let index = registry::resolve(types, selector)?;
    let node = types
        .get(index)
        .ok_or("Impact operation type is unavailable")?;
    if node.qualified_name != IS_SAME_ENTITY || node.primitive_type != PrimitiveType::Struct {
        return Err(format!(
            "no validated registered Impact adapter for {}",
            node.qualified_name
        ));
    }
    let entity_id = types
        .iter()
        .find(|ty| ty.qualified_name == "keen::ecs::EntityId")
        .ok_or("original EntityId type is unavailable")?;
    let fields = registry::fields(types, index)?;
    let inputs: Vec<_> = fields
        .iter()
        .filter(|(_, field)| field.attributes.contains_key("impact_node_input"))
        .collect();
    if inputs.len() != 2
        || fields
            .iter()
            .any(|(_, field)| field.attributes.contains_key("impact_config"))
    {
        return Err(
            "original Impact operation has an unsupported input/configuration contract".into(),
        );
    }
    for name in ["entityAId", "entityBId"] {
        if inputs
            .iter()
            .filter(|(_, field)| field.name == name && field.r#type == entity_id.index)
            .count()
            != 1
        {
            return Err(format!(
                "original Impact operation lacks a unique EntityId input {name}"
            ));
        }
    }
    let id_fields = registry::fields(types, entity_id.index)?;
    if entity_id.primitive_type != PrimitiveType::Struct
        || entity_id.size != 4
        || id_fields.len() != 1
    {
        return Err("original EntityId has an unsupported inline contract".into());
    }
    let id = &id_fields[0].1;
    if id.name != "id"
        || id.data_offset != 0
        || types
            .get(id.r#type)
            .is_none_or(|ty| ty.primitive_type != PrimitiveType::UInt32 || ty.size != 4)
    {
        return Err("original EntityId.id is not an inline uint32".into());
    }
    let LuaValue::Table(arguments) = value else {
        return Err("Impact call expects an input-name/value table".into());
    };
    require_keys(arguments, &["entityAId", "entityBId"])?;
    let mut ids = [0; 2];
    for (index, name) in ["entityAId", "entityBId"].into_iter().enumerate() {
        let LuaValue::Table(value) = arguments
            .raw_get::<LuaValue>(name)
            .map_err(|e| e.to_string())?
        else {
            return Err(format!("{name} must be an EntityId value {{id=integer}}"));
        };
        require_keys(&value, &["id"])?;
        let LuaValue::Integer(value) =
            value.raw_get::<LuaValue>("id").map_err(|e| e.to_string())?
        else {
            return Err(format!("{name}.id must be an unsigned 32-bit integer"));
        };
        ids[index] = u32::try_from(value)
            .map_err(|_| format!("{name}.id must be an unsigned 32-bit integer"))?;
    }
    Ok(ids)
}

fn require_keys(table: &mlua::Table, names: &[&str]) -> Result<(), String> {
    let mut count = 0;
    for pair in table.clone().pairs::<LuaValue, LuaValue>() {
        let (key, _) = pair.map_err(|e| e.to_string())?;
        let LuaValue::String(key) = key else {
            return Err("Impact inputs require exact named fields".into());
        };
        if !names.contains(&key.to_str().map_err(|e| e.to_string())?.as_ref()) {
            return Err(format!(
                "unknown Impact input field: {}",
                key.to_string_lossy()
            ));
        }
        count += 1;
    }
    if count != names.len() {
        return Err(format!(
            "Impact inputs require exactly {}",
            names.join(", ")
        ));
    }
    Ok(())
}

fn call(
    lua: &mlua::Lua,
    args: FunctionArgs,
    owner: &Mod,
) -> mlua::Result<(Option<mlua::Table>, Option<String>)> {
    let invoke = || -> Result<mlua::Table, String> {
        let (session, ids) = {
            let state = lua.app_data_ref::<AppState>().unwrap();
            // This first adapter compares owned IDs only. It has no engine-world
            // write inputs and uses the same access boundary as named service reads.
            if let Some(reason) = runtime_denial_reason(&state, owner, "runtime.ecs.read") {
                return Err(reason);
            }
            if !available(&state, owner, "runtime.ecs.read") {
                return Err("registered Impact reads are currently unavailable".into());
            }
            if args.len() != 2 {
                return Err("runtime.impact.call expects an original type and named inputs".into());
            }
            let ids = argument_ids(
                state.type_registry(),
                args.get::<LuaValue>(0).map_err(|e| e.to_string())?,
                args.get::<LuaValue>(1).map_err(|e| e.to_string())?,
            )?;
            (runtime_provider::world_session_id(), ids)
        };
        let branch = runtime_provider::impact_compare_entity_ids(session, ids[0], ids[1])?;
        let result = lua.create_table().map_err(|e| e.to_string())?;
        result
            .raw_set("branch", branch)
            .map_err(|e| e.to_string())?;
        Ok(result)
    };
    Ok(match invoke() {
        Ok(result) => (Some(result), None),
        Err(reason) => (None, Some(reason)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn types() -> TypeRegistry {
        let mut types = Vec::new();
        for (index, name, primitive, size) in [
            (0, "uint32", "UINT32", 4),
            (1, "keen::ecs::EntityId", "STRUCT", 4),
            (2, IS_SAME_ENTITY, "STRUCT", 20),
            (3, "keen::impact_nodes::DestroyEntity", "STRUCT", 12),
        ] {
            types.push(
                serde_json::json!({"index":index,"name":name,"qualifiedName":name,"impactName":name,
                "size":size,"alignment":4,"elementAlignment":4,"fieldCount":0,
                "primitiveType":primitive,"flags":"","nameHash":index+1,"qualifiedHash":index+1,
                "impactHash":index+1,"internalHash":index+1}),
            );
        }
        types[1]["structFields"] = serde_json::json!({"id":{"name":"id","type":0,"dataOffset":0}});
        types[2]["structFields"] = serde_json::json!({
            "entityAId":{"name":"entityAId","type":1,"dataOffset":12,"attributes":{"impact_node_input":{"name":"impact_node_input","value":""}}},
            "entityBId":{"name":"entityBId","type":1,"dataOffset":16,"attributes":{"impact_node_input":{"name":"impact_node_input","value":""}}}});
        serde_json::from_value(serde_json::json!({"version":"fixture","types":types})).unwrap()
    }

    #[test]
    fn exact_typed_inputs_preserve_full_raw_id_range() -> mlua::Result<()> {
        let lua = mlua::Lua::new();
        let selector = LuaValue::String(lua.create_string(IS_SAME_ENTITY)?);
        let input = lua
            .load("return {entityAId={id=0},entityBId={id=4294967295}}")
            .eval::<LuaValue>()?;
        assert_eq!(
            argument_ids(&types(), &selector, &input).unwrap(),
            [0, u32::MAX]
        );
        Ok(())
    }

    #[test]
    fn unsupported_types_and_invalid_inputs_never_reach_native_adapter() -> mlua::Result<()> {
        let lua = mlua::Lua::new();
        let selector = LuaValue::String(lua.create_string(IS_SAME_ENTITY)?);
        for input in [
            "{}",
            "{entityAId={id=-1},entityBId={id=1}}",
            "{entityAId={id=1.5},entityBId={id=1}}",
            "{entityAId={id=4294967296},entityBId={id=1}}",
            "{entityAId={id='1'},entityBId={id=1}}",
            "{entityAId={id=1,other=0},entityBId={id=1}}",
            "{entityAId={id=1},entityBId={id=1},extra=0}",
            "{entityAId=1,entityBId=1}",
        ] {
            let value = lua.load(format!("return {input}")).eval::<LuaValue>()?;
            assert!(
                argument_ids(&types(), &selector, &value).is_err(),
                "{input}"
            );
        }
        let other = LuaValue::String(lua.create_string("keen::impact_nodes::DestroyEntity")?);
        let valid = lua
            .load("return {entityAId={id=1},entityBId={id=1}}")
            .eval::<LuaValue>()?;
        assert!(
            argument_ids(&types(), &other, &valid)
                .unwrap_err()
                .contains("no validated")
        );
        let mut changed = serde_json::to_value(types()).unwrap();
        changed["types"][1]["structFields"]["id"]["dataOffset"] = serde_json::json!(1);
        let changed: TypeRegistry = serde_json::from_value(changed).unwrap();
        assert!(argument_ids(&changed, &selector, &valid).is_err());
        Ok(())
    }
}
