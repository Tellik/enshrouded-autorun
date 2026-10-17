use super::*;
pub(super) fn lua_write_loaded_field(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &mod_loader::Mod,
) -> mlua::Result<(Option<Resource>, Option<String>)> {
    let apply = || -> Result<Resource, String> {
        let state = lua.app_data_ref::<AppState>().unwrap();
        for operation in ["runtime.assets.write", "runtime.ecs.write"] {
            if let Some(reason) =
                crate::env::loader::runtime_denial_reason(&state, r#mod, operation)
            {
                return Err(reason);
            }
            if !crate::env::loader::available(&state, r#mod, operation) {
                return Err(format!("{operation} is currently unavailable"));
            }
        }
        if args.len() != 3 && args.len() != 4 {
            return Err("write_loaded_field expects a loaded Resource, field path, value, and optional expected value".into());
        }
        let resource = args.get::<&Resource>(0).map_err(|e| e.to_string())?;
        let session = resource.info().require_runtime_write_session(
            crate::env::loader::runtime_provider::world_session_id(),
        )?;
        let path = args.get::<String>(1).map_err(|e| e.to_string())?;
        let value = args.get::<LuaValue>(2).map_err(|e| e.to_string())?;
        let id = resource.info().resource_id;
        let registry = state.type_registry();
        let metadata = kfc_format::type_for_resource(registry, &id)
            .ok_or("resource type is absent from the active reflection")?;
        let before = crate::env::loader::runtime_provider::resource_read(&id)?;
        let field =
            crate::env::runtime_values::loaded_field(&state, metadata.index, &path, &before)?;
        let range = field.byte_range();
        if args.len() == 4 {
            let expected = args.get::<LuaValue>(3).map_err(|e| e.to_string())?;
            let mut candidate = before.clone();
            field.write_into(lua, &expected, &mut candidate, &mut vec![0; before.len()])?;
            if candidate[range.clone()] != before[range.clone()] {
                return Err(
                    "Resource scalar snapshot changed before update. Read current values and retry"
                        .into(),
                );
            }
        } else {
            // A prior Resource.data read must not silently turn into a blind
            // assignment. Require the full immutable source snapshot unless
            // the caller supplies an explicit expected scalar.
            let source = resource.info().loaded_snapshot_bytes()?;
            let source_field =
                crate::env::runtime_values::loaded_field(&state, metadata.index, &path, source)?;
            if source.as_ref() != before.as_slice()
                || source_field.byte_range() != range
                || source_field.type_index() != field.type_index()
                || source[range.clone()] != before[range.clone()]
            {
                return Err("Resource source snapshot changed before update. Get a fresh resource or supply the observed expected scalar value".into());
            }
        }
        let mut after = before.clone();
        let mut mask = vec![0; before.len()];
        field.write_into(lua, &value, &mut after, &mut mask)?;
        let offset = range.start;
        if mask.iter().filter(|byte| **byte != 0).count() != 4
            || mask
                .get(offset..offset + 4)
                .is_none_or(|span| span.iter().any(|byte| *byte == 0))
        {
            return Err("live resource writes support one contiguous 32-bit value per call".into());
        }
        resource.info().require_runtime_write_session(
            crate::env::loader::runtime_provider::world_session_id(),
        )?;
        crate::env::runtime_ownership::check_write(
            &state,
            r#mod,
            &crate::env::runtime_ownership::Target::Resource(id),
        )?;
        if let Err(reason) = state.compare_exchange_loaded_resource_scalar(
            &id,
            before.len(),
            offset,
            &before,
            &after[offset..offset + 4],
            session,
        ) {
            if reason.to_lowercase().contains("uncertain") {
                state.record_uncertain_runtime_write(session, &reason);
                resource.info().stop_runtime_writes(&reason);
            }
            return Err(reason);
        }
        // Native success independently read back the complete expected+desired
        // payload under both original gates. Preserve that exact proven result,
        // even if a later observer sees another state or world session.
        Ok(Resource::new(state.get_loaded_resource_info_from_snapshot(
            &id, session, after,
        )))
    };
    Ok(match apply() {
        Ok(resource) => (Some(resource), None),
        Err(reason) => (None, Some(reason)),
    })
}
