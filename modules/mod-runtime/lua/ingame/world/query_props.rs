use super::*;

pub(super) fn lua_world_entity_query_props(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    if let Some(reason) = runtime_denial_reason(&state, r#mod, "runtime.world.entity.query_props") {
        return Ok((LuaValue::Nil, Some(reason)));
    }
    if args.len() != 2 {
        return Err(LuaError::generic(
            "runtime.world.entity.query_props expects bounds[6] and padding",
        ));
    }
    let bounds_table = args.get::<mlua::Table>(0)?;
    let mut bounds = [0.0_f64; 6];
    for (index, value) in bounds.iter_mut().enumerate() {
        *value = bounds_table.raw_get::<f64>(index + 1)?;
        if !value.is_finite() {
            return Ok((
                LuaValue::Nil,
                Some("bounds must contain six finite numbers".into()),
            ));
        }
    }
    let padding = args.get::<f64>(1)?;
    if !padding.is_finite() || padding < 0.0 {
        return Ok((
            LuaValue::Nil,
            Some("padding must be a finite non-negative number".into()),
        ));
    }
    let props = match runtime_provider::world_entity_query_props(bounds, padding) {
        Ok(props) => props,
        Err(reason) => return Ok((LuaValue::Nil, Some(reason.into()))),
    };
    let result = lua.create_table_with_capacity(props.len(), 0)?;
    for (index, prop) in props.into_iter().enumerate() {
        let item = super::prop_to_lua(lua, &prop)?;
        result.raw_set(index + 1, item)?;
    }
    Ok((LuaValue::Table(result), None))
}

pub(super) fn lua_world_entity_query_props_in_bounds(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    let query_started = std::time::Instant::now();
    let state = lua.app_data_ref::<AppState>().unwrap();
    if let Some(reason) =
        runtime_denial_reason(&state, r#mod, "runtime.world.entity.query_props_in_bounds")
    {
        return Ok((LuaValue::Nil, Some(reason)));
    }
    if args.len() != 1 {
        return Err(LuaError::generic(
            "runtime.world.entity.query_props_in_bounds expects bounds[6]",
        ));
    }
    let bounds = lua_vec::<6>(args.get::<mlua::Table>(0)?.clone(), "bounds")?;
    if bounds.iter().any(|value| !value.is_finite())
        || (0..3).any(|axis| bounds[axis] >= bounds[axis + 3])
    {
        return Ok((
            LuaValue::Nil,
            Some("bounds must contain finite minimum xyz and maximum xyz values".into()),
        ));
    }
    let props = match runtime_provider::world_entity_query_props_in_bounds(bounds) {
        Ok(props) => props,
        Err(reason) => return Ok((LuaValue::Nil, Some(reason.into()))),
    };
    tracing::debug!(
        target: "shroudforge::runtime",
        elapsed_ms = query_started.elapsed().as_millis() as u64,
        matched_props = props.len(),
        "Native recipe-bounds prop query completed"
    );
    let result = lua.create_table_with_capacity(props.len(), 0)?;
    for (index, prop) in props.into_iter().enumerate() {
        let item = super::prop_to_lua(lua, &prop)?;
        result.raw_set(index + 1, item)?;
    }
    Ok((LuaValue::Table(result), None))
}
