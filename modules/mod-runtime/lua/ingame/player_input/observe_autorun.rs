use super::*;

pub(crate) fn lua_world_autorun_set(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(bool, Option<String>)> {
    if args.len() != 1 {
        return Err(LuaError::generic("autorun_set expects one boolean"));
    }
    let enabled = args.get::<bool>(0)?;
    let state = lua.app_data_ref::<AppState>().unwrap();
    if !enabled {
        if !has_capability(r#mod, mod_loader::Capability::Runtime) {
            return Ok((false, Some("runtime capability is required".into())));
        }
        return match runtime_provider::world_autorun_set(false, 0) {
            Ok(()) => Ok((true, None)),
            Err(reason) => Ok((false, Some(reason))),
        };
    }
    if let Some(reason) = runtime_denial_reason(&state, r#mod, "runtime.world.player_input.autorun")
    {
        return Ok((false, Some(reason)));
    }
    let session = if enabled {
        runtime_provider::world_session_id()
    } else {
        0
    };
    match runtime_provider::world_autorun_set(enabled, session) {
        Ok(()) => Ok((true, None)),
        Err(reason) => Ok((false, Some(reason))),
    }
}

pub(crate) fn lua_world_autorun_status(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    if args.len() != 0 {
        return Err(LuaError::generic(
            "autorun_status does not accept arguments",
        ));
    }
    let state = lua.app_data_ref::<AppState>().unwrap();
    if let Some(reason) = runtime_denial_reason(&state, r#mod, "runtime.world.player_input.autorun")
    {
        return Ok((LuaValue::Nil, Some(reason)));
    }
    let Some((hits, local, writes, active, x, y)) = runtime_provider::world_autorun_status() else {
        return Ok((
            LuaValue::Nil,
            Some("native autorun status is unavailable".into()),
        ));
    };
    let result = lua.create_table()?;
    result.raw_set("hits", hits)?;
    result.raw_set("localHits", local)?;
    result.raw_set("writes", writes)?;
    result.raw_set("active", active)?;
    result.raw_set("x", x)?;
    result.raw_set("y", y)?;
    Ok((LuaValue::Table(result), None))
}
