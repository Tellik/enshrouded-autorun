use std::{ffi::OsString, path::PathBuf};

pub(crate) fn run(args: &[OsString]) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(game) = args.get(2) {
        let _ = shroudforge_logging::initialize(PathBuf::from(game), true);
    }
    match args.get(1).and_then(|value| value.to_str()) {
        Some("types") if args.len() == 4 => {
            crate::export_api_snapshot::run(args)?;
        }
        Some("prepare") if args.len() == 3 => {
            crate::prepare_game::run(args)?;
        }
        Some("create") if (args.len() == 5 || args.len() == 6) => {
            crate::create_mod_project::run(args)?;
        }
        Some("restore") if args.len() == 3 => {
            crate::restore_game_assets::run(args)?;
        }
        Some("launch") if args.len() >= 3 => {
            crate::launch_game::run(args)?;
        }
        Some("inspect") if args.len() == 3 => {
            crate::inspect_mod_plan::run(args)?;
        }
        _ => {
            eprintln!("usage: shroudforge types <game-directory> <output-directory>");
            eprintln!(
                "       shroudforge create <game-directory> <mod-id> <mod-name> [patch,export,runtime]"
            );
            eprintln!("       shroudforge prepare <game-directory>");
            eprintln!("       shroudforge restore <game-directory>");
            eprintln!("       shroudforge launch <game-directory> [-- <game-arguments>]");
            eprintln!("       shroudforge inspect <game-directory>");
            std::process::exit(2);
        }
    }
    Ok(())
}
