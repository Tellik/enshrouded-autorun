use super::*;

pub(super) fn lua_resolve_loaded_resource(
    lua: &mlua::Lua,
    args: FunctionArgs,
    owner: &mod_loader::Mod,
) -> mlua::Result<(Option<Resource>, Option<String>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    let resolve = || -> Result<Option<Resource>, String> {
        if let Some(reason) =
            crate::env::loader::runtime_denial_reason(&state, owner, "runtime.ecs.read")
        {
            return Err(reason);
        }
        if args.len() != 2 {
            return Err(
                "resolve_loaded_resource expects an original registry name and typed key".into(),
            );
        }
        let registry_name = args.get::<String>(0).map_err(|e| e.to_string())?;
        if registry_name != "g38_items::ItemRegistry" {
            return Err(format!(
                "no validated resource relation for {registry_name}"
            ));
        }
        let item_id = args.get::<u32>(1).map_err(|e| e.to_string())?;
        let registry = state.type_registry();
        let ty = registry
            .get_by_name(kfc::reflection::LookupKey::Qualified("keen::ItemInfo"))
            .ok_or("ItemInfo is absent from active reflection")?;
        let key = state.inline_field(ty.index, "itemId.value")?;
        let guid = state.inline_field(ty.index, "objectId")?;
        let key_range = key.byte_range();
        let guid_range = guid.byte_range();
        if key_range.len() != 4
            || guid_range.len() != 16
            || registry
                .get(guid.type_index())
                .is_none_or(|field| field.primitive_type != kfc::reflection::PrimitiveType::Guid)
        {
            return Err("original ItemRegistry fields do not match the active reflection".into());
        }
        let type_id = kfc_format::resource_id(registry, "keen::ItemInfo", Guid::NONE, 0)
            .ok_or("ItemInfo resource identity unavailable")?;
        let session = crate::env::loader::runtime_provider::world_session_id();
        let Some(id) = crate::env::loader::runtime_provider::resource_resolve_item(
            item_id,
            type_id.type_hash(),
            ty.size as usize,
            key_range.start,
            guid_range.start,
            session,
        )?
        else {
            return Ok(None);
        };
        let bytes = crate::env::loader::runtime_provider::resource_read(&id)?;
        if bytes.len() < ty.size as usize
            || bytes.get(key_range.clone()) != Some(item_id.to_le_bytes().as_slice())
            || bytes.get(guid_range) != Some(id.guid().data().as_slice())
            || crate::env::loader::runtime_provider::world_session_id() != session
        {
            return Err("original ItemRegistry candidate changed during snapshot read".into());
        }
        Ok(Some(Resource::new(
            state.get_loaded_resource_info_from_snapshot(&id, session, bytes),
        )))
    };
    Ok(match resolve() {
        Ok(Some(resource)) => (Some(resource), None),
        Ok(None) => (
            None,
            Some("item is not registered in the current original ItemRegistry".into()),
        ),
        Err(reason) => (None, Some(reason)),
    })
}

pub(super) fn lua_get_loaded_resource(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Option<Resource>> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();
    if app_state.phase() != crate::RuntimePhase::Ingame {
        return Err(LuaError::generic(
            "loaded ResourceService access is available only in the ingame phase",
        ));
    }
    let session = crate::env::loader::runtime_provider::world_session_id();
    let guid = args.get::<Guid>(0)?;
    let r#type = get_type(&args, 1, app_state.type_registry().as_ref())?;
    let part = args.get::<Option<u32>>(2)?.unwrap_or(0);
    let requested = kfc_format::resource_id(
        app_state.type_registry(),
        &r#type.qualified_name,
        guid,
        part,
    )
    .ok_or_else(|| LuaError::generic(format!("type not found: {}", r#type.qualified_name)))?;
    // A GUID lookup returns one confirmed snapshot, instead of first probing
    // its page region and fetching the same resource again on first data access.
    let snapshot = match crate::env::loader::runtime_provider::resource_read(&requested) {
        Ok(bytes) => Some(bytes),
        Err(reason) => {
            // Preserve missing-GUID versus unavailable/unstable reader behavior.
            if crate::env::loader::runtime_provider::resource_exists(&requested)
                .map_err(LuaError::generic)?
            {
                return Err(LuaError::generic(reason));
            }
            None
        }
    };
    if crate::env::loader::runtime_provider::world_session_id() != session {
        return Err(LuaError::generic(
            "world session changed during loaded resource lookup",
        ));
    }
    Ok(snapshot.map(|bytes| {
        Resource::new(app_state.get_loaded_resource_info_from_snapshot(&requested, session, bytes))
    }))
}

pub(super) fn lua_get_loaded_resources_by_type(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Table> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();
    if app_state.phase() != crate::RuntimePhase::Ingame {
        return Err(LuaError::generic(
            "loaded ResourceService access is available only in the ingame phase",
        ));
    }
    let session = crate::env::loader::runtime_provider::world_session_id();
    let r#type = get_type(&args, 0, app_state.type_registry().as_ref())?;
    let type_id = kfc_format::resource_id(
        app_state.type_registry(),
        &r#type.qualified_name,
        Guid::NONE,
        0,
    )
    .ok_or_else(|| LuaError::generic(format!("type not found: {}", r#type.qualified_name)))?;
    let resources = crate::env::loader::runtime_provider::resources_by_type(type_id.type_hash())
        .map_err(LuaError::generic)?;
    let result = lua.create_table_with_capacity(resources.len(), 0)?;
    for resource_id in resources {
        result.push(Resource::new(
            app_state.get_loaded_resource_info_for_session(&resource_id, session),
        ))?;
    }
    if crate::env::loader::runtime_provider::world_session_id() != session {
        return Err(LuaError::generic(
            "world session changed during loaded resource listing",
        ));
    }
    Ok(result)
}

pub(super) fn lua_read_loaded_field(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<(LuaValue, Option<String>, LuaValue)> {
    let read = || -> Result<(LuaValue, LuaValue), String> {
        if args.len() != 2 {
            return Err(
                "read_loaded_field expects a loaded Resource and reflected field path".into(),
            );
        }
        let state = lua.app_data_ref::<AppState>().unwrap();
        let resource = args
            .get::<&Resource>(0)
            .map_err(|error| error.to_string())?;
        let path = args.get::<String>(1).map_err(|error| error.to_string())?;
        let bytes = resource.info().loaded_snapshot_bytes()?;
        let metadata =
            kfc_format::type_for_resource(state.type_registry(), &resource.info().resource_id)
                .ok_or("resource type is absent from active reflection")?;
        let field = crate::env::runtime_values::loaded_field(&state, metadata.index, &path, bytes)?;
        let value = field.read(lua, bytes)?;
        let ty = state
            .get_type(lua, field.type_index())
            .map_err(|error| error.to_string())?
            .ok_or("resource field type is unavailable")?;
        Ok((value, ty))
    };
    Ok(match read() {
        Ok((value, ty)) => (value, None, ty),
        Err(reason) => (LuaValue::Nil, Some(reason), LuaValue::Nil),
    })
}
