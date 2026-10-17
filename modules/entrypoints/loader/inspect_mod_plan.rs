use crate::game_files::detect_files;
use kfc::{GameParser, KfcParser};
use std::{ffi::OsString, path::PathBuf};

pub(crate) fn run(args: &[OsString]) -> Result<(), Box<dyn std::error::Error>> {
    let game = PathBuf::from(&args[2]);
    let is_client = game.join("enshrouded.exe").is_file();
    let schema = KfcParser.parse(&detect_files(&game))?;
    let root = game
        .to_str()
        .ok_or("game directory path must be valid UTF-8")?;
    let environment = shroudforge_package::ModEnvironment::load(root)
        .map_err(|report| format!("mod discovery failed: {report:?}"))?;
    let is_server = !is_client;
    let (plan, errors) = environment.plan_report(is_server, shroudforge_api::API_VERSION);
    let ids = plan
        .iter()
        .map(|item| item.info().id.as_str())
        .collect::<Vec<_>>();
    println!(
        "{} target: {}",
        if is_server { "Server" } else { "Client" },
        game.display()
    );
    println!("Planned mods: {}", ids.join(", "));
    for error in errors {
        eprintln!("Skipped mod: {error}");
    }
    let _ = schema;
    Ok(())
}
