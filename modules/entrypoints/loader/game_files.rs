use kfc::GameFiles;
use std::path::PathBuf;

pub(crate) fn game_executable(game: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    for name in ["enshrouded.exe", "enshrouded_server.exe"] {
        let executable = game.join(name);
        if executable.is_file() {
            return Ok(executable);
        }
    }
    Err(format!("no Enshrouded executable in {}", game.display()).into())
}

pub(crate) fn detect_files(game: &PathBuf) -> GameFiles {
    if game.join("enshrouded.exe").is_file() {
        GameFiles::client(game)
    } else {
        GameFiles::server(game)
    }
}
