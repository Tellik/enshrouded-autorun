use super::validate_message_name::{default_message_name_fallback, default_username_mode};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UiSettings {
    #[serde(default)]
    pub(crate) module_preferences: Option<serde_json::Value>,
    #[serde(default)]
    pub(crate) compact_mode: bool,
    #[serde(default)]
    pub(crate) reduced_motion: bool,
    #[serde(default = "default_username_mode")]
    pub(crate) username_mode: String,
    #[serde(default = "default_message_name_fallback")]
    pub(crate) message_name_fallback: String,
    pub(crate) log_level: String,
    #[serde(default)]
    pub(crate) export_enabled: bool,
    pub(crate) update_enabled: bool,
    pub(crate) base_url: String,
    pub(crate) project_id: String,
    pub(crate) check_minutes: u64,
}
