use super::*;
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
