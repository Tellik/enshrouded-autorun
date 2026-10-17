pub mod generate_game_type_definitions;
pub(crate) use generate_game_type_definitions as generator;

pub static DEFINITION_FILE: &str = include_str!(concat!(env!("OUT_DIR"), "/definitions.lua"));

pub(crate) mod export_lua_definitions;
