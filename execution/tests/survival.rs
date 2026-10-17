use mlua::Lua;

#[test]
fn survival_uses_public_api_and_restores_owned_inputs() -> mlua::Result<()> {
    let lua = Lua::new();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../mods/sf-survival/src");
    lua.globals().set(
        "survival_source",
        std::fs::read_to_string(root.join("mod.lua")).unwrap(),
    )?;
    lua.globals().set(
        "overrides_source",
        include_str!("../lua/shroudforge/v1/lua/overrides.lua"),
    )?;
    let targets: mlua::Function = lua
        .load(include_str!("../lua/shroudforge/v1/lua/override_targets.lua"))
        .set_name("survival-override-targets")
        .eval()?;
    lua.globals().set("override_targets_factory", targets)?;
    lua.load(include_str!("survival.lua"))
        .set_name("survival-lifecycle")
        .exec()
}
