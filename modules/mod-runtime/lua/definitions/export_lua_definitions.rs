use crate::{
    logging::debug,
    type_aliases::{Path, PathBuf as Utf8PathBuf},
};
use kfc::reflection::TypeRegistry;
pub(crate) fn export_to_cache(cache_dir: &Path, type_registry: &TypeRegistry, force: bool) {
    let lua_cache_dir = cache_dir.join("lua");

    if let Err(e) = std::fs::create_dir_all(&lua_cache_dir) {
        debug!(
            error = %e,
            path = ?lua_cache_dir,
            "Failed to create lua cache directory, unable to generate lua definitions",
        );
    } else {
        let lua_type_def_path = lua_cache_dir.join("types.lua");
        let lua_base_def_path = lua_cache_dir.join("base.lua");

        if force || !lua_type_def_path.exists() {
            let def = crate::definition::generator::generate(type_registry);

            match std::fs::write(&lua_type_def_path, def) {
                Ok(_) => debug!(
                    path = ?lua_type_def_path,
                    "Lua type definition file has been generated",
                ),
                Err(e) => {
                    debug!(
                        error = %e,
                        path = ?lua_type_def_path,
                        "Failed to write lua type definition file",
                    );
                }
            }
        }

        if force || !lua_base_def_path.exists() {
            match std::fs::write(&lua_base_def_path, crate::definition::DEFINITION_FILE) {
                Ok(_) => debug!(
                    path = ?lua_base_def_path,
                    "Lua base definition file has been generated",
                ),
                Err(e) => {
                    debug!(
                        error = %e,
                        path = ?lua_base_def_path,
                        "Failed to write lua base definition file",
                    );
                }
            }
        }
    }
}

pub fn export_lua_definitions(game_dir: impl AsRef<Path>, file_name: &str, force: bool) -> bool {
    let game_dir = game_dir.as_ref();
    let Ok(cache_dir) = Utf8PathBuf::from_path_buf(shroudforge_loader_config::paths::cache_dir(
        game_dir.as_std_path(),
    )) else {
        return false;
    };
    let (type_registry, is_dirty) = match crate::reflection::load_type_registry::load_type_registry(
        game_dir, &cache_dir, file_name,
    ) {
        Ok(type_registry) => type_registry,
        Err(_) => return false,
    };

    crate::lua::definitions::export_lua_definitions::export_to_cache(
        &cache_dir,
        &type_registry,
        is_dirty || force,
    );

    true
}
