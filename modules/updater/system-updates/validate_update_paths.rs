use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

pub(crate) fn validate_roots(root: &Path, staged: &Path) -> Result<(), String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("invalid install root: {error}"))?;
    let staged = staged
        .canonicalize()
        .map_err(|error| format!("invalid staged root: {error}"))?;
    let updates = shroudforge_loader_config::paths::updates_dir(&root);
    if !staged.starts_with(&updates) {
        return Err("staged update is outside the configured updates folder".into());
    }
    if root == staged {
        return Err("staged update must not equal install root".into());
    }
    Ok(())
}

pub(crate) fn validate_relative_path(relative: &str) -> Result<PathBuf, String> {
    let path = Path::new(relative);
    if relative.is_empty()
        || relative.contains('\\')
        || relative.contains(':')
        || relative.contains('\0')
        || !path
            .components()
            .all(|part| matches!(part, std::path::Component::Normal(_)))
    {
        return Err(format!("unsafe managed path: {relative}"));
    }
    Ok(path.to_path_buf())
}

pub(crate) fn reserved_game_path(key: &str) -> bool {
    matches!(
        key,
        "enshrouded.exe" | "enshrouded_server.exe" | "enshrouded.log" | "steam_api64.dll"
    ) || key.starts_with("enshrouded_")
        || key.starts_with("enshrouded.kfc")
}

pub(crate) fn protected_user_path(key: &str) -> bool {
    let protected = [
        "shroudforge/config/modloader-config.json",
        "shroudforge/config/.shroudforge-write.lock",
        "shroudforge/config/.shroudforge-mod-update-check.lock",
        "shroudforge/state.json",
        "shroudforge/cache/",
        "shroudforge/exports/",
        "shroudforge/logs/",
        "shroudforge/ui/",
        "shroudforge/updates/",
        "shroudforge/recycle-bin/",
        "shroudforge/runtime/heartbeat.json",
        "shroudforge/runtime/startup-assets.lock",
    ];
    protected.iter().any(|item| {
        if item.ends_with('/') {
            key.starts_with(item)
        } else {
            key == *item
        }
    })
}

pub(crate) fn validate_release_path(relative: &str) -> Result<PathBuf, String> {
    let path = validate_relative_path(relative)?;
    let key = relative.to_ascii_lowercase();
    if protected_user_path(&key) || reserved_game_path(&key) {
        return Err(format!(
            "release path is reserved for game or user data: {relative}"
        ));
    }
    Ok(path)
}

pub(crate) fn installed_path(root: &Path, relative: &str) -> PathBuf {
    if let Some(mod_relative) = relative.strip_prefix("mods/") {
        shroudforge_loader_config::paths::mods_dir(root).join(mod_relative)
    } else {
        root.join(relative)
    }
}

pub(crate) fn validate_file_path(root: &Path, relative: &str) -> Result<(), String> {
    let mut path = if relative.starts_with("mods/") {
        shroudforge_loader_config::paths::mods_dir(root)
    } else {
        root.to_path_buf()
    };
    // Never follow a junction/symlink into another installation or user directory.
    let relative_path = relative.strip_prefix("mods/").unwrap_or(relative);
    for component in Path::new(relative_path).components() {
        path.push(component);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!("reparse point in update path: {}", path.display()));
            }
            #[cfg(windows)]
            Ok(metadata) if metadata.file_attributes() & 0x400 != 0 => {
                return Err(format!("reparse point in update path: {}", path.display()));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    if path.exists() && !path.is_file() {
        return Err(format!("update target is not a file: {}", path.display()));
    }
    Ok(())
}
