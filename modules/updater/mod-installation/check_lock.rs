use fs2::FileExt;
use std::{fs, io::ErrorKind, path::Path};

/// Holds the per-installation lock for a catalog mod-update check.
pub(crate) struct ModUpdateCheckLock {
    _file: fs::File,
}

impl ModUpdateCheckLock {
    pub(crate) fn try_acquire(root: &Path) -> Result<Option<Self>, String> {
        let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        let config_dir = shroudforge_loader_config::paths::config_dir(&root);
        fs::create_dir_all(&config_dir).map_err(|error| error.to_string())?;
        let path = config_dir.join(".shroudforge-mod-update-check.lock");
        let file = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)
            .map_err(|error| error.to_string())?;

        match file.try_lock_exclusive() {
            Ok(()) => Ok(Some(Self { _file: file })),
            Err(error) if error.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }
}
