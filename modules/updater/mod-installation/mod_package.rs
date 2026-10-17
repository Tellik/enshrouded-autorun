use serde::Deserialize;

pub const MAX_MOD_BYTES: u64 = 512 * 1024 * 1024;

pub const MAX_EXTRACTED_BYTES: u64 = 2 * 1024 * 1024 * 1024;

pub const MAX_ARCHIVE_ENTRIES: usize = 4096;

#[derive(Deserialize)]
pub struct ResolveContentPlan {
    pub primary: ResolvedContent,
    #[serde(default)]
    pub dependencies: Vec<ResolvedContent>,
}

#[derive(Deserialize)]
pub struct ResolvedContent {
    pub project_id: String,
    pub version_id: String,
}

#[derive(Deserialize)]
pub struct ApiVersion {
    pub id: String,
    pub project_id: String,
    #[serde(rename = "version_number")]
    pub _version_number: String,
    #[serde(default)]
    pub files: Vec<ApiVersionFile>,
}

#[derive(Deserialize)]
pub struct ApiVersionFile {
    pub url: String,
    pub filename: String,
    #[serde(default)]
    pub hashes: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub primary: bool,
    pub size: u64,
}
