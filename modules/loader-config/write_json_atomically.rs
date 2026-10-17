use serde_json::Value;
use std::{fs, io::Write, path::Path};

pub fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let parent = path.parent().ok_or("missing config parent")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temporary
        .write_all(&serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    let mut temporary = temporary;
    for attempt in 0..5 {
        match temporary.persist(path) {
            Ok(_) => return Ok(()),
            Err(error)
                if error.error.kind() == std::io::ErrorKind::PermissionDenied && attempt < 4 =>
            {
                temporary = error.file;
                std::thread::sleep(std::time::Duration::from_millis(10 << attempt));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("state file replacement retries exhausted".into())
}
