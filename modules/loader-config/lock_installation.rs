use fs2::FileExt;
use std::{fs, path::Path};

pub fn installation_lock(root: &Path) -> Result<fs::File, String> {
    fs::create_dir_all(crate::paths::config_dir(root)).map_err(|error| error.to_string())?;
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(crate::paths::config_dir(root).join(".shroudforge-write.lock"))
        .map_err(|error| error.to_string())?;
    lock.lock_exclusive().map_err(|error| error.to_string())?;
    Ok(lock)
}
