use mlua::Lua;

#[test]
fn bundled_lua_compiles() -> mlua::Result<()> {
    let lua = Lua::new();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../mods");
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.path().extension().is_some_and(|ext| ext == "lua") {
            lua.load(std::fs::read_to_string(entry.path()).unwrap())
                .set_name(entry.path().to_string_lossy())
                .into_function()?;
        }
    }
    Ok(())
}
