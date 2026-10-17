use super::*;

pub(super) fn lua_ecs_query(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    if let Some(reason) = runtime_denial_reason(&state, r#mod, "runtime.ecs.query") {
        return Ok((LuaValue::Nil, Some(reason)));
    }
    if args.len() == 0 {
        return Err(LuaError::generic(
            "runtime.ecs.query requires at least one keen::ecs type",
        ));
    }

    if !runtime_provider::ready() {
        return Ok((LuaValue::Nil, Some("KFC Runtime is not ready".into())));
    }

    let mut components = Vec::with_capacity(args.len());
    for index in 0..args.len() {
        let name = runtime_type_name(lua, &args, index)?;
        if runtime_provider::resolve(&name).is_none() {
            return Ok((
                LuaValue::Nil,
                Some(format!("live ECS component unavailable: {name}")),
            ));
        }
        components.push(name);
    }
    let entities = match runtime_provider::query(&components) {
        Ok(entities) => entities,
        Err(reason) => return Ok((LuaValue::Nil, Some(reason.into()))),
    };
    let result = lua.create_table_with_capacity(entities.len(), 0)?;
    for (index, entity) in entities.into_iter().enumerate() {
        result.raw_set(index + 1, entity)?;
    }
    Ok((LuaValue::Table(result), None))
}

pub(super) fn lua_ecs_query_bounds(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    if let Some(reason) = runtime_denial_reason(&state, r#mod, "runtime.ecs.query") {
        return Ok((LuaValue::Nil, Some(reason)));
    }
    if args.len() < 3 {
        return Err(LuaError::generic(
            "runtime.ecs.query_bounds requires bounds[6], padding, and at least one keen::ecs type",
        ));
    }
    let bounds_table = args.get::<mlua::Table>(0)?;
    let padding = args.get::<f64>(1)?;
    if !padding.is_finite() || padding < 0.0 {
        return Ok((
            LuaValue::Nil,
            Some("padding must be a finite non-negative number".into()),
        ));
    }
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
    if !runtime_provider::ready() {
        return Ok((LuaValue::Nil, Some("KFC Runtime is not ready".into())));
    }
    let mut components = Vec::with_capacity(args.len() - 2);
    for index in 2..args.len() {
        let name = runtime_type_name(lua, &args, index)?;
        if runtime_provider::resolve(&name).is_none() {
            return Ok((
                LuaValue::Nil,
                Some(format!("live ECS component unavailable: {name}")),
            ));
        }
        components.push(name);
    }
    let entities = match runtime_provider::query_bounds(&components, bounds, padding) {
        Ok(entities) => entities,
        Err(reason) => return Ok((LuaValue::Nil, Some(reason.into()))),
    };
    let result = lua.create_table_with_capacity(entities.len(), 0)?;
    for (index, entity) in entities.into_iter().enumerate() {
        result.raw_set(index + 1, entity)?;
    }
    Ok((LuaValue::Table(result), None))
}
