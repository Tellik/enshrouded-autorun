use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub const MAX_EXPORT_BYTES: usize = 4 * 1024 * 1024;

/// Diagnostic results use the installation's existing export directory.
#[derive(Clone, Debug)]
pub struct JobExports {
    directory: PathBuf,
}

impl JobExports {
    pub(crate) fn new(root: &Path, job_id: &str) -> Self {
        Self {
            directory: shroudforge_loader_config::paths::export_dir(root)
                .join("diagnostics")
                .join(job_id),
        }
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    fn path(&self, name: &str) -> Result<PathBuf, String> {
        if name.is_empty()
            || name.len() > 80
            || matches!(name, "." | "..")
            || !name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
        {
            return Err(
                "diagnostic export must be one file name of at most 80 ASCII characters".into(),
            );
        }
        let path = self.directory.join(name);
        if fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            return Err("diagnostic exports cannot be symbolic links".into());
        }
        Ok(path)
    }

    pub fn exists(&self, name: &str) -> Result<bool, String> {
        match fs::metadata(self.path(name)?) {
            Ok(metadata) if metadata.is_file() => Ok(true),
            Ok(_) => Err("diagnostic export is not a regular file".into()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn read(&self, name: &str) -> Result<String, String> {
        let path = self.path(name)?;
        let metadata = fs::metadata(&path).map_err(|error| error.to_string())?;
        if !metadata.is_file() || metadata.len() > MAX_EXPORT_BYTES as u64 {
            return Err("diagnostic export is not a regular file within the 4 MiB limit".into());
        }
        let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
        if text.len() > MAX_EXPORT_BYTES {
            return Err("diagnostic export grew past 4 MiB".into());
        }
        Ok(text)
    }

    /// Persist and verify before returning, including before a probe's world call.
    pub fn write(&self, name: &str, text: &str) -> Result<(), String> {
        if text.len() > MAX_EXPORT_BYTES {
            return Err("diagnostic export exceeds 4 MiB".into());
        }
        let path = self.path(name)?;
        fs::create_dir_all(&self.directory).map_err(|error| error.to_string())?;
        let mut temporary =
            tempfile::NamedTempFile::new_in(&self.directory).map_err(|error| error.to_string())?;
        temporary
            .write_all(text.as_bytes())
            .map_err(|error| error.to_string())?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|error| error.to_string())?;
        temporary.persist(path).map_err(|error| error.to_string())?;
        if self.read(name)? != text {
            return Err("diagnostic export readback differs from persisted data".into());
        }
        Ok(())
    }
}
