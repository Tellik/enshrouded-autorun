use crate::command_result::AsyncUiResult;
use crate::mod_installation::catalog::catalog_project_for_mod;
use crate::provider_config::CatalogProviderConfig;
use std::path::PathBuf;
use std::sync::mpsc;

pub fn start_mod_update(
    root: PathBuf,
    provider: CatalogProviderConfig,
    mod_id: String,
    title: String,
    results: mpsc::Sender<AsyncUiResult>,
    request_id: String,
) {
    if provider.kind != "shroudedit" {
        let _ = results.send(AsyncUiResult {
            request_id,
            source: mod_id,
            action: "Update mod".into(),
            success_message: "mod.update.queued".into(),
            result: Err("Unsupported catalog provider".into()),
        });
        return;
    }
    let project_id = catalog_project_for_mod(&root, &mod_id);
    let result = match project_id {
        Some(project_id) => crate::request_mod_update(
            &root,
            &project_id,
            &mod_id,
            &title,
            serde_json::to_value(provider).unwrap_or_default(),
        )
        .map(|()| format!("Update queued for {mod_id}.")),
        None => Err("This mod is not registered as a ShroudEdit installation.".into()),
    };
    let _ = results.send(AsyncUiResult {
        request_id,
        source: mod_id,
        action: "Update mod".into(),
        success_message: "mod.update.queued".into(),
        result,
    });
}
