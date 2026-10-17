use serde::Deserialize;

#[derive(Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub body: String,
    pub html_url: String,
    #[serde(default)]
    pub assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
pub struct GithubAsset {
    pub name: String,
    pub browser_download_url: String,
}
