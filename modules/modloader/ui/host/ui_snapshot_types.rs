use super::read_ui_configuration::{default_refresh, default_toggle_key};
use serde::Deserialize;
use serde::Serialize;
use shroudforge_updater::catalog_search_state::CatalogState;
use shroudforge_updater::provider_config::default_system_provider;
use shroudforge_updater::provider_config::{CatalogProviderConfig, ProviderConfig};
use shroudforge_updater::release_state::ReleaseState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModuleConfig {
    #[serde(default = "default_toggle_key")]
    pub(crate) toggle_key: u32,
    #[serde(default = "default_refresh")]
    pub(crate) refresh_milliseconds: u64,
    #[serde(skip, default = "default_updates_enabled")]
    pub(crate) updates_enabled: bool,
    #[serde(skip, default = "default_system_provider")]
    pub(crate) system_update_provider: ProviderConfig,
}

fn default_updates_enabled() -> bool {
    true
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserConfig {
    #[serde(default)]
    pub(crate) update_provider: Option<CatalogProviderConfig>,
    #[serde(default)]
    pub(crate) locale: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModInfo {
    pub(crate) revision: String,
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) dependencies: Vec<ModDependencyInfo>,
    pub(crate) ecosystem: &'static str,
    pub(crate) target: String,
    pub(crate) asset: bool,
    pub(crate) runtime: bool,
    pub(crate) description: Option<String>,
    pub(crate) authors: Vec<String>,
    pub(crate) license: Option<String>,
    pub(crate) icon: Option<String>,
    pub(crate) links: shroudforge_package::ModLinks,
    pub(crate) source: &'static str,
    pub(crate) enabled: bool,
    pub(crate) settings: Vec<serde_json::Value>,
    pub(crate) groups: Vec<serde_json::Value>,
    pub(crate) changelog: Vec<String>,
    pub(crate) assets: std::collections::HashMap<String, String>,
    pub(crate) setting_values: serde_json::Value,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModDependencyInfo {
    pub(crate) id: String,
    pub(crate) version: String,
    pub(crate) optional: bool,
    pub(crate) installed_version: Option<String>,
    pub(crate) status: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Activity {
    pub(crate) id: String,
    pub(crate) time: String,
    pub(crate) source: String,
    pub(crate) action: String,
    pub(crate) result: String,
    pub(crate) level: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Snapshot {
    pub(crate) diagnostics: serde_json::Value,
    pub(crate) windows: serde_json::Value,
    pub(crate) configuration: serde_json::Value,
    pub(crate) connected: bool,
    pub(crate) mode: &'static str,
    pub(crate) updater_window: bool,
    pub(crate) game_version: String,
    pub(crate) version: String,
    pub(crate) mods: Vec<ModInfo>,
    pub(crate) activity: Vec<Activity>,
    pub(crate) notices: Vec<shroudforge_news::Notice>,
    pub(crate) news_templates: serde_json::Value,
    pub(crate) read_notice_ids: Vec<String>,
    pub(crate) release: ReleaseState,
    pub(crate) update_queue: serde_json::Value,
    pub(crate) backup_management: serde_json::Value,
    pub(crate) settings: SnapshotSettings,
    pub(crate) catalog: CatalogState,
    pub(crate) locale: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SnapshotSettings {
    pub(crate) config_revision: String,
    pub(crate) module_preferences: serde_json::Value,
    pub(crate) compact_mode: bool,
    pub(crate) reduced_motion: bool,
    pub(crate) username_mode: String,
    pub(crate) detected_username: Option<String>,
    pub(crate) message_name_fallback: String,
    pub(crate) log_level: String,
    pub(crate) export_enabled: bool,
    pub(crate) runtime_profile_id: Option<String>,
    pub(crate) active_runtime_profile: Option<String>,
    pub(crate) runtime_profiles: Vec<String>,
    pub(crate) runtime_profile_directory: String,
    pub(crate) diagnostic_jobs: Vec<shroudforge_runtime_diagnostics::DiagnosticJob>,
    pub(crate) diagnostic_job_errors: Vec<String>,
    pub(crate) diagnostic_job_directory: String,
    pub(crate) diagnostic_selected_job_id: String,
    pub(crate) base_url: String,
    pub(crate) project_id: String,
    pub(crate) paths: Vec<StoragePath>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StoragePath {
    pub(crate) id: String,
    pub(crate) path: String,
    pub(crate) default_path: String,
}
