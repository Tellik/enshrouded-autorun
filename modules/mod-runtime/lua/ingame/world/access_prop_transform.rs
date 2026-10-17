use super::*;
pub(super) fn lua_world_entity_get_transform(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    if let Some(reason) = runtime_denial_reason(&state, r#mod, "runtime.world.entity.get_transform")
    {
        return Ok((LuaValue::Nil, Some(reason)));
    }
    if args.len() != 1 {
        return Err(LuaError::generic(
            "runtime.world.entity.get_transform expects an entity handle",
        ));
    }
    let handle = args.get::<u32>(0)?;
    let Some(prop) = runtime_provider::world_entity_get_transform(handle) else {
        return Ok((
            LuaValue::Nil,
            Some("entity handle is stale or not a live prop".into()),
        ));
    };
    let item = super::prop_to_lua(lua, &prop)?;
    Ok((LuaValue::Table(item), None))
}

pub(super) fn lua_world_entity_set_scale(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(bool, Option<String>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    if let Some(reason) = runtime_denial_reason(&state, r#mod, "runtime.world.entity.set_scale") {
        return Ok((false, Some(reason)));
    }
    if args.len() != 2 {
        return Err(LuaError::generic(
            "runtime.world.entity.set_scale expects an entity handle and scale[3]",
        ));
    }
    let handle = args.get::<u32>(0)?;
    let scale = lua_vec::<3>(args.get::<mlua::Table>(1)?.clone(), "scale")?;
    if let Err(reason) = crate::env::runtime_ownership::check_write(
        &state,
        r#mod,
        &crate::env::runtime_ownership::component_target(handle, "keen::ecs::CurrentTransform"),
    ) {
        return Ok((false, Some(reason)));
    }
    match runtime_provider::world_entity_set_scale(handle, scale) {
        Ok(()) => Ok((true, None)),
        Err(reason) => Ok((false, Some(reason))),
    }
}
