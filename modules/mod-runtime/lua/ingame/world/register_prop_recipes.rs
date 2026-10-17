use super::*;
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
