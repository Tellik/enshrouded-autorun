use std::{fs, path::Path};

pub(crate) fn remove_entry(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|error| error.to_string())
    } else {
        fs::remove_file(path).map_err(|error| error.to_string())
    }
}

pub(crate) fn remove_empty_mod_directories(root: &Path, file: &Path) -> Result<(), String> {
    let mods = shroudforge_loader_config::paths::mods_dir(root);
    let mut directory = file.parent();
    while let Some(path) = directory {
        if path == mods || !path.starts_with(&mods) {
            break;
        }
        match fs::remove_dir(path) {
            Ok(()) => directory = path.parent(),
            Err(error)
                if error.kind() == std::io::ErrorKind::DirectoryNotEmpty
                    || error.kind() == std::io::ErrorKind::NotFound =>
            {
                break;
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(())
}
