mod command_line;
mod create_mod_project;
mod export_api_snapshot;
mod game_files;
mod inspect_mod_plan;
mod launch_game;
mod prepare_game;
mod process_modes;
mod restore_game_assets;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() == 1 {
        return process_modes::launch_desktop();
    }
    if let Some(result) = process_modes::run(&args) {
        return result;
    }
    command_line::run(&args)
}
