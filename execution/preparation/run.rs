use crate::{
    API_VERSION, RunArgs, ShroudForgeApi,
    type_aliases::{Path, PathBuf as Utf8PathBuf},
    cache::{CacheDiff, FileStateCache},
    env::{AppFeatures, AppState},
    game_files,
    logging::{debug, info},
    runner::LuaModRunner,
};
use mod_loader::ModEnvironment;

pub fn run(env: &ModEnvironment, api: ShroudForgeApi, args: RunArgs) -> anyhow::Result<()> {
    run_with_api(env, Some(api), args)
}

pub fn run_with_local_schema(env: &ModEnvironment, args: RunArgs) -> anyhow::Result<()> {
    run_with_api(env, None, args)
}

fn run_with_api(
    env: &ModEnvironment,
    api: Option<ShroudForgeApi>,
    mut args: RunArgs,
) -> anyhow::Result<()> {
    debug!("Running lua with options: {:?}", args);

    // check cache if files have changed

    let cache_diff = if !args.options.skip_cache {
        let current_cache = FileStateCache::read(env.cache_dir());
        let mut new_cache = FileStateCache::default();

        new_cache.track_game_files(env.game_dir());
        new_cache.track_mod_files(env.mods_dir());
        new_cache.track_mod_config(env.game_dir());

        let cache_diff = new_cache.diff(&current_cache);

        if cache_diff.is_none() && args.options.patch && !args.options.force_patch {
            args.options.patch = false;

            debug!("No changes detected, skipping asset writes");
        }

        cache_diff
    } else {
        debug!("Skipping cache check");
        CacheDiff::new_dirty()
    };

    // create a new app state

    let app_state = match AppState::new(env.clone(), api, args, &cache_diff) {
        Ok(context) => context,
        Err(_) => anyhow::bail!("Failed to create AppState"),
    };

    // setup and run all mods

    let runner = LuaModRunner::new(app_state)?;

    let is_server = runner.lua.app_data_ref::<AppState>().unwrap().is_server();
    match runner.setup(env.plan(is_server, API_VERSION)) {
        Ok(_) => {}
        Err(err) => {
            anyhow::bail!("Failed to setup LuaModRunner: {}", err);
        }
    }

    debug!("Running mods...");

    // TODO: move this somewhere else, maybe to the GameContext?

    runner.run()?;

    let app_state = runner.lua.app_data_ref::<AppState>().unwrap();

    if app_state.has_feature(AppFeatures::PATCH) {
        debug!("Committing typed asset changes...");

        let mut buf = Vec::new();
        let mut writer = app_state.take_writer()?;
        let resources = app_state.get_cached_resources();
        let mut applied_resources = 0;

        for resource in resources.iter() {
            let value = resource.apply(&runner.lua)?;

            if let Some(value) = value {
                let r#type = kfc::kfc_format::type_for_resource(
                    app_state.type_registry(),
                    &resource.resource_id,
                )
                .expect("resource type must exist in current game registry");

                buf.clear();
                value
                    .write_into(app_state.type_registry(), r#type, &mut buf)
                    .unwrap();
                writer.write_resource(&resource.resource_id, &buf)?;
                applied_resources += 1;
            }
        }

        writer.finalize()?;
        app_state.commit_assets()?;

        info!("Committed {} changed resources", applied_resources);
    }

    // create a new cache file

    if app_state.has_feature(AppFeatures::PATCH) {
        let mut new_cache = FileStateCache::default();

        new_cache.track_game_files(env.game_dir());
        new_cache.track_mod_files(env.mods_dir());
        new_cache.track_mod_config(env.game_dir());

        new_cache.write(env.cache_dir());
    } else {
        let mut new_cache = FileStateCache::read(env.cache_dir());

        new_cache.track_game_files(env.game_dir());
        new_cache.track_mod_config(env.game_dir());
        new_cache.write(env.cache_dir());
    }

    Ok(())
}

pub fn export_lua_definitions(game_dir: impl AsRef<Path>, file_name: &str, force: bool) -> bool {
    let game_dir = game_dir.as_ref();
    let Ok(cache_dir) =
        Utf8PathBuf::from_path_buf(shroudforge_loader_config::paths::cache_dir(game_dir.as_std_path()))
    else {
        return false;
    };
    let (type_registry, is_dirty) =
        match crate::game_files::load_type_registry(game_dir, &cache_dir, file_name) {
            Ok(type_registry) => type_registry,
            Err(_) => return false,
        };

    crate::game_files::export_lua_definitions(&cache_dir, &type_registry, is_dirty || force);

    true
}

pub fn restore(game_dir: impl AsRef<Path>, file_name: &str) -> bool {
    let kfc_path = game_dir.as_ref().join(format!("{file_name}.kfc"));
    if game_files::restore_backup(&kfc_path).is_err() {
        return false;
    }
    let resources = game_dir.as_ref().join(format!("{file_name}.kfc_resources"));
    game_files::restore_companion_backup(resources).is_ok()
}

pub fn backup_companion_file(path: impl AsRef<std::path::Path>) -> bool {
    game_files::create_companion_backup(path).is_ok()
}
