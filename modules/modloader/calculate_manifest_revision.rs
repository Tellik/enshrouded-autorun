use crate::config::revision;
use std::{io::Read, path::Path};
pub fn manifest_revision(package: &Path) -> Result<String, String> {
    let path = camino::Utf8Path::from_path(package).ok_or("package path is not UTF-8")?;
    let mut package_fs = if package.is_dir() {
        crate::FileSystem::new_disk(path)
    } else {
        crate::FileSystem::new_zip(path)
    }
    .map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    for name in ["mod.json", "extended.mod.json"] {
        bytes.extend_from_slice(name.as_bytes());
        bytes.push(0);
        match package_fs.read_file(name) {
            Ok(mut file) => {
                file.read_to_end(&mut bytes)
                    .map_err(|error| error.to_string())?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => bytes.push(0xff),
            Err(error) => return Err(format!("{}: {error}", package.display())),
        }
        bytes.push(0);
    }
    Ok(revision(&bytes))
}
