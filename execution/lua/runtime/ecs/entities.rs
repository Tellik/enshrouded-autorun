use super::*;

pub(super) fn lua_ecs_resolve(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    if let Some(reason) = runtime_denial_reason(&state, r#mod, "runtime.ecs.resolve") {
        return Ok((LuaValue::Nil, Some(reason)));
    }
    let entity_id = args.get::<u32>(0)?;
    match runtime_provider::resolve_entity(entity_id) {
        Some(handle) => Ok((LuaValue::Integer(i64::from(handle)), None)),
        None => Ok((
            LuaValue::Nil,
            Some(format!("live ECS entity unavailable: {entity_id}")),
        )),
    }
}

pub(super) fn lua_ecs_get_template(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(Option<crate::env::game::Resource>, Option<String>)> {
    let resolve = || -> Result<crate::env::game::Resource, String> {
        let state = lua.app_data_ref::<AppState>().unwrap();
        if let Some(reason) = runtime_denial_reason(&state, r#mod, "runtime.ecs.read") {
            return Err(reason);
        }
        if !available(&state, r#mod, "runtime.ecs.read") {
            return Err("live ECS reads are unavailable".into());
        }
        if args.len() != 1 {
            return Err("get_template expects one current entity handle".into());
        }
        let session = runtime_provider::world_session_id();
        if session == 0 {
            return Err("get_template requires an active world session".into());
        }
        let entity = args.get::<u32>(0).map_err(|error| error.to_string())?;
        let guid = runtime_provider::entity_template_uuid(entity)
            .ok_or("entity template UUID is unavailable or unstable")?;
        let id = kfc::kfc_format::resource_id(
            state.type_registry(),
            "keen::ecs::TemplateResource",
            guid,
            0,
        )
        .ok_or("TemplateResource is absent from the active reflection")?;
        if !runtime_provider::resource_exists(&id)? {
            return Err("entity template is not loaded in the active ResourceService".into());
        }
        // Revalidate the entity-to-template association after resource lookup.
        if runtime_provider::entity_template_uuid(entity) != Some(guid) {
            return Err("entity template changed during resource resolution".into());
        }
        if runtime_provider::world_session_id() != session {
            return Err("world session changed during entity template resolution".into());
        }
        Ok(crate::env::game::Resource::new(
            state.get_loaded_resource_info_for_session(&id, session),
        ))
    };
    Ok(match resolve() {
        Ok(resource) => (Some(resource), None),
        Err(reason) => (None, Some(reason)),
    })
}
