use crate::env::{AppState, game, loader, registration, runtime_services, runtime_templates};
use mlua::Table;
use mod_loader::Mod;
pub(crate) mod claim_runtime_storage;
pub(crate) mod track_runtime_overrides;
pub(crate) fn attach(
    lua: &mlua::Lua,
    runtime: &Table,
    game: &Table,
    owner: &Mod,
) -> mlua::Result<()> {
    let overrides: Table = runtime.raw_get("overrides")?;
    claim_runtime_storage::attach(lua, &overrides, owner)?;
    let factory: mlua::Function = lua
        .load(include_str!("create_override_targets.lua"))
        .set_name("runtime.overrides.targets")
        .eval()?;
    let targets: Table = factory.call((runtime.clone(), game.raw_get::<Table>("assets")?))?;
    for pair in targets.pairs::<String, mlua::Value>() {
        let (name, target) = pair?;
        overrides.raw_set(name, target)?;
    }
    Ok(())
}
