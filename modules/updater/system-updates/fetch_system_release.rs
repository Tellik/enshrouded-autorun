use crate::compare_release_versions::parse_release_tag;
use crate::github_release::GithubRelease;
use crate::provider_config::ProviderConfig;
use crate::release::Release;
use semver::Version;
use std::time::Duration;

pub fn fetch_latest(provider: &ProviderConfig, current: &str) -> Result<Option<Release>, String> {
    if provider.kind != "github-releases" {
        return Err(format!("unsupported update provider: {}", provider.kind));
    }
    fetch_latest_github(provider, current)
}

pub fn fetch_latest_github(
    provider: &ProviderConfig,
    current: &str,
) -> Result<Option<Release>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent(format!("ShroudForge/{current} (modloader-ui)"))
        .build()
        .map_err(|error| error.to_string())?;
    let url = format!(
        "{}/repos/{}/releases?per_page=100",
        provider.base_url.trim_end_matches('/'),
        provider.project_id.trim_matches('/')
    );
    let response = client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .map_err(|error| format!("GitHub is unreachable: {error}"))?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !response.status().is_success() {
        return Err(format!("GitHub returned HTTP {}", response.status()));
    }
    let releases: Vec<GithubRelease> = response
        .json()
        .map_err(|error| format!("Invalid GitHub response: {error}"))?;
    let mut eligible = releases
        .into_iter()
        .filter(|release| !release.draft && (provider.include_prereleases || !release.prerelease))
        .filter_map(|release| {
            let version = release.tag_name.trim_start_matches('v').to_owned();
            let (base_version, build) = parse_release_tag(&version).ok()?;
            let parsed_version = Version::parse(&base_version).ok()?;
            let archive = release.assets.iter().find(|asset| {
                asset.name.starts_with("shroudforge-") && asset.name.ends_with(".zip")
            })?;
            let checksum_name = format!("{}.sha256", archive.name);
            let checksum_asset = release
                .assets
                .iter()
                .find(|asset| asset.name == checksum_name)?;
            let archive_url = archive.browser_download_url.clone();
            let checksum_url = checksum_asset.browser_download_url.clone();
            Some((
                parsed_version,
                build,
                release,
                archive_url,
                checksum_url,
                base_version,
                version,
            ))
        })
        .collect::<Vec<_>>();
    eligible.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    let Some((_, build, release, archive_url, checksum_url, base_version, version)) =
        eligible.pop()
    else {
        return Ok(None);
    };
    let checksum_response = client
        .get(&checksum_url)
        .send()
        .map_err(|error| format!("Could not download GitHub checksum: {error}"))?;
    if !checksum_response.status().is_success() {
        return Err(format!(
            "GitHub checksum endpoint returned HTTP {}",
            checksum_response.status()
        ));
    }
    let checksum_text = checksum_response
        .text()
        .map_err(|error| format!("Could not read GitHub checksum: {error}"))?;
    let checksum = checksum_text
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if checksum.len() != 64 || !checksum.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("GitHub release does not contain a valid SHA-256 checksum".into());
    }
    let (mandatory, notes) = crate::parse_release_notes(&release.body);
    Ok(Some(Release {
        version,
        base_version,
        build,
        download_url: archive_url,
        checksum,
        release_url: Some(release.html_url),
        prerelease: release.prerelease,
        mandatory,
        message: if notes.trim().is_empty() {
            "A new ShroudForge version is available on GitHub.".into()
        } else {
            notes
        },
    }))
}
