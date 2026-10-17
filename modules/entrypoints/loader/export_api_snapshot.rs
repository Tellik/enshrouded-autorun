use crate::game_files::detect_files;
use kfc::{GameParser, KfcParser, export_api_snapshot};
use std::{ffi::OsString, path::PathBuf};

pub(crate) fn run(args: &[OsString]) -> Result<(), Box<dyn std::error::Error>> {
    let game = PathBuf::from(&args[2]);
    let schema = KfcParser.parse(&detect_files(&game))?;
    let result = export_api_snapshot(&schema, PathBuf::from(&args[3]))?;
    println!(
        "exported {} native types and {} fields to {}",
        result.type_count,
        result.field_count,
        result.directory.display()
    );
    Ok(())
}
