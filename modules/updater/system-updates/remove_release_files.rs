use std::{fs, path::Path};

pub(crate) fn remove_entry(path: &Path) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "could not inspect update target {}: {error}",
                path.display()
            ));
        }
    };
    retry_file_operation(|| {
        if metadata.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        }
    })
    .map_err(|error| format!("could not remove update target {}: {error}", path.display()))
}

fn retry_file_operation<T>(
    mut operation: impl FnMut() -> std::io::Result<T>,
) -> std::io::Result<T> {
    const ATTEMPTS: usize = 10;
    for attempt in 0..ATTEMPTS {
        match operation() {
            Err(error)
                if (error.kind() == std::io::ErrorKind::WouldBlock
                    || matches!(error.raw_os_error(), Some(5 | 32 | 33)))
                    && attempt + 1 < ATTEMPTS =>
            {
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            result => return result,
        }
    }
    unreachable!()
}

pub(crate) fn remove_empty_mod_directories(root: &Path, file: &Path) -> Result<(), String> {
    let mods = shroudforge_loader_config::paths::mods_dir(root);
    let mut directory = file.parent();
    while let Some(path) = directory {
        if path == mods || !path.starts_with(&mods) {
            break;
        }
        let mut permissions = match fs::metadata(path) {
            Ok(metadata) => metadata.permissions(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => {
                return Err(format!(
                    "could not inspect mod directory {}: {error}",
                    path.display()
                ));
            }
        };
        if permissions.readonly() {
            permissions.set_readonly(false);
            fs::set_permissions(path, permissions).map_err(|error| {
                format!(
                    "could not clear the read-only attribute on mod directory {}: {error}",
                    path.display()
                )
            })?;
        }
        match retry_file_operation(|| fs::remove_dir(path)) {
            Ok(()) => directory = path.parent(),
            Err(error)
                if error.kind() == std::io::ErrorKind::DirectoryNotEmpty
                    || error.kind() == std::io::ErrorKind::NotFound =>
            {
                break;
            }
            Err(error) => {
                return Err(format!(
                    "could not remove empty mod directory {}: {error}",
                    path.display()
                ));
            }
        }
    }
    Ok(())
}
