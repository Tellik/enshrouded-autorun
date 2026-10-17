use crate::LoaderError;
use std::path::Path;

pub(super) fn target_name(game_directory: &Path) -> Result<&'static str, LoaderError> {
    if game_directory.join("enshrouded.exe").is_file() {
        Ok("enshrouded")
    } else if game_directory.join("enshrouded_server.exe").is_file() {
        Ok("enshrouded_server")
    } else {
        Err(LoaderError::Environment(format!(
            "no Enshrouded executable in {}",
            game_directory.display()
        )))
    }
}

pub(super) fn process_target_name(game_directory: &Path) -> Result<&'static str, LoaderError> {
    if let Ok(executable) = std::env::current_exe() {
        if executable
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("enshrouded_server.exe"))
        {
            return Ok("enshrouded_server");
        }
    }
    target_name(game_directory)
}
