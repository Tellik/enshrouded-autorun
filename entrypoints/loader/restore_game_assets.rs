use std::{ffi::OsString, path::PathBuf};

pub(crate) fn run(args: &[OsString]) -> Result<(), Box<dyn std::error::Error>> {
    let game = PathBuf::from(&args[2]);
    let file_name = if game.join("enshrouded.exe").is_file() {
        "enshrouded"
    } else {
        "enshrouded_server"
    };
    let game_utf8 = game.to_str().ok_or("game directory must be valid UTF-8")?;
    if !shroudforge_api::restore(game_utf8, file_name) {
        return Err(format!("could not restore the original {file_name}.kfc file").into());
    }
    println!("original game data restored");
    Ok(())
}
