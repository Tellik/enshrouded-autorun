#[derive(Clone)]
pub struct Release {
    pub version: String,
    pub base_version: String,
    pub build: u64,
    pub download_url: String,
    pub checksum: String,
    pub release_url: Option<String>,
    pub message: String,
    pub mandatory: bool,
    pub prerelease: bool,
}

pub struct InstalledRelease {
    pub version: String,
    pub build: u64,
}
