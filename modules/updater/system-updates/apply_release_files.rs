use crate::validate_update_paths::installed_path;
use std::{fs, path::Path};

pub(crate) fn apply(source: &Path, target: &Path, paths: &[String]) -> Result<(), String> {
    for relative in paths {
        let incoming = source.join(relative);
        if !incoming.exists() {
            return Err(format!("release is missing managed path: {relative}"));
        }
        let current = installed_path(target, relative);
        copy_entry(&incoming, &current)?;
    }
    Ok(())
}

pub(crate) fn copy_entry(source: &Path, target: &Path) -> Result<(), String> {
    if source.is_dir() {
        fs::create_dir_all(target).map_err(|error| error.to_string())?;
        for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            copy_entry(&entry.path(), &target.join(entry.file_name()))?;
        }
    } else {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::copy(source, target).map_err(|error| error.to_string())?;
    }
    Ok(())
}
