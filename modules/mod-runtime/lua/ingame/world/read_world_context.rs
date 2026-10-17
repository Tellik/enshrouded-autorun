use super::*;

pub(super) fn lua_world_operation_available(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<bool> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    let name = args.get::<String>(0)?;
    if runtime_denial_reason(&state, r#mod, &name).is_some() {
        return Ok(false);
    }
    Ok(runtime_provider::world_operation_available(&name))
}

pub(super) fn lua_world_session_id(
    lua: &mlua::Lua,
    _args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<u64> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    if runtime_denial_reason(&state, r#mod, "runtime.ecs.query").is_some() {
        return Ok(0);
    }
    Ok(runtime_provider::world_session_id())
}

pub(super) fn lua_world_context_active(
    lua: &mlua::Lua,
    _args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(bool, Option<String>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    if let Some(reason) = runtime_denial_reason(&state, r#mod, "runtime.world.context.active") {
        return Ok((false, Some(reason)));
    }
    let active = runtime_provider::world_context_active();
    Ok((
        active,
        (!active).then(|| "active voxel world context is not available".into()),
    ))
}

pub(super) fn lua_world_context_kind(
    lua: &mlua::Lua,
    _args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<String> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    if runtime_denial_reason(&state, r#mod, "runtime.world.context.active").is_some()
        || !runtime_provider::world_operation_available("runtime.world.context.active")
    {
        return Ok("unknown".into());
    }
    Ok(match runtime_provider::world_context_kind() {
        1 => "direct",
        2 => "client-read-only",
        _ => "unknown",
    }
    .into())
}
