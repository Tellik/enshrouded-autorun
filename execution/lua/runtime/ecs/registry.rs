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

pub(super) fn lua_ecs_get_catalog(
    lua: &mlua::Lua,
    _args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<mlua::Table> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    let registry = state.type_registry();
    let result = lua.create_table()?;
    result.raw_set("version", registry.version.clone())?;
    let entries = lua.create_table()?;
    let registration_report = runtime_provider::registry().ok();
    let registration_available = registration_report
        .as_ref()
        .is_some_and(|r| r["available"] == true);
    let mut registrations =
        std::collections::BTreeMap::<&str, Vec<(&serde_json::Value, &str)>>::new();
    if let Some(report) = registration_report
        .as_ref()
        .filter(|_| registration_available)
    {
        for row in report["entries"].as_array().into_iter().flatten() {
            for (key, role) in [("runtime_type", "runtime"), ("template_type", "template")] {
                if let Some(name) = row[key]["qualified_name"].as_str() {
                    registrations.entry(name).or_default().push((row, role));
                }
            }
        }
    }
    let mut registered_count = 0usize;
    let mut runtime_count = 0usize;
    let mut template_count = 0usize;
    let mut resolved_count = 0usize;
    let ready = runtime_provider::ready();
    let can_write = runtime_provider::can_write();
    let read_denial = runtime_denial_reason(&state, r#mod, "runtime.ecs.read");
    let write_denial = runtime_denial_reason(&state, r#mod, "runtime.ecs.write");
    for metadata in registry
        .iter()
        .filter(|ty| ty.qualified_name.starts_with("keen::ecs::"))
    {
        let entry = lua.create_table()?;
        entry.raw_set("type", state.get_type(lua, metadata.index)?)?;
        entry.raw_set("qualified_name", metadata.qualified_name.clone())?;
        entry.raw_set("size", metadata.size)?;
        let roles = registrations.get(metadata.qualified_name.as_str());
        let registered = roles.is_some();
        let runtime_registered = roles.is_some_and(|rows| {
            rows.iter().any(|(row, role)| {
                *role == "runtime"
                    && row["runtime_type"]["size"].as_u64() == Some(metadata.size as u64)
            })
        });
        let template_registered = roles.is_some_and(|rows| {
            rows.iter().any(|(row, role)| {
                *role == "template"
                    && row["template_type"]["size"].as_u64() == Some(metadata.size as u64)
            })
        });
        // Mapping and current entity access are different facts. A verified
        // registration stays resolved while the game is in its main menu.
        let runtime_resolved = runtime_provider::resolve(&metadata.qualified_name)
            .is_some_and(|component| component.size == metadata.size)
            && (!registration_available || runtime_registered);
        let type_writable = runtime_resolved
            && can_write
            && runtime_provider::can_write_component(&metadata.qualified_name);
        let type_write_reason = if !can_write {
            Some("runtime entity write context is not ready".to_owned())
        } else if runtime_resolved && !type_writable {
            Some("component requires an original command phase that is not validated for this profile".to_owned())
        } else {
            None
        };
        let resolved = runtime_resolved || template_registered;
        registered_count += usize::from(registered);
        runtime_count += usize::from(runtime_registered);
        template_count += usize::from(template_registered);
        resolved_count += usize::from(resolved);
        entry.raw_set("registered", registered)?;
        entry.raw_set("runtime_resolved", runtime_resolved)?;
        entry.raw_set(
            "registration_state",
            if registered {
                "registered"
            } else {
                "reflected-only"
            },
        )?;
        entry.raw_set(
            "storage",
            if runtime_registered {
                "entity"
            } else if template_registered {
                "template-only"
            } else {
                "unregistered"
            },
        )?;
        let role_entries = lua.create_table()?;
        if let Some(rows) = roles {
            for (row, role) in rows {
                let association = lua.create_table()?;
                association.raw_set("role", *role)?;
                association.raw_set(
                    "index",
                    crate::env::registry::json_to_lua(lua, &row["index"])?,
                )?;
                association.raw_set(
                    "qualified_name",
                    crate::env::registry::json_to_lua(lua, &row["qualified_name"])?,
                )?;
                role_entries.push(association)?;
            }
        }
        entry.raw_set("registrations", role_entries)?;
        entry.raw_set("resolved", resolved)?;
        let codec_reason =
            crate::env::runtime_values::validate_owned_type(registry, metadata.index).err();
        entry.raw_set("partial_value", codec_reason.is_some())?;
        entry.raw_set("value_reason", codec_reason.clone())?;
        entry.raw_set(
            "read_available",
            runtime_resolved
                && ready
                && read_denial.is_none()
                && matches!(
                    state.api().runtime("runtime.ecs.read"),
                    Availability::Available
                ),
        )?;
        entry.raw_set(
            "write_available",
            runtime_resolved
                && codec_reason.is_none()
                && type_writable
                && write_denial.is_none()
                && matches!(
                    state.api().runtime("runtime.ecs.write"),
                    Availability::Available
                ),
        )?;
        entry.raw_set(
            "read_fields_available",
            runtime_resolved
                && ready
                && read_denial.is_none()
                && matches!(
                    state.api().runtime("runtime.ecs.read"),
                    Availability::Available
                ),
        )?;
        entry.raw_set(
            "write_fields_available",
            runtime_resolved
                && type_writable
                && write_denial.is_none()
                && matches!(
                    state.api().runtime("runtime.ecs.write"),
                    Availability::Available
                ),
        )?;
        entry.raw_set("read_reason", read_denial.clone())?;
        entry.raw_set(
            "write_fields_reason",
            write_denial.clone().or_else(|| type_write_reason.clone()),
        )?;
        entry.raw_set(
            "write_reason",
            write_denial.clone().or(codec_reason).or(type_write_reason),
        )?;
        entry.raw_set(
            "reason",
            if !registered && !runtime_resolved {
                Some("reflected type has no engine component registration")
            } else if !resolved {
                Some("engine registration and reflected storage size do not agree")
            } else {
                None
            },
        )?;
        entries.push(entry)?;
    }
    result.raw_set("registry_available", registration_available)?;
    result.raw_set("registered_type_count", registered_count)?;
    result.raw_set("runtime_type_count", runtime_count)?;
    result.raw_set("template_type_count", template_count)?;
    result.raw_set("resolved_count", resolved_count)?;
    result.raw_set("reflected_only_count", entries.raw_len() - registered_count)?;
    result.raw_set("entity_access_ready", ready)?;
    result.raw_set("entity_writes_ready", can_write)?;
    result.raw_set("entries", entries)?;
    Ok(result)
}

pub(super) fn ecs_registry_entry(
    lua: &mlua::Lua,
    row: &serde_json::Value,
    r#mod: &Mod,
) -> mlua::Result<mlua::Table> {
    let LuaValue::Table(entry) = crate::env::registry::json_to_lua(lua, row)? else {
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
        result.raw_set(key, crate::env::registry::json_to_lua(lua, &report[key])?)?;
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
