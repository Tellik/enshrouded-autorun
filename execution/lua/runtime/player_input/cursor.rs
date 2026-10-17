use super::*;

pub(super) fn lua_world_cursor_get(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<(LuaValue, Option<String>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    if let Some(reason) = runtime_denial_reason(&state, r#mod, "runtime.world.cursor.get") {
        return Ok((LuaValue::Nil, Some(reason)));
    }
    if args.len() != 0 {
        return Err(LuaError::generic(
            "runtime.world.cursor.get does not accept arguments",
        ));
    }
    let Some((bytes, sequence)) = runtime_provider::world_cursor_read() else {
        return Ok((
            LuaValue::Nil,
            Some("native cursor hook has not published a live sample yet".into()),
        ));
    };
    if bytes.len() != 0xa0 || bytes[0x98] > 1 {
        return Ok((
            LuaValue::Nil,
            Some("native cursor snapshot failed layout validation".into()),
        ));
    }
    let transform = |base: usize| -> mlua::Result<mlua::Table> {
        let output = lua.create_table()?;
        let position = lua.create_table()?;
        for (axis, name) in ["x", "y", "z"].iter().enumerate() {
            let start = base + axis * 8;
            position.raw_set(
                *name,
                i64::from_le_bytes(bytes[start..start + 8].try_into().unwrap()),
            )?;
        }
        let rotation = lua.create_table()?;
        for index in 0..4 {
            let start = base + 0x18 + index * 4;
            rotation.raw_set(
                index + 1,
                f32::from_le_bytes(bytes[start..start + 4].try_into().unwrap()),
            )?;
        }
        let scale = lua.create_table()?;
        for index in 0..3 {
            let start = base + 0x28 + index * 4;
            scale.raw_set(
                index + 1,
                f32::from_le_bytes(bytes[start..start + 4].try_into().unwrap()),
            )?;
        }
        output.raw_set("position", position)?;
        output.raw_set("rotation", rotation)?;
        output.raw_set("scale", scale)?;
        Ok(output)
    };
    let value = lua.create_table()?;
    value.raw_set("primaryTransform", transform(0)?)?;
    value.raw_set("secondaryTransform", transform(0x38)?)?;
    value.raw_set("primaryFlags", bytes[0x70])?;
    value.raw_set("secondaryFlags", bytes[0x71])?;
    value.raw_set("material", bytes[0x72])?;
    value.raw_set(
        "selectionVersion",
        format!(
            "{}",
            u64::from_le_bytes(bytes[0x78..0x80].try_into().unwrap())
        ),
    )?;
    let selected = lua.create_table()?;
    selected.raw_set(
        "value",
        format!(
            "{:016x}",
            u64::from_le_bytes(bytes[0x80..0x88].try_into().unwrap())
        ),
    )?;
    selected.raw_set(
        "type",
        u32::from_le_bytes(bytes[0x88..0x8c].try_into().unwrap()),
    )?;
    value.raw_set("selectedObject", selected)?;
    value.raw_set("attachedToProp", bytes[0x98] != 0)?;
    let result = lua.create_table()?;
    result.raw_set("sequence", sequence)?;
    result.raw_set("value", value)?;
    Ok((LuaValue::Table(result), None))
}
