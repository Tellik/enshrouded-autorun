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

#[test]
fn crafting_scale_updates_loaded_recipes_and_restores_originals() -> mlua::Result<()> {
    let lua = Lua::new();
    lua.globals().set(
        "crafting_source",
        include_str!("../../../mods/sf-crafting-essentials/src/mod.lua"),
    )?;
    lua.globals().set(
        "overrides_source",
        include_str!("../lua/ingame/overrides/track_runtime_overrides.lua"),
    )?;
    let targets: mlua::Function = lua
        .load(include_str!(
            "../lua/ingame/overrides/create_override_targets.lua"
        ))
        .eval()?;
    lua.globals().set("override_targets_factory", targets)?;
    lua.load(include_str!("crafting_lifecycle.lua"))
        .set_name("crafting-live-production-scale")
        .exec()
}
