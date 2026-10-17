use mod_loader::Mod;

use crate::lua::FunctionArgs;

#[inline]
pub fn add_function<F, R>(
    lua: &mlua::Lua,
    table: &mlua::Table,
    name: &str,
    func: F,
) -> mlua::Result<()>
where
    F: Fn(&mlua::Lua, FunctionArgs) -> mlua::Result<R> + mlua::MaybeSend + 'static,
    R: mlua::IntoLuaMulti,
{
    let function = lua.create_function(func)?;

    table.raw_set(name, function)?;

    Ok(())
}

#[inline]
pub fn add_function_with_mod<F, R>(
    lua: &mlua::Lua,
    table: &mlua::Table,
    name: &str,
    r#mod: &Mod,
    func: F,
) -> mlua::Result<()>
where
    F: Fn(&mlua::Lua, FunctionArgs, &Mod) -> mlua::Result<R> + mlua::MaybeSend + 'static,
    R: mlua::IntoLuaMulti,
{
    let mod_clone = r#mod.clone();
    let function =
        lua.create_function(move |lua, args: FunctionArgs| func(lua, args, &mod_clone))?;

    table.raw_set(name, function)?;

    Ok(())
}
