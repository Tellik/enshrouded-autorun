use crate::release::{InstalledRelease, Release};
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseState {
    pub current_version: String,
    pub installation_detected: bool,
    #[serde(skip)]
    pub current_build: u64,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub mandatory: bool,
    pub prerelease: bool,
    pub state: String,
    pub message: Option<String>,
    pub release_url: Option<String>,
    pub staged: bool,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub bytes_per_second: Option<u64>,
    pub completed_files: Option<usize>,
    pub total_files: Option<usize>,
    pub current_file: Option<String>,
    pub current_file_bytes: Option<u64>,
    pub current_file_total_bytes: Option<u64>,
    #[serde(skip)]
    pub release: Option<Release>,
}

impl ReleaseState {
    pub fn idle(installed: InstalledRelease, staged: bool) -> Self {
        Self {
            current_version: installed.version,
            installation_detected: false,
            current_build: installed.build,
            latest_version: None,
            update_available: false,
            mandatory: false,
            prerelease: false,
            state: "idle".into(),
            message: None,
            release_url: None,
            staged,
            downloaded_bytes: 0,
            total_bytes: None,
            bytes_per_second: None,
            completed_files: None,
            total_files: None,
            current_file: None,
            current_file_bytes: None,
            current_file_total_bytes: None,
            release: None,
        }
    }
}
