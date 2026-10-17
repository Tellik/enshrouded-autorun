use mlua::Lua;

#[test]
fn reversible_overrides_stop_all_writes_after_uncertain_results() -> mlua::Result<()> {
    let lua = Lua::new();
    let overrides: mlua::Value = lua
        .load(include_str!("../lua/shroudforge/v1/lua/overrides.lua"))
        .set_name("runtime.overrides")
        .eval()?;
    lua.globals().set("overrides", overrides)?;
    lua.load(include_str!("runtime_overrides.lua"))
        .set_name("runtime-overrides-fault-regressions")
        .exec()
}
