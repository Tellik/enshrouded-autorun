use crate::provider_config::ProviderConfig;
use crate::release::Release;

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
    let Some(release) = shroudforge_release_feed::fetch_latest(
        &provider.base_url,
        &provider.project_id,
        current,
        provider.include_prereleases,
        true,
    )?
    else {
        return Ok(None);
    };
    let checksum = release
        .checksum
        .ok_or("GitHub release checksum was not fetched")?;
    let (mandatory, notes) = crate::parse_release_notes(&release.body);
    Ok(Some(Release {
        version: release.version,
        base_version: release.base_version,
        build: release.build,
        download_url: release.download_url,
        checksum,
        release_url: Some(release.release_url),
        prerelease: release.prerelease,
        mandatory,
        message: if notes.trim().is_empty() {
            "A new ShroudForge version is available on GitHub.".into()
        } else {
            notes
        },
    }))
}
