use crate as shroudforge_updater;
use crate::extract_mod_archive::extract_zip;
use crate::mod_package::{ApiVersion, MAX_MOD_BYTES, ResolvedContent};
use crate::provider_config::CatalogProviderConfig;
use crate::verify_mod_checksum::ModHashVerifier;
use shroudforge_package::package_lookup::find_mod_package;
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

struct TemporaryDownload {
    path: PathBuf,
    file: Option<fs::File>,
}

impl TemporaryDownload {
    fn create(path: PathBuf) -> Result<Self, String> {
        let file = fs::File::create(&path).map_err(|error| error.to_string())?;
        Ok(Self {
            path,
            file: Some(file),
        })
    }

    fn file_mut(&mut self) -> &mut fs::File {
        self.file.as_mut().expect("temporary download is open")
    }

    fn close(&mut self) {
        self.file.take();
    }
}

impl Drop for TemporaryDownload {
    fn drop(&mut self) {
        self.close();
        let _ = fs::remove_file(&self.path);
    }
}

struct StagingDirectory {
    path: PathBuf,
    keep: bool,
}

impl Drop for StagingDirectory {
    fn drop(&mut self) {
        if !self.keep {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

pub fn download_catalog_version(
    root: &Path,
    provider: &CatalogProviderConfig,
    client: &reqwest::blocking::Client,
    resolved: &ResolvedContent,
    update_mod_id: Option<&str>,
    queue_item_id: &str,
) -> Result<(String, PathBuf), String> {
    shroudforge_updater::set_active_queue_item_state(
        root,
        queue_item_id,
        "downloading",
        Some("Downloading mod package"),
    )?;
    let response = client
        .get(format!(
            "{}/v3/version/{}",
            provider.base_url.trim_end_matches('/'),
            resolved.version_id
        ))
        .send()
        .map_err(|error| format!("Could not load version metadata: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Version metadata endpoint returned HTTP {}",
            response.status()
        ));
    }
    let version: ApiVersion = response
        .json()
        .map_err(|error| format!("Invalid version metadata: {error}"))?;
    if version.id != resolved.version_id || version.project_id != resolved.project_id {
        return Err("Version metadata does not belong to the resolved project".into());
    }
    let file = version
        .files
        .iter()
        .find(|file| file.primary)
        .or_else(|| version.files.first())
        .ok_or("Version contains no mod package")?;
    if !file.filename.to_ascii_lowercase().ends_with(".zip")
        || !file.url.starts_with("https://")
        || file.size > MAX_MOD_BYTES
    {
        return Err("Mod package is not a valid ZIP archive".into());
    }
    let response = client
        .get(&file.url)
        .send()
        .map_err(|error| format!("Could not read mod package: {error}"))?;
    if !response.status().is_success() {
        return Err(format!("Mod download returned HTTP {}", response.status()));
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_MOD_BYTES)
    {
        return Err("Mod package exceeds 512 MiB".into());
    }
    let downloads = shroudforge_loader_config::paths::ui_data_dir(root).join("downloads");
    fs::create_dir_all(&downloads).map_err(|error| error.to_string())?;
    let temporary = downloads.join(format!(
        "{}-{}-{}.zip",
        resolved.project_id,
        resolved.version_id,
        std::process::id()
    ));
    let mut download = TemporaryDownload::create(temporary.clone())?;
    let mut hasher = ModHashVerifier::new(&file.hashes)?;
    let mut response = response.take(MAX_MOD_BYTES + 1);
    let mut buffer = vec![0u8; 1024 * 1024];
    let mut total = 0u64;
    loop {
        shroudforge_updater::check_update_cancelled(root)?;
        let count = response
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        total = total.saturating_add(count as u64);
        if total > MAX_MOD_BYTES {
            return Err("Mod package exceeds 512 MiB".into());
        }
        download
            .file_mut()
            .write_all(&buffer[..count])
            .map_err(|error| error.to_string())?;
        hasher.update(&buffer[..count]);
    }
    shroudforge_updater::check_update_cancelled(root)?;
    download
        .file_mut()
        .flush()
        .map_err(|error| error.to_string())?;
    download.close();
    hasher.verify()?;
    shroudforge_updater::set_active_queue_item_state(
        root,
        queue_item_id,
        "installing",
        Some("Download verified. Preparing mod installation"),
    )?;

    let manifest = shroudforge_package::config::read_manifest_path(root, &temporary);
    let manifest = match manifest {
        Ok(manifest) => manifest,
        Err(error) => {
            return Err(format!("Invalid mod.json: {error}"));
        }
    };
    if let Err(error) = shroudforge_package::validate_manifest(&manifest) {
        return Err(format!("Invalid mod manifest: {error}"));
    }
    if update_mod_id.is_none() && find_mod_package(root, &manifest.id).is_some() {
        return Err(format!("Mod '{}' is already installed", manifest.name));
    }
    if let Some(expected_id) = update_mod_id
        && manifest.id != expected_id
    {
        return Err(format!(
            "catalog update package ID '{}' does not match installed mod '{expected_id}'",
            manifest.id
        ));
    }
    let mods = shroudforge_loader_config::paths::mods_dir(root);
    fs::create_dir_all(&mods).map_err(|error| error.to_string())?;
    let staging_root = if update_mod_id.is_some() {
        shroudforge_loader_config::paths::updates_dir(root).join("mod-staging")
    } else {
        downloads.clone()
    };
    fs::create_dir_all(&staging_root).map_err(|error| error.to_string())?;
    let staging = staging_root.join(format!(
        "{}-{}-extract",
        manifest.id,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir(&staging).map_err(|error| error.to_string())?;
    let mut staging_cleanup = StagingDirectory {
        path: staging.clone(),
        keep: false,
    };
    extract_zip(&temporary, &staging)?;
    let extracted = shroudforge_package::config::read_manifest_path(root, &staging)?;
    if extracted.id != manifest.id || !staging.join("src/mod.lua").is_file() {
        return Err("extracted mod must have the same id and a src/mod.lua entrypoint".into());
    }
    let destination = mods.join(&manifest.id);
    if let Some(expected_id) = update_mod_id {
        if manifest.id != expected_id {
            return Err("updated mod identity changed".into());
        }
        staging_cleanup.keep = true;
        Ok((manifest.id, staging))
    } else {
        if destination.exists() {
            return Err("mod destination already exists".into());
        }
        crate::stage_mod_in_storage::stage_mod_in_storage(root, &staging, &manifest.id)?
            .commit(&destination)?;
        Ok((manifest.id, destination))
    }
}
