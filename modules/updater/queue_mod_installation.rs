use crate::command_result::AsyncUiResult;
use crate::provider_config::CatalogProviderConfig;
use shroudforge_package::package_lookup::valid_identifier;
use std::path::PathBuf;
use std::sync::mpsc;

pub fn start_mod_install(
    root: PathBuf,
    provider: CatalogProviderConfig,
    project_id: String,
    title: String,
    results: mpsc::Sender<AsyncUiResult>,
    request_id: String,
) {
    if provider.kind != "shroudedit" {
        let _ = results.send(AsyncUiResult {
            request_id,
            source: "ShroudEdit".into(),
            action: "Install mod".into(),
            success_message: "mod.install.queued".into(),
            result: Err("Unsupported catalog provider".into()),
        });
        return;
    }
    if !valid_identifier(&project_id) {
        let _ = results.send(AsyncUiResult {
            request_id,
            source: "ShroudEdit".into(),
            action: "Install mod".into(),
            success_message: "mod.install.succeeded".into(),
            result: Err("Invalid project ID".into()),
        });
        return;
    }
    let result = crate::request_mod_install(
        &root,
        &project_id,
        &title,
        serde_json::to_value(provider).unwrap_or_default(),
    );
    let _ = results.send(AsyncUiResult {
        request_id,
        source: "ShroudEdit".into(),
        action: "Queue mod install".into(),
        success_message: "mod.install.queued".into(),
        result: result.map(|()| "Mod install queued in the independent updater.".into()),
    });
}
