use crate::validate_update_paths::installed_path;
use std::{
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

const VERSION_MANIFEST: &str = "shroudforge/version.json";
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) fn apply(source: &Path, target: &Path, paths: &[String]) -> Result<(), String> {
    let mut version_manifest = false;
    for relative in paths {
        if relative == VERSION_MANIFEST {
            version_manifest = true;
            continue;
        }
        let incoming = source.join(relative);
        if !incoming.exists() {
            return Err(format!("release is missing managed path: {relative}"));
        }
        let current = installed_path(target, relative);
        copy_entry(&incoming, &current)?;
    }

    // Publish the manifest only after every other managed file is in place.
    // Readers can then observe the old complete manifest or the new complete
    // manifest, never the truncated file produced by copying over it directly.
    if version_manifest {
        let incoming = source.join(VERSION_MANIFEST);
        if !incoming.is_file() {
            return Err(format!(
                "release is missing managed path: {VERSION_MANIFEST}"
            ));
        }
        let current = installed_path(target, VERSION_MANIFEST);
        copy_file_atomically(&incoming, &current)?;
    }
    Ok(())
}

fn copy_file_atomically(source: &Path, target: &Path) -> Result<(), String> {
    let parent = target.parent().ok_or_else(|| {
        format!(
            "version manifest has no parent directory: {}",
            target.display()
        )
    })?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let source_metadata = fs::metadata(source).map_err(|error| error.to_string())?;
    let mut source_file = File::open(source).map_err(|error| error.to_string())?;
    let file_name = target
        .file_name()
        .ok_or_else(|| format!("version manifest has no file name: {}", target.display()))?;

    let (temporary, mut destination_file) = (0..32)
        .find_map(|_| {
            let mut name = OsString::from(".");
            name.push(file_name);
            name.push(format!(
                ".{}.{}.tmp",
                std::process::id(),
                TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            let temporary = parent.join(name);
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
            {
                Ok(file) => Some(Ok((temporary, file))),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => None,
                Err(error) => Some(Err(error)),
            }
        })
        .transpose()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "could not allocate a temporary version manifest".to_owned())?;

    let result = (|| -> io::Result<()> {
        io::copy(&mut source_file, &mut destination_file)?;
        destination_file.sync_all()?;
        drop(destination_file);
        fs::set_permissions(&temporary, source_metadata.permissions())?;
        replace_file_atomically(&temporary, target)
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&temporary);
        return Err(format!(
            "could not publish version manifest atomically: {error}"
        ));
    }
    Ok(())
}

#[cfg(windows)]
fn replace_file_atomically(source: &Path, target: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };

    let source = source
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let target = target
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let moved = unsafe {
        MoveFileExW(
            source.as_ptr(),
            target.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if moved == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn replace_file_atomically(source: &Path, target: &Path) -> io::Result<()> {
    fs::rename(source, target)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_root(label: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "shroudforge-{label}-{}-{stamp}",
            std::process::id()
        ))
    }

    #[test]
    fn manifest_is_not_published_when_another_managed_file_fails() {
        let root = temporary_root("manifest-order");
        let source = root.join("source");
        let target = root.join("target");
        fs::create_dir_all(source.join("shroudforge")).unwrap();
        fs::create_dir_all(target.join("shroudforge")).unwrap();
        fs::write(source.join(VERSION_MANIFEST), b"new manifest").unwrap();
        fs::write(target.join(VERSION_MANIFEST), b"old manifest").unwrap();

        let result = apply(
            &source,
            &target,
            &[
                VERSION_MANIFEST.to_owned(),
                "missing-managed-file.dll".to_owned(),
            ],
        );

        assert!(result.is_err());
        assert_eq!(
            fs::read(target.join(VERSION_MANIFEST)).unwrap(),
            b"old manifest"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn manifest_replacement_publishes_complete_new_contents() {
        let root = temporary_root("manifest-atomic");
        let source = root.join("source");
        let target = root.join("target");
        fs::create_dir_all(source.join("shroudforge")).unwrap();
        fs::create_dir_all(target.join("shroudforge")).unwrap();
        fs::write(source.join(VERSION_MANIFEST), b"new complete manifest").unwrap();
        fs::write(target.join(VERSION_MANIFEST), b"old complete manifest").unwrap();

        apply(&source, &target, &[VERSION_MANIFEST.to_owned()]).unwrap();

        assert_eq!(
            fs::read(target.join(VERSION_MANIFEST)).unwrap(),
            b"new complete manifest"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
