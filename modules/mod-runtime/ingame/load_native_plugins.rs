use crate::{env::AppState, runner::LuaModRunner};
use mod_loader::ModEnvironment;

pub(super) fn load_native_plugins(
    env: &ModEnvironment,
    runner: &LuaModRunner,
    errors: &mut serde_json::Map<String, serde_json::Value>,
) {
    let state = runner.lua.app_data_ref::<AppState>().unwrap();
    // Usually prepared before the game entrypoint. This also supports callers
    // that create an in-game runtime directly and reuses process-owned handles.
    if let Err(error) =
        crate::native_dlls::apply_plan(env.game_dir().as_std_path(), state.is_server(), false)
    {
        tracing::error!(target: "shroudforge::dll_manager", %error, "DLL Manager could not prepare native DLLs");
        state.report_runtime_effect("dll-manager", "error", &error);
        errors.insert("dll-manager".into(), error.into());
    }
    state.poll_native_dll_loads();
    for status in crate::native_dlls::statuses() {
        if status.state == "error" {
            errors.insert(status.id, status.detail.into());
        }
    }
}
