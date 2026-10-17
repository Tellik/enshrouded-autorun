use super::*;

pub(super) fn lua_ecs_get_components(
    lua: &mlua::Lua,
    _args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    if let Some(reason) = runtime_denial_reason(&state, r#mod, "runtime.ecs.query") {
        return Ok((LuaValue::Nil, Some(reason)));
    }
    if !runtime_provider::ready() {
        return Ok((LuaValue::Nil, Some("KFC Runtime is not ready".into())));
    }
    let result = lua.create_table()?;
    for metadata in state.type_registry().iter() {
        if !metadata.qualified_name.starts_with("keen::ecs::") {
            continue;
        }
        let Some(component) = runtime_provider::resolve(&metadata.qualified_name) else {
            continue;
        };
        if component.size != metadata.size {
            continue;
        }
        if let Some(value) = state.get_type(lua, metadata.index)? {
            result.push(value)?;
        }
    }
    Ok((LuaValue::Table(result), None))
}

pub(super) fn ecs_registry_entry(
    lua: &mlua::Lua,
    row: &serde_json::Value,
    r#mod: &Mod,
) -> mlua::Result<mlua::Table> {
    let LuaValue::Table(entry) = crate::lua::convert_json::json_to_lua(lua, row)? else {
        unreachable!()
    };
    let state = lua.app_data_ref::<AppState>().unwrap();
    let mut codec_reason = None;
    for key in ["runtime_type", "template_type"] {
        if let Some(name) = row[key]["qualified_name"].as_str() {
            let ty = state
                .type_registry()
                .get_by_name(kfc::reflection::LookupKey::Qualified(name))
                .filter(|ty| {
                    ty.qualified_name == name && row[key]["size"].as_u64() == Some(ty.size as u64)
                })
                .ok_or_else(|| {
                    LuaError::generic("engine component registry and parser disagree")
                })?;
            entry.raw_set(key, state.get_type(lua, ty.index)?)?;
            if key == "runtime_type" {
                codec_reason = crate::env::runtime_values::validate_owned_type(
                    state.type_registry(),
                    ty.index,
                )
                .err();
            }
        }
    }
    let has_storage = !row["runtime_type"].is_null();
    let type_writable = row["runtime_type"]["qualified_name"]
        .as_str()
        .is_some_and(runtime_provider::can_write_component);
    let phase_reason = (has_storage && !type_writable)
        .then(|| "component write context or original command phase is unavailable".to_owned());
    entry.raw_set(
        "read_fields_available",
        has_storage
            && available(&state, r#mod, "runtime.ecs.read")
            && runtime_denial_reason(&state, r#mod, "runtime.ecs.read").is_none(),
    )?;
    entry.raw_set(
        "write_fields_available",
        has_storage
            && type_writable
            && available(&state, r#mod, "runtime.ecs.write")
            && runtime_denial_reason(&state, r#mod, "runtime.ecs.write").is_none(),
    )?;
    for (key, operation) in [
        ("read_available", "runtime.ecs.read"),
        ("write_available", "runtime.ecs.write"),
    ] {
        entry.raw_set(
            key,
            has_storage
                && (operation != "runtime.ecs.write" || type_writable)
                && (operation != "runtime.ecs.write" || codec_reason.is_none())
                && available(&state, r#mod, operation)
                && runtime_denial_reason(&state, r#mod, operation).is_none(),
        )?;
    }
    entry.raw_set("partial_value", codec_reason.is_some())?;
    entry.raw_set("value_reason", codec_reason.clone())?;
    entry.raw_set("write_fields_reason", phase_reason.clone())?;
    entry.raw_set("write_reason", codec_reason.or(phase_reason))?;
    if !has_storage {
        entry.raw_set(
            "reason",
            "template-only registration has no entity storage column",
        )?;
    }
    Ok(entry)
}

pub(super) fn lua_ecs_get_registry(
    lua: &mlua::Lua,
    _args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    let report = match runtime_provider::registry() {
        Ok(report) => report,
        Err(reason) => return Ok((LuaValue::Nil, Some(reason))),
    };
    if report["available"] != true {
        return Ok((
            LuaValue::Nil,
            Some(
                report["reason"]
                    .as_str()
                    .unwrap_or("component registry unavailable")
                    .into(),
            ),
        ));
    }
    let result = lua.create_table()?;
    for key in [
        "schema_version",
        "layout_version",
        "source",
        "count",
        "runtime_type_count",
    ] {
        result.raw_set(
            key,
            crate::lua::convert_json::json_to_lua(lua, &report[key])?,
        )?;
    }
    let entries = lua.create_table()?;
    for row in report["entries"]
        .as_array()
        .ok_or_else(|| LuaError::generic("invalid provider registry"))?
    {
        entries.push(ecs_registry_entry(lua, row, r#mod)?)?;
    }
    result.raw_set("entries", entries)?;
    Ok((LuaValue::Table(result), None))
}

pub(super) fn lua_ecs_get_component(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    let index = match crate::env::registry::resolve(state.type_registry(), args.get::<LuaValue>(0)?)
    {
        Ok(index) => index,
        Err(reason) => return Ok((LuaValue::Nil, Some(reason))),
    };
    let name = &state.type_registry().get(index).unwrap().qualified_name;
    let report = match runtime_provider::registry() {
        Ok(report) => report,
        Err(reason) => return Ok((LuaValue::Nil, Some(reason))),
    };
    for row in report["entries"].as_array().into_iter().flatten() {
        if row["qualified_name"].as_str() == Some(name)
            || row["runtime_type"]["qualified_name"].as_str() == Some(name)
        {
            return Ok((LuaValue::Table(ecs_registry_entry(lua, row, r#mod)?), None));
        }
    }
    Ok((
        LuaValue::Nil,
        Some(format!("no engine component registration for {name}")),
    ))
}
