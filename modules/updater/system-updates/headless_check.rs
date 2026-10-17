use crate::*;
#[cfg(windows)]
pub(crate) fn headless_check_system_updates(
    root: &std::path::Path,
    manual: bool,
) -> Result<String, String> {
    use semver::Version;

    let repair_required = super::refresh_cached_repair_status(root)?;
    let config = shroudforge_loader_config::config::read_loader(root)?;
    let repair_without_release = repair_required
        && !config["modules"]["updates"]["system"]["latestRelease"].is_object();
    let _check_lock = match crate::update_check::begin_system_update_check(
        root, manual || repair_without_release,
    )? {
        crate::update_check::SystemUpdateCheckStart::Started(lock) => lock,
        crate::update_check::SystemUpdateCheckStart::CoolingDown {
            remaining_seconds,
        } => {
            return Ok(format!(
                "Recent system update check reused. Next automatic trigger allowed in {remaining_seconds} seconds."
            ));
        }
        crate::update_check::SystemUpdateCheckStart::AlreadyRunning => {
            return Ok("System update check is already running in another process.".into());
        }
    };

    let include_prereleases = config
        .pointer("/modules/updates/system/includePrereleases")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let current = std::fs::read(shroudforge_loader_config::paths::version_file(root))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
    let current_version = current
        .as_ref()
        .and_then(|value| value["version"].as_str())
        .unwrap_or(env!("CARGO_PKG_VERSION"));
    let current_build = current
        .as_ref()
        .and_then(|value| value["build"].as_u64())
        .or_else(|| {
            current
                .as_ref()
                .and_then(|value| value["build"].as_str()?.parse().ok())
        })
        .unwrap_or(0);

    #[derive(serde::Deserialize)]
    struct GithubRelease {
        tag_name: String,
        #[serde(default)]
        draft: bool,
        #[serde(default)]
        prerelease: bool,
        #[serde(default)]
        body: String,
        html_url: String,
        #[serde(default)]
        assets: Vec<GithubAsset>,
    }
    #[derive(serde::Deserialize)]
    struct GithubAsset {
        name: String,
        browser_download_url: String,
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent(format!("ShroudForge/{current_version} updater"))
        .build()
        .map_err(|error| error.to_string())?;
    let response = client
        .get("https://api.github.com/repos/bonsaibauer/shroudforge/releases?per_page=100")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .map_err(|error| format!("GitHub is unreachable: {error}"))?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        let checked_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        shroudforge_loader_config::config::update_loader(root, |value| {
            value["modules"]["updates"]["system"]["latestRelease"] = serde_json::Value::Null;
            value["modules"]["updates"]["system"]["lastCheckAt"] = serde_json::json!(checked_at);
            Ok(())
        })?;
        return Ok("No ShroudForge release has been published yet".into());
    }
    if !response.status().is_success() {
        return Err(format!("GitHub returned HTTP {}", response.status()));
    }
    let releases: Vec<GithubRelease> = response
        .json()
        .map_err(|error| format!("Invalid GitHub response: {error}"))?;
    let mut eligible = releases
        .into_iter()
        .filter(|release| !release.draft && (include_prereleases || !release.prerelease))
        .filter_map(|release| {
            let version = release.tag_name.trim_start_matches('v').to_owned();
            let (base_version, build) = version
                .rsplit_once("-build.")
                .or_else(|| version.rsplit_once("-dev."))?;
            let parsed = Version::parse(base_version).ok()?;
            let build = build.parse::<u64>().ok()?;
            let archive = release.assets.iter().find(|asset| {
                asset.name.starts_with("shroudforge-") && asset.name.ends_with(".zip")
            })?;
            let checksum_name = format!("{}.sha256", archive.name);
            let checksum = release
                .assets
                .iter()
                .find(|asset| asset.name == checksum_name)?;
            let base_version = base_version.to_owned();
            let download_url = archive.browser_download_url.clone();
            let checksum_url = checksum.browser_download_url.clone();
            Some((
                parsed,
                build,
                version,
                base_version,
                release,
                download_url,
                checksum_url,
            ))
        })
        .collect::<Vec<_>>();
    eligible.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    let Some((_, build, version, base_version, release, download_url, checksum_url)) =
        eligible.pop()
    else {
        let checked_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        shroudforge_loader_config::config::update_loader(root, |value| {
            value["modules"]["updates"]["system"]["latestRelease"] = serde_json::Value::Null;
            value["modules"]["updates"]["system"]["lastCheckAt"] = serde_json::json!(checked_at);
            Ok(())
        })?;
        return Ok("No compatible ShroudForge release was found".into());
    };
    let checksum_response = client
        .get(checksum_url)
        .send()
        .map_err(|error| format!("Could not download GitHub checksum: {error}"))?;
    if !checksum_response.status().is_success() {
        return Err(format!(
            "GitHub checksum endpoint returned HTTP {}",
            checksum_response.status()
        ));
    }
    let checksum = checksum_response
        .text()
        .map_err(|error| format!("Could not read GitHub checksum: {error}"))?
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if checksum.len() != 64 || !checksum.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("GitHub release does not contain a valid SHA-256 checksum".into());
    }
    let repair_required = !super::missing_required_files(root).is_empty();
    let update_available = repair_required || Version::parse(current_version.trim_start_matches('v'))
        .ok()
        .zip(Version::parse(&base_version).ok())
        .is_some_and(|(current, latest)| {
            latest > current || (latest == current && build > current_build)
        });
    let (release_mandatory, notes) = super::parse_release_notes(&release.body);
    let mandatory = release_mandatory || repair_required;
    let message = if notes.trim().is_empty() {
        "A new ShroudForge version is available on GitHub.".to_owned()
    } else {
        notes.chars().take(20_000).collect()
    };
    let checked_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    shroudforge_loader_config::config::update_loader(root, |value| {
        value["modules"]["updates"]["system"]["lastCheckAt"] = serde_json::json!(checked_at);
        value["modules"]["updates"]["system"]["latestRelease"] = serde_json::json!({
            "version": version.clone(),
            "baseVersion": base_version.clone(),
            "build": build,
            "downloadUrl": download_url.clone(),
            "checksum": checksum.clone(),
            "releaseUrl": release.html_url.clone(),
            "message": message.clone(),
            "prerelease": release.prerelease,
            "mandatory": mandatory,
            "repairRequired": repair_required,
            "updateAvailable": update_available,
            "checkedAt": checked_at
        });
        Ok(())
    })?;
    Ok(if update_available {
        format!("ShroudForge update {version} is available")
    } else {
        "ShroudForge is up to date".into()
    })
}


#[cfg(windows)]
pub fn startup_check(root: &std::path::Path) -> Result<bool, String> {
    let missing = missing_required_files(root);
    if !missing.is_empty() {
        let detail = format!(
            "Installation requires repair. Missing or invalid files: {}",
            missing.join(", ")
        );
        let _ = shroudforge_logging::append(root, 'E', "updater", &detail);
        scheduled::open_update_window(root).map_err(|error| {
            let error = format!("{detail}. Could not open the repair updater: {error}");
            let _ = shroudforge_logging::append(root, 'E', "updater", &error);
            error
        })?;
        return Ok(true);
    }
    if let Err(error) = headless_check_system_updates(root, false) {
        let _ = shroudforge_logging::append(
            root,
            'W',
            "updater",
            &format!("Startup update check did not complete. Game startup continues: {error}"),
        );
        return Ok(false);
    }

    let release = match shroudforge_loader_config::config::read_loader(root)
        .ok()
        .and_then(|config| config.pointer("/modules/updates/system/latestRelease").cloned())
    {
        Some(release) if release["updateAvailable"].as_bool() == Some(true) => release,
        _ => return Ok(false),
    };
    let Some(version) = release["version"].as_str() else {
        return Ok(false);
    };
    let mandatory = release["mandatory"].as_bool() == Some(true);
    let previous_prompt = shroudforge_loader_config::config::read_loader(root)
        .ok()
        .and_then(|config| {
            config
                .pointer("/modules/updates/system/startupPromptedVersion")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        });
    if !mandatory && previous_prompt.as_deref() == Some(version) {
        return Ok(false);
    }

    match scheduled::open_update_window(root) {
        Ok(()) => {
            if !mandatory {
                let version = version.to_owned();
                if let Err(error) = shroudforge_loader_config::config::update_loader(root, |config| {
                    config["modules"]["updates"]["system"]["startupPromptedVersion"] =
                        serde_json::Value::String(version);
                    Ok(())
                }) {
                    let _ = shroudforge_logging::append(root, 'W', "updater",
                        &format!("Could not save the startup update prompt: {error}"));
                }
            }
            Ok(mandatory)
        }
        Err(error) => {
            let _ = shroudforge_logging::append(
                root,
                'E',
                "updater",
                &format!("Startup update window could not be opened: {error}"),
            );
            if mandatory { Err(error) } else { Ok(false) }
        }
    }
}


#[cfg(not(windows))]
pub fn startup_check(_: &std::path::Path) -> Result<bool, String> {
    Ok(false)
}
