use mlua::Lua;

#[test]
fn reversible_overrides_stop_all_writes_after_uncertain_results() -> mlua::Result<()> {
    let lua = Lua::new();
    let overrides: mlua::Value = lua
        .load(include_str!(
            "../lua/ingame/overrides/track_runtime_overrides.lua"
        ))
        .set_name("runtime.overrides")
        .eval()?;
    lua.globals().set("overrides", overrides)?;
    lua.load(include_str!("override_recovery.lua"))
        .set_name("runtime-overrides-fault-regressions")
        .exec()
}
