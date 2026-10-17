use super::*;

pub(super) fn create_patch(lua: &mlua::Lua, r#mod: &Mod) -> mlua::Result<mlua::Table> {
    let patch = lua.create_table()?;
    add_function_with_mod(lua, &patch, "available", r#mod, lua_runtime_patch_available)?;
    add_function_with_mod(
        lua,
        &patch,
        "set_enabled",
        r#mod,
        lua_runtime_patch_set_enabled,
    )?;
    Ok(patch)
}

pub(super) fn lua_runtime_patch_available(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<bool> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    let name = args.get::<String>(0)?;
    if runtime_denial_reason(&state, r#mod, "runtime.gameplay.patch").is_some() {
        return Ok(false);
    }
    Ok(runtime_provider::runtime_patch_available(&name))
}

pub(super) fn lua_runtime_patch_set_enabled(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(bool, Option<String>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    if let Some(reason) = runtime_denial_reason(&state, r#mod, "runtime.gameplay.patch") {
        return Ok((false, Some(reason)));
    }
    if args.len() != 2 {
        return Err(LuaError::generic(
            "runtime.patch.set_enabled expects profile patch name and enabled:boolean",
        ));
    }
    let name = args.get::<String>(0)?;
    let enabled = args.get::<bool>(1)?;
    if !runtime_provider::runtime_patch_available(&name) {
        return Ok((
            false,
            Some(format!("build profile does not resolve {name}")),
        ));
    }
    match runtime_provider::runtime_patch_set_enabled(&name, enabled) {
        Ok(()) => Ok((true, None)),
        Err(reason) => Ok((false, Some(reason))),
    }
}
