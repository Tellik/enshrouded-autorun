use super::*;

pub(super) fn lua_ecs_read(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    lua_ecs_read_impl(lua, args, r#mod, false)
}

pub(super) fn lua_ecs_read_bytes(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    lua_ecs_read_impl(lua, args, r#mod, true)
}

pub(super) fn lua_ecs_read_impl(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
    raw: bool,
) -> mlua::Result<(LuaValue, Option<String>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    if let Some(reason) = runtime_denial_reason(&state, r#mod, "runtime.ecs.read") {
        return Ok((LuaValue::Nil, Some(reason)));
    }
    if !matches!(
        state.api().runtime("runtime.ecs.read"),
        Availability::Available
    ) {
        return unavailable_runtime_result(lua, &state, "runtime.ecs.read");
    }
    let entity_id = args.get::<u32>(0)?;
    let component_name = runtime_type_name(lua, &args, 1)?;
    let Some(component) = runtime_provider::resolve(&component_name) else {
        return Ok((
            LuaValue::Nil,
            Some(format!("live ECS component unavailable: {component_name}")),
        ));
    };
    let Some(metadata) = state
        .type_registry()
        .get_by_name(kfc::reflection::LookupKey::Qualified(&component_name))
    else {
        return Ok((
            LuaValue::Nil,
            Some(format!("type metadata unavailable: {component_name}")),
        ));
    };
    if metadata.size != component.size {
        return Ok((
            LuaValue::Nil,
            Some(format!(
                "component layout mismatch for {component_name}: parser={}, runtime={}",
                metadata.size, component.size
            )),
        ));
    }
    let Some(bytes) = runtime_provider::read(entity_id, &component_name, component.size) else {
        return Ok((
            LuaValue::Nil,
            Some(format!("component read failed: {component_name}")),
        ));
    };
    if raw {
        return Ok((LuaValue::String(lua.create_string(&bytes)?), None));
    }
    let bytes: Rc<[u8]> = bytes.into();
    let mapped = MappedValue::from_bytes(state.type_registry(), metadata, &bytes)
        .map_err(LuaError::external)?;
    Ok((convert_value_to_lua(&mapped, lua)?, None))
}

pub(super) fn lua_ecs_read_fields(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    Ok(match ecs_fields(lua, args, r#mod, false) {
        Ok(value) => (value, None),
        Err(reason) => (LuaValue::Nil, Some(reason)),
    })
}

pub(super) fn lua_ecs_write_fields(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(bool, Option<String>, LuaValue)> {
    Ok(match ecs_fields(lua, args, r#mod, true) {
        Ok(confirmed) => (true, None, confirmed),
        Err(reason) => (false, Some(reason), LuaValue::Nil),
    })
}

pub(super) fn ecs_fields(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
    write: bool,
) -> Result<LuaValue, String> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    let operation = if write {
        "runtime.ecs.write"
    } else {
        "runtime.ecs.read"
    };
    if let Some(reason) = runtime_denial_reason(&state, r#mod, operation) {
        return Err(reason);
    }
    if !available(&state, r#mod, operation) {
        return Err(format!("{operation} is currently unavailable"));
    }
    if args.len() != 3 && !(write && args.len() == 4) {
        return Err(if write {
            "runtime.ecs.write_fields expects entity, component, changes, and optional expected fields"
        } else {
            "runtime.ecs.read_fields expects entity, component, and an array of field paths"
        }
        .into());
    }
    let entity = args.get::<u32>(0).map_err(|e| e.to_string())?;
    let name = runtime_type_name(lua, &args, 1).map_err(|e| e.to_string())?;
    let component = runtime_provider::resolve(&name)
        .ok_or_else(|| format!("live ECS component unavailable: {name}"))?;
    let registry = state.type_registry();
    let metadata = registry
        .get_by_name(kfc::reflection::LookupKey::Qualified(&name))
        .ok_or_else(|| format!("component type metadata unavailable: {name}"))?;
    if metadata.size != component.size {
        return Err(format!(
            "component layout mismatch for {name}: parser={}, runtime={}",
            metadata.size, component.size
        ));
    }
    let selections = args.get::<mlua::Table>(2).map_err(|e| e.to_string())?;
    let mut fields = std::collections::BTreeMap::new();
    for pair in selections.clone().pairs::<LuaValue, LuaValue>() {
        let (key, value) = pair.map_err(|e| e.to_string())?;
        if fields.len() >= 1024 {
            return Err("at most 1024 field paths may be selected".into());
        }
        let path = if write {
            let LuaValue::String(path) = key else {
                return Err("write_fields keys must be reflected field paths".into());
            };
            path.to_str().map_err(|e| e.to_string())?.to_owned()
        } else {
            let LuaValue::Integer(position) = key else {
                return Err("read_fields expects a dense array of field paths".into());
            };
            if position <= 0 || position as usize > selections.raw_len() {
                return Err("read_fields expects a dense array of field paths".into());
            }
            let LuaValue::String(path) = &value else {
                return Err("read_fields expects string field paths".into());
            };
            path.to_str().map_err(|e| e.to_string())?.to_owned()
        };
        let field = state
            .inline_field(metadata.index, &path)
            .map_err(|e| format!("{path}: {e}"))?;
        if fields.insert(path, (field, value)).is_some() {
            return Err("duplicate field path".into());
        }
    }
    if fields.is_empty() || (!write && fields.len() != selections.raw_len()) {
        return Err("select a nonempty set of field paths".into());
    }
    let before = runtime_provider::read(entity, &name, component.size)
        .ok_or_else(|| format!("component read failed: {name}"))?;
    if write {
        if !runtime_provider::can_write_component(&name) {
            return Err("component write context or original command phase is unavailable".into());
        }
        if args.len() == 4 {
            let expected = args.get::<mlua::Table>(3).map_err(|e| e.to_string())?;
            crate::env::runtime_values::expect_inline_fields(
                lua,
                registry,
                metadata.index,
                &before,
                &expected,
            )?;
        }
        let mut after = before.clone();
        let mut mask = vec![0; before.len()];
        for (path, (field, value)) in &fields {
            field
                .write_into(lua, value, &mut after, &mut mask)
                .map_err(|e| format!("{path}: {e}"))?;
        }
        crate::env::runtime_attributes::recalculate_component_update(
            &state, &name, &before, &mut after, &mut mask,
        )?;
        crate::env::runtime_ownership::check_write(
            &state,
            r#mod,
            &crate::env::runtime_ownership::component_target(entity, &name),
        )?;
        let confirmed = lua
            .create_table_with_capacity(0, fields.len())
            .map_err(|error| error.to_string())?;
        for (path, (field, _)) in &fields {
            confirmed
                .raw_set(path.as_str(), field.read(lua, &after)?)
                .map_err(|error| error.to_string())?;
        }
        if after != before {
            runtime_provider::compare_exchange(entity, &name, &before, &mask, &after)?;
        }
        Ok(LuaValue::Table(confirmed))
    } else {
        let result = lua
            .create_table_with_capacity(0, fields.len())
            .map_err(|e| e.to_string())?;
        for (path, (field, _)) in fields {
            result
                .raw_set(
                    path.as_str(),
                    field
                        .read(lua, &before)
                        .map_err(|e| format!("{path}: {e}"))?,
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(LuaValue::Table(result))
    }
}

pub(super) fn lua_ecs_write(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(bool, Option<String>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    if let Some(reason) = runtime_denial_reason(&state, r#mod, "runtime.ecs.write") {
        return Ok((false, Some(reason)));
    }
    match state.api().runtime("runtime.ecs.write") {
        Availability::Unavailable { reason } => return Ok((false, Some(reason))),
        Availability::Available => {}
    }
    if !runtime_provider::ready() {
        return Ok((false, Some("KFC Runtime is not ready".into())));
    }
    let entity_id = args.get::<u32>(0)?;
    let component_name = runtime_type_name(lua, &args, 1)?;
    let lua_value = args.get::<LuaValue>(2)?;
    let Some(component) = runtime_provider::resolve(&component_name) else {
        return Ok((
            false,
            Some(format!("live ECS component unavailable: {component_name}")),
        ));
    };
    let registry = state.type_registry();
    let Some(metadata) =
        registry.get_by_name(kfc::reflection::LookupKey::Qualified(&component_name))
    else {
        return Ok((
            false,
            Some(format!("type metadata unavailable: {component_name}")),
        ));
    };
    if metadata.size != component.size {
        return Ok((
            false,
            Some(format!(
                "component layout mismatch for {component_name}: parser={}, runtime={}",
                metadata.size, component.size
            )),
        ));
    }
    if let Err(reason) = crate::env::runtime_values::validate_owned_type(registry, metadata.index) {
        return Ok((false, Some(reason)));
    }
    let Some(source) = mapped_source_bytes(&lua_value)? else {
        return Ok((
            false,
            Some(format!(
                "runtime.ecs.write requires the value returned by runtime.ecs.read for {component_name}"
            )),
        ));
    };
    if source.len() != component.size as usize {
        return Ok((
            false,
            Some(format!("runtime source size mismatch: {component_name}")),
        ));
    }
    let handle = crate::type_aliases::TypeHandle::new(registry.clone(), metadata.index);
    let checked =
        validate_and_clone_lua_value(&lua_value, &handle, lua).map_err(LuaError::external)?;
    let value = convert_lua_to_value(&checked, &handle).map_err(LuaError::external)?;
    let bytes = value
        .to_bytes(registry, metadata)
        .map_err(LuaError::external)?;
    if bytes.len() != component.size as usize {
        return Ok((
            false,
            Some(format!(
                "serialized component size mismatch: {component_name}"
            )),
        ));
    }
    let mut ranges = Vec::new();
    collect_changed_ranges(registry, metadata, 0, &source, &bytes, &mut ranges)
        .map_err(LuaError::generic)?;
    if ranges.is_empty() {
        return Ok((true, None));
    }
    let mut mask = vec![0u8; component.size as usize];
    for (start, end) in ranges {
        mask[start..end].fill(1);
    }
    let mut bytes = bytes;
    for (index, value) in bytes.iter_mut().enumerate() {
        if mask[index] == 0 {
            *value = source[index];
        }
    }
    if let Err(reason) = crate::env::runtime_attributes::recalculate_component_update(
        &state,
        &component_name,
        &source,
        &mut bytes,
        &mut mask,
    ) {
        return Ok((false, Some(reason)));
    }
    if let Err(reason) = crate::env::runtime_ownership::check_write(
        &state,
        r#mod,
        &crate::env::runtime_ownership::component_target(entity_id, &component_name),
    ) {
        return Ok((false, Some(reason)));
    }
    Ok(
        match runtime_provider::compare_exchange(entity_id, &component_name, &source, &mask, &bytes)
        {
            Ok(()) => (true, None),
            Err(reason) => (false, Some(reason)),
        },
    )
}

pub(super) fn collect_changed_ranges(
    registry: &TypeRegistry,
    metadata: &TypeMetadata,
    base: usize,
    source: &[u8],
    candidate: &[u8],
    ranges: &mut Vec<(usize, usize)>,
) -> Result<(), String> {
    let end = base
        .checked_add(metadata.size as usize)
        .ok_or_else(|| format!("runtime layout overflow: {}", metadata.qualified_name))?;
    if end > source.len() || end > candidate.len() {
        return Err(format!(
            "runtime layout outside component: {}",
            metadata.qualified_name
        ));
    }
    match metadata.primitive_type {
        PrimitiveType::Typedef => {
            let inner = registry
                .get_inner_type(metadata)
                .ok_or_else(|| format!("missing inner type: {}", metadata.qualified_name))?;
            collect_changed_ranges(registry, inner, base, source, candidate, ranges)
        }
        PrimitiveType::Struct => {
            if let Some(parent) = registry.get_inner_type(metadata) {
                collect_changed_ranges(registry, parent, base, source, candidate, ranges)?;
            }
            for field in metadata.struct_fields.values() {
                let field_type = registry.get(field.r#type).ok_or_else(|| {
                    format!(
                        "missing field type: {}.{}",
                        metadata.qualified_name, field.name
                    )
                })?;
                let offset = usize::try_from(field.data_offset).map_err(|_| {
                    format!(
                        "invalid field offset: {}.{}",
                        metadata.qualified_name, field.name
                    )
                })?;
                collect_changed_ranges(
                    registry,
                    field_type,
                    base + offset,
                    source,
                    candidate,
                    ranges,
                )?;
            }
            Ok(())
        }
        PrimitiveType::StaticArray => {
            let inner = registry.get_inner_type(metadata).ok_or_else(|| {
                format!("missing array element type: {}", metadata.qualified_name)
            })?;
            for index in 0..metadata.field_count as usize {
                collect_changed_ranges(
                    registry,
                    inner,
                    base + index * inner.size as usize,
                    source,
                    candidate,
                    ranges,
                )?;
            }
            Ok(())
        }
        PrimitiveType::DsArray
        | PrimitiveType::DsString
        | PrimitiveType::DsOptional
        | PrimitiveType::DsVariant
        | PrimitiveType::BlobArray
        | PrimitiveType::BlobString
        | PrimitiveType::BlobOptional
        | PrimitiveType::BlobVariant
        | PrimitiveType::ObjectReference => {
            if source[base..end] != candidate[base..end] {
                return Err(format!(
                    "runtime write for dynamic field type is not supported: {}",
                    metadata.qualified_name
                ));
            }
            Ok(())
        }
        _ => {
            if source[base..end] != candidate[base..end] {
                crate::env::runtime_values::validate_runtime_scalar(
                    metadata.primitive_type,
                    &candidate[base..end],
                )?;
                if let Some((_, previous_end)) =
                    ranges.last_mut().filter(|(_, value)| *value == base)
                {
                    *previous_end = end;
                } else {
                    ranges.push((base, end));
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod write_policy_tests {
    use super::*;

    #[test]
    fn whole_component_writes_share_finite_and_reference_rules() {
        let mut model = serde_json::to_value(crate::env::registry::tests::fixture()).unwrap();
        model["types"][0]["primitiveType"] = serde_json::json!("FLOAT32");
        let registry: TypeRegistry = serde_json::from_value(model.clone()).unwrap();
        let ty = registry.get(kfc::reflection::TypeIndex::new(0)).unwrap();
        let source = 1f32.to_le_bytes();
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(
                collect_changed_ranges(
                    &registry,
                    ty,
                    0,
                    &source,
                    &value.to_le_bytes(),
                    &mut Vec::new()
                )
                .is_err()
            );
        }
        let mut ranges = Vec::new();
        collect_changed_ranges(&registry, ty, 0, &source, &2f32.to_le_bytes(), &mut ranges)
            .unwrap();
        assert_eq!(ranges, [(0, 4)]);

        model["types"][0]["primitiveType"] = serde_json::json!("OBJECT_REFERENCE");
        let registry: TypeRegistry = serde_json::from_value(model).unwrap();
        let ty = registry.get(kfc::reflection::TypeIndex::new(0)).unwrap();
        assert!(
            collect_changed_ranges(&registry, ty, 0, &[0; 4], &[1, 0, 0, 0], &mut Vec::new())
                .is_err()
        );
        let mut ranges = Vec::new();
        collect_changed_ranges(&registry, ty, 0, &[0; 4], &[0; 4], &mut ranges).unwrap();
        assert!(ranges.is_empty());
    }
}
