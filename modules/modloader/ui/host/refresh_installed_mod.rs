use super::read_host_arguments::Arguments;
use super::read_installed_mods::read_mods;
use super::report_command_result::report_command_result;
use std::time::Instant;

pub(crate) fn refresh_installed_mod(
    id: String,
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    next_refresh: &mut Instant,
) {
    let result = if read_mods(&arguments.root)
        .iter()
        .any(|mod_info| mod_info.id == id)
    {
        Ok(())
    } else {
        Err(format!("Mod '{id}' is no longer installed"))
    };
    report_command_result(
        &arguments.root,
        &webview,
        &request_id,
        &id,
        "Refresh mod",
        result,
        "mod.refreshed",
        None,
        false,
    );
    *next_refresh = Instant::now();
}
