use super::loader_settings_input::UiSettings;
use super::read_host_arguments::Arguments;
use super::report_command_result::report_command_result;
use super::save_loader_settings::save_settings;
use shroudforge_updater::provider_config::CatalogProviderConfig;
use std::time::Instant;

pub(crate) fn apply_loader_settings(
    settings: UiSettings,
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    active_provider: &mut CatalogProviderConfig,
    next_refresh: &mut Instant,
) {
    let result = save_settings(&arguments.root, &settings);
    *next_refresh = Instant::now();
    if result.is_ok() {
        active_provider.base_url = settings.base_url;
        active_provider.project_id = settings.project_id;
        *next_refresh = Instant::now();
    }
    report_command_result(
        &arguments.root,
        &webview,
        &request_id,
        "Modloader",
        "Save settings",
        result,
        "settings.saved",
        None,
        true,
    );
}
