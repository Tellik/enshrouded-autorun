use super::{
    evaluate_attributes::evaluate_attributes, read_attribute::read_attribute,
    read_attribute_catalog::read_attribute_catalog,
    read_attribute_descriptor::read_attribute_descriptor, read_attributes::read_attributes,
    update_attribute::update_attribute, write_attribute_storage::write_attribute_storage,
};
use mlua::{Lua, Table};
use mod_loader::Mod;

pub(crate) fn attach(lua: &Lua, ecs: &Table, r#mod: &Mod) -> mlua::Result<()> {
    ecs.raw_set(
        "evaluate_attributes",
        lua.create_function(evaluate_attributes)?,
    )?;
    let reader = r#mod.clone();
    ecs.raw_set(
        "read_attributes",
        lua.create_function(move |lua, args: mlua::MultiValue| {
            read_attributes(lua, args, &reader)
        })?,
    )?;
    let updater = r#mod.clone();
    ecs.raw_set(
        "update_attribute",
        lua.create_function(move |lua, args: mlua::MultiValue| {
            update_attribute(lua, args, &updater)
        })?,
    )?;
    ecs.raw_set(
        "get_attributes",
        lua.create_function(read_attribute_catalog)?,
    )?;
    ecs.raw_set(
        "get_attribute",
        lua.create_function(read_attribute_descriptor)?,
    )?;
    let scalar_reader = r#mod.clone();
    ecs.raw_set(
        "read_attribute",
        lua.create_function(move |lua, args: mlua::MultiValue| {
            read_attribute(lua, args, &scalar_reader)
        })?,
    )?;
    let scalar_writer = r#mod.clone();
    ecs.raw_set(
        "write_attribute_storage",
        lua.create_function(move |lua, args: mlua::MultiValue| {
            write_attribute_storage(lua, args, &scalar_writer)
        })?,
    )?;
    Ok(())
}
