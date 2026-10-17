use super::*;
pub(super) fn lua_read(_lua: &mlua::Lua, args: FunctionArgs, r#mod: &Mod) -> mlua::Result<Buffer> {
    let path = args.get::<String>(0)?;

    let mut filesystem = r#mod.fs();
    let buffer = read_bytes(&mut filesystem, &path)?;

    Ok(Buffer::wrap(buffer))
}

pub(super) fn lua_read_to_string(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<mlua::String> {
    let path = args.get::<String>(0)?;

    let mut filesystem = r#mod.fs();
    let buffer = read_bytes(&mut filesystem, &path)?;

    lua.create_string(&buffer)
}

pub(super) fn lua_list_files(
    _lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<Vec<String>> {
    let path = args.get::<String>(0)?;

    let fs = r#mod.fs();
    let mod_id = &r#mod.info().id;

    let files = fs.read_directory(&path)?;
    let mut result = Vec::with_capacity(files.len());

    for file in files {
        let file = match file {
            Ok(p) => into_forward_slash(p),
            Err(e) => {
                warn_span!("lua", mod_id).in_scope(|| {
                    warn!(
                        error = %e,
                        path = path.as_str(),
                        "failed to read directory",
                    );
                });

                continue;
            }
        };

        result.push(file);
    }

    Ok(result)
}

pub(super) fn lua_exists(_lua: &mlua::Lua, args: FunctionArgs, r#mod: &Mod) -> mlua::Result<bool> {
    let path = args.get::<String>(0)?;
    let fs = r#mod.fs();

    Ok(fs.exists(path))
}

pub(super) fn lua_is_file(_lua: &mlua::Lua, args: FunctionArgs, r#mod: &Mod) -> mlua::Result<bool> {
    let path = args.get::<String>(0)?;
    let mut fs = r#mod.fs();

    Ok(fs.is_file(path))
}

pub(super) fn lua_is_directory(
    _lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<bool> {
    let path = args.get::<String>(0)?;
    let mut fs = r#mod.fs();

    Ok(fs.is_directory(path))
}

fn read_bytes(filesystem: &mut mod_loader::FileSystem, path: &str) -> mlua::Result<Vec<u8>> {
    let file = filesystem.read_file(path)?;

    // read all bytes

    use std::io::Read;
    let mut buffer = Vec::new();
    let mut reader = BufReader::new(file);
    reader.read_to_end(&mut buffer)?;

    Ok(buffer)
}
