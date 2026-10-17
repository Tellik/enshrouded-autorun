use super::*;
pub(super) fn lua_name(lua: &mlua::Lua, args: FunctionArgs) -> mlua::Result<mlua::String> {
    let path = args.get::<String>(0)?;

    PathBuf::from(path)
        .file_name()
        .map(|s| lua.create_string(s))
        .unwrap_or_else(|| lua.create_string(""))
}

pub(super) fn lua_name_without_extension(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<mlua::String> {
    let path = args.get::<String>(0)?;

    PathBuf::from(path)
        .file_stem()
        .map(|s| lua.create_string(s))
        .unwrap_or_else(|| lua.create_string(""))
}

pub(super) fn lua_extension(lua: &mlua::Lua, args: FunctionArgs) -> mlua::Result<mlua::String> {
    let path = args.get::<String>(0)?;

    PathBuf::from(path)
        .extension()
        .map(|s| lua.create_string(s))
        .unwrap_or_else(|| lua.create_string(""))
}

pub(super) fn lua_parent(lua: &mlua::Lua, args: FunctionArgs) -> mlua::Result<mlua::String> {
    let path = args.get::<String>(0)?;

    PathBuf::from(path)
        .parent()
        .map(|p| into_forward_slash(p.to_path_buf()))
        .map(|s| lua.create_string(s.as_str()))
        .unwrap_or_else(|| lua.create_string(""))
}

pub(super) fn lua_join(lua: &mlua::Lua, args: FunctionArgs) -> mlua::Result<mlua::String> {
    let mut path = PathBuf::new();

    for p in 0..args.len() {
        let part = args.get::<String>(p)?;
        path.push(part);
    }

    let path = into_forward_slash(path);

    lua.create_string(path.as_str())
}
