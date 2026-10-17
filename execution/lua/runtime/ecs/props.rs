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
        let item = lua.create_table()?;
        item.raw_set("handle", prop.entity_handle)?;
        item.raw_set(
            "entityId",
            runtime_provider::world_entity_identity(prop.entity_handle).unwrap_or(prop.entity_id),
        )?;
        item.raw_set("itemId", prop.item_id)?;
        item.raw_set(
            "templateUuidHighHex",
            format!("{:016x}", prop.template_uuid[0]),
        )?;
        item.raw_set(
            "templateUuidLowHex",
            format!("{:016x}", prop.template_uuid[1]),
        )?;
        let position = lua.create_table()?;
        let position_names = ["x", "y", "z"];
        for axis in 0..3 {
            let value = prop.position[axis] as f64 / 4_294_967_296.0;
            position.raw_set(axis + 1, value)?;
            position.raw_set(position_names[axis], value)?;
        }
        let orientation = lua.create_table()?;
        let orientation_names = ["x", "y", "z", "w"];
        for axis in 0..4 {
            orientation.raw_set(axis + 1, prop.orientation[axis])?;
            orientation.raw_set(orientation_names[axis], prop.orientation[axis])?;
        }
        let scale = lua.create_table()?;
        for axis in 0..3 {
            scale.raw_set(axis + 1, prop.scale[axis])?;
            scale.raw_set(position_names[axis], prop.scale[axis])?;
        }
        let transform = lua.create_table()?;
        transform.raw_set("position", position)?;
        transform.raw_set("orientation", orientation)?;
        transform.raw_set("scale", scale)?;
        item.raw_set("transform", transform)?;
        result.raw_set(index + 1, item)?;
    }
    Ok((LuaValue::Table(result), None))
}

pub(super) fn lua_world_entity_register_prop_recipes(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    if let Some(reason) =
        runtime_denial_reason(&state, r#mod, "runtime.world.entity.register_prop_recipes")
    {
        return Ok((LuaValue::Nil, Some(reason)));
    }
    if args.len() != 1 {
        return Err(LuaError::generic(
            "runtime.world.entity.register_prop_recipes expects a recipe array",
        ));
    }
    let input = args.get::<mlua::Table>(0)?;
    let count = input.raw_len();
    if count > 1_000_000 {
        return Ok((
            LuaValue::Nil,
            Some("prop recipe catalog exceeds the 1,000,000-entry safety limit".into()),
        ));
    }
    let mut recipes = Vec::with_capacity(count);
    for index in 1..=count {
        let item = input.raw_get::<mlua::Table>(index)?;
        let item_id = item.raw_get::<u32>("itemId")?;
        let bounds = lua_vec::<6>(item.raw_get::<mlua::Table>("bounds")?, "recipe bounds")?
            .map(|value| value as f32);
        if item_id == 0
            || bounds.iter().any(|value| !value.is_finite())
            || (0..3).any(|axis| bounds[axis] > bounds[axis + 3])
        {
            return Ok((
                LuaValue::Nil,
                Some(format!("invalid placement recipe at index {index}")),
            ));
        }
        let feedback = item.raw_get::<u32>("feedback")?;
        recipes.push(runtime_provider::PropRecipe {
            item_id,
            bounds,
            feedback,
        });
    }
    match runtime_provider::world_entity_register_prop_recipes(&recipes) {
        Ok(()) => Ok((LuaValue::Boolean(true), None)),
        Err(reason) => Ok((LuaValue::Nil, Some(reason))),
    }
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
        let item = lua.create_table()?;
        item.raw_set("handle", prop.entity_handle)?;
        item.raw_set(
            "entityId",
            runtime_provider::world_entity_identity(prop.entity_handle).unwrap_or(prop.entity_id),
        )?;
        item.raw_set("itemId", prop.item_id)?;
        item.raw_set(
            "templateUuidHighHex",
            format!("{:016x}", prop.template_uuid[0]),
        )?;
        item.raw_set(
            "templateUuidLowHex",
            format!("{:016x}", prop.template_uuid[1]),
        )?;
        let position = lua.create_table()?;
        for axis in 0..3 {
            let value = prop.position[axis] as f64 / 4_294_967_296.0;
            position.raw_set(axis + 1, value)?;
            position.raw_set(["x", "y", "z"][axis], value)?;
        }
        let orientation = lua.create_table()?;
        for axis in 0..4 {
            orientation.raw_set(axis + 1, prop.orientation[axis])?;
            orientation.raw_set(["x", "y", "z", "w"][axis], prop.orientation[axis])?;
        }
        let scale = lua.create_table()?;
        for axis in 0..3 {
            scale.raw_set(axis + 1, prop.scale[axis])?;
            scale.raw_set(["x", "y", "z"][axis], prop.scale[axis])?;
        }
        let transform = lua.create_table()?;
        transform.raw_set("position", position)?;
        transform.raw_set("orientation", orientation)?;
        transform.raw_set("scale", scale)?;
        item.raw_set("transform", transform)?;
        result.raw_set(index + 1, item)?;
    }
    Ok((LuaValue::Table(result), None))
}

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
    let item = lua.create_table()?;
    item.raw_set("handle", prop.entity_handle)?;
    item.raw_set(
        "entityId",
        runtime_provider::world_entity_identity(prop.entity_handle).unwrap_or(prop.entity_id),
    )?;
    item.raw_set("itemId", prop.item_id)?;
    item.raw_set(
        "templateUuidHighHex",
        format!("{:016x}", prop.template_uuid[0]),
    )?;
    item.raw_set(
        "templateUuidLowHex",
        format!("{:016x}", prop.template_uuid[1]),
    )?;
    let position = lua.create_table()?;
    let axes = ["x", "y", "z"];
    for axis in 0..3 {
        let value = prop.position[axis] as f64 / 4_294_967_296.0;
        position.raw_set(axis + 1, value)?;
        position.raw_set(axes[axis], value)?;
    }
    let orientation = lua.create_table()?;
    for (axis, name) in ["x", "y", "z", "w"].iter().enumerate() {
        orientation.raw_set(axis + 1, prop.orientation[axis])?;
        orientation.raw_set(*name, prop.orientation[axis])?;
    }
    let scale = lua.create_table()?;
    for axis in 0..3 {
        scale.raw_set(axis + 1, prop.scale[axis])?;
        scale.raw_set(axes[axis], prop.scale[axis])?;
    }
    let transform = lua.create_table()?;
    transform.raw_set("position", position)?;
    transform.raw_set("orientation", orientation)?;
    transform.raw_set("scale", scale)?;
    item.raw_set("transform", transform)?;
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
