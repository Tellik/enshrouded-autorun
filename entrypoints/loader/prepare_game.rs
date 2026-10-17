use shroudforge_modloader::prepare;
use std::{ffi::OsString, path::PathBuf};

pub(crate) fn run(args: &[OsString]) -> Result<(), Box<dyn std::error::Error>> {
    prepare(PathBuf::from(&args[2]))?;
    println!("pregame phase complete");
    Ok(())
}
