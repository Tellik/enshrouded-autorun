use crate::catalog_search_state::CatalogState;
use crate::command_result::AsyncUiResult;
use crate::mod_installation::catalog::fetch_catalog;
use crate::provider_config::ProviderConfig;
use shroudforge_logging::write_activity;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;

pub fn start_catalog_search(
    root: PathBuf,
    provider: ProviderConfig,
    query: String,
    state: Arc<Mutex<CatalogState>>,
    results: mpsc::Sender<AsyncUiResult>,
    request_id: String,
) {
    let query = query.trim().chars().take(100).collect::<String>();
    if let Ok(mut value) = state.lock() {
        value.query = query.clone();
        value.items.clear();
        value.message = None;
        if !provider.enabled {
            let error = "ShroudEdit access is disabled.".to_owned();
            value.state = "error".into();
            value.message = Some(error.clone());
            let _ = results.send(AsyncUiResult {
                request_id,
                source: "ShroudEdit".into(),
                action: "Search catalog".into(),
                success_message: "catalog.searchFailed".into(),
                result: Err(error),
            });
            return;
        }
        value.state = "loading".into();
    }
    thread::spawn(move || {
        let result = fetch_catalog(&provider, &query);
        let mut failure = None;
        let mut success_detail = None;
        if let Ok(mut value) = state.lock() {
            match result {
                Ok(items) => {
                    success_detail = Some(format!(
                        "Catalog search completed: {} result(s).",
                        items.len()
                    ));
                    value.state = "ready".into();
                    value.message = if items.is_empty() {
                        Some("No matching ShroudForge mods found.".into())
                    } else {
                        None
                    };
                    value.items = items;
                }
                Err(error) => {
                    value.state = "error".into();
                    value.message = Some(error.clone());
                    failure = Some(error);
                }
            }
        }
        if let Some(detail) = success_detail {
            write_activity(
                &root,
                "ShroudEdit",
                "Search catalog",
                "Succeeded",
                Some(&detail),
                "debug",
            );
        }
        if let Some(error) = failure {
            let _ = results.send(AsyncUiResult {
                request_id,
                source: "ShroudEdit".into(),
                action: "Search catalog".into(),
                success_message: "catalog.searchFailed".into(),
                result: Err(error),
            });
        }
    });
}
