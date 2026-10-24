use super::*;

pub(crate) fn lua_world_player_input_get(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    if let Some(reason) = runtime_denial_reason(&state, r#mod, "runtime.world.player_input.get") {
        return Ok((LuaValue::Nil, Some(reason)));
    }
    if args.len() != 0 {
        return Err(LuaError::generic(
            "runtime.world.player_input.get does not accept arguments",
        ));
    }
    let Some(metadata) = state
        .type_registry()
        .get_by_name(kfc::reflection::LookupKey::Qualified(
            "keen::ecs::ClientPlayerInput",
        ))
    else {
        return Ok((
            LuaValue::Nil,
            Some("local ClientPlayerInput reflection is unavailable".into()),
        ));
    };
    let session = runtime_provider::world_session_id();
    if session == 0 {
        return Ok((
            LuaValue::Nil,
            Some("local player input is waiting for a world session".into()),
        ));
    }
    let (bytes, sequence) = match runtime_provider::world_cursor_read_input(metadata.size as usize)
    {
        Ok(sample) => sample,
        Err(reason) => return Ok((LuaValue::Nil, Some(reason))),
    };
    if runtime_provider::world_session_id() != session {
        return Ok((
            LuaValue::Nil,
            Some("local player input world session changed".into()),
        ));
    }
    let read_axis = |path: &str| -> Result<f64, String> {
        let value = state
            .inline_field(metadata.index, path)?
            .read(lua, &bytes)
            .map_err(|error| error.to_string())?;
        match value {
            LuaValue::Number(value) if value.is_finite() => Ok(value),
            _ => Err(format!(
                "local player input field is not a finite number: {path}"
            )),
        }
    };
    let x = match read_axis("data.movementStick.x") {
        Ok(value) => value,
        Err(reason) => return Ok((LuaValue::Nil, Some(reason))),
    };
    let y = match read_axis("data.movementStick.y") {
        Ok(value) => value,
        Err(reason) => return Ok((LuaValue::Nil, Some(reason))),
    };
    let movement = lua.create_table()?;
    movement.raw_set("x", x)?;
    movement.raw_set("y", y)?;
    let snapshot = lua.create_table()?;
    snapshot.raw_set("movementStick", movement)?;
    snapshot.raw_set("sequence", sequence)?;
    snapshot.raw_set("session", session)?;
    Ok((LuaValue::Table(snapshot), None))
}
