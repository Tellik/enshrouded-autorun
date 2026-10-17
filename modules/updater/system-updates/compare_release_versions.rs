use semver::Version;

pub fn parse_release_tag(tag: &str) -> Result<(String, u64), String> {
    let tag = tag.trim_start_matches('v');
    let (version, build) = tag
        .rsplit_once("-build.")
        .or_else(|| tag.rsplit_once("-dev."))
        .ok_or_else(|| format!("Invalid ShroudForge release tag: {tag}"))?;
    Version::parse(version)
        .map_err(|error| format!("Invalid ShroudForge version '{version}': {error}"))?;
    let build = build
        .parse::<u64>()
        .map_err(|_| format!("Invalid ShroudForge build ID: {build}"))?;
    Ok((version.to_owned(), build))
}

pub fn is_update_available(
    current_version: &str,
    current_build: u64,
    latest_version: &str,
    latest_build: u64,
) -> bool {
    let Ok(current) = Version::parse(current_version.trim_start_matches('v')) else {
        return false;
    };
    let Ok(latest) = Version::parse(latest_version.trim_start_matches('v')) else {
        return false;
    };
    latest > current || (latest == current && latest_build > current_build)
}
