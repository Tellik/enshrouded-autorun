use super::disable_mods_for_restore::disable_all_mods_for_gamefile_restore;
use super::read_host_arguments::Arguments;
use super::report_command_result::report_command_result;
use std::time::Instant;

pub(crate) fn restore_game_files(
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    next_refresh: &mut Instant,
) {
    let target = if arguments.server { "server" } else { "client" };
    let result = if shroudforge_game_assets::originals::originals_status(&arguments.root, target)["status"]
        != "ready"
    {
        Err("verified original game files are not available for this game build".into())
    } else {
        disable_all_mods_for_gamefile_restore(&arguments.root)
            .and_then(|()| shroudforge_game_assets::request_gamefiles_restore(&arguments.root))
    };
    report_command_result(
        &arguments.root,
        &webview,
        &request_id,
        "Backup Management",
        "Restore Gamefiles",
        result,
        "backups.restore.queued",
        None,
        false,
    );
    *next_refresh = Instant::now();
}
