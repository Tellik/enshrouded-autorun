use crate::game_files::game_executable;
use shroudforge_modloader::prepare;
use std::{ffi::OsString, path::PathBuf, process::Command};

pub(crate) fn run(args: &[OsString]) -> Result<(), Box<dyn std::error::Error>> {
    let game = PathBuf::from(&args[2]);
    prepare(&game)?;
    let executable = game_executable(&game)?;
    let forwarded = args
        .iter()
        .skip(3)
        .skip_while(|argument| argument.to_str() == Some("--"));
    let status = Command::new(&executable)
        .current_dir(&game)
        .args(forwarded)
        .status()?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}
