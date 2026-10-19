use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogProviderConfig {
    #[serde(default = "default_provider_kind")]
    pub kind: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_project_id")]
    pub project_id: String,
    #[serde(default = "default_loader")]
    pub loader: String,
}

impl Default for CatalogProviderConfig {
    fn default() -> Self {
        Self {
            kind: default_provider_kind(),
            base_url: default_base_url(),
            project_id: default_project_id(),
            loader: default_loader(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_provider_kind")]
    pub kind: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_project_id")]
    pub project_id: String,
    #[serde(default = "default_loader")]
    pub loader: String,
    #[serde(default = "default_check_minutes")]
    pub check_minutes: u64,
    #[serde(default)]
    pub include_prereleases: bool,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            kind: default_provider_kind(),
            base_url: default_base_url(),
            project_id: default_project_id(),
            loader: default_loader(),
            check_minutes: default_check_minutes(),
            include_prereleases: false,
        }
    }
}

pub fn default_provider_kind() -> String {
    "shroudedit".into()
}

pub fn default_base_url() -> String {
    "https://api.shroudedit.com".into()
}

pub fn default_project_id() -> String {
    "shroudforge".into()
}

pub fn default_loader() -> String {
    "shroudforge".into()
}

pub fn default_check_minutes() -> u64 {
    60
}

pub fn default_system_provider() -> ProviderConfig {
    ProviderConfig {
        enabled: true,
        kind: "github-releases".into(),
        base_url: shroudforge_release_feed::GITHUB_API_BASE_URL.into(),
        project_id: shroudforge_release_feed::SHROUDFORGE_PROJECT_ID.into(),
        loader: default_loader(),
        check_minutes: 60,
        include_prereleases: false,
    }
}
