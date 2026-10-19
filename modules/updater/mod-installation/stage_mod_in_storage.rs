use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

/// Copy a verified package into the configured mods volume before committing it.
/// Mod storage can be on a different drive from the download and update folders,
/// where a direct rename would fail with a cross-device error.
pub(crate) fn stage_mod_in_storage(
    root: &Path,
    package: &Path,
    mod_id: &str,
) -> Result<LocalModStage, String> {
    if !shroudforge_package::package_lookup::valid_identifier(mod_id) {
        return Err("invalid mod ID for installation staging".into());
    }
    let mods = shroudforge_loader_config::paths::mods_dir(root);
    fs::create_dir_all(&mods).map_err(|error| {
        format!(
            "could not create configured mod storage {}: {error}",
            mods.display()
        )
    })?;
    let staging_root = mods.join(".shroudforge-staging");
    fs::create_dir_all(&staging_root).map_err(|error| {
        format!(
            "could not prepare temporary mod storage {}: {error}",
            staging_root.display()
        )
    })?;
    let metadata = fs::symlink_metadata(&staging_root).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "temporary mod storage must not be a symbolic link: {}",
            staging_root.display()
        ));
    }
    #[cfg(windows)]
    if metadata.file_attributes() & 0x400 != 0 {
        return Err(format!(
            "temporary mod storage must not be a reparse point: {}",
            staging_root.display()
        ));
    }
    if !metadata.is_dir() {
        return Err(format!(
            "temporary mod storage is not a directory: {}",
            staging_root.display()
        ));
    }

    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let local_staging = staging_root.join(format!("{}-{}-{id}", std::process::id(), mod_id));
    let cancel_path = crate::check_update_cancelled::cancel_path(root);
    crate::recycle_bin::restore_directory_with_cancellation(
        package,
        &local_staging,
        Some(&cancel_path),
    )
    .map_err(|error| {
        if error == "UPDATE_CANCELLED" {
            error
        } else {
            format!("could not verify the package copy in configured mod storage: {error}")
        }
    })?;
    Ok(LocalModStage {
        path: local_staging,
    })
}

pub(crate) struct LocalModStage {
    path: PathBuf,
}

impl LocalModStage {
    pub(crate) fn commit(self, destination: &Path) -> Result<(), String> {
        fs::rename(&self.path, destination).map_err(|error| {
            format!(
                "could not move the verified package into {}: {error}",
                destination.display()
            )
        })
    }
}

impl Drop for LocalModStage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
        if let Some(parent) = self.path.parent() {
            let _ = fs::remove_dir(parent);
        }
    }
}
