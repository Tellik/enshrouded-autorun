use crate::env::AppState;

#[path = "../assets/files.rs"]
mod assets;
mod content_id_casts;
mod guid;
mod types;

pub(crate) use assets::loaded_index::Indexes as LoadedResourceIndexes;
pub(crate) use assets::loaded_index::tick as tick_loaded_resource_indexes;
pub use assets::{Content, Resource, value};
pub use types::Type;

pub fn create(lua: &mlua::Lua, r#mod: &mod_loader::Mod) -> mlua::Result<mlua::Table> {
    let table = lua.create_table()?;
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    table.raw_set("version", app_state.kfc_file().game_version())?;
    table.raw_set("assets", assets::create(lua, r#mod)?)?;
    table.raw_set("types", types::create(lua)?)?;
    table.raw_set("guid", guid::create(lua)?)?;

    Ok(table)
}
