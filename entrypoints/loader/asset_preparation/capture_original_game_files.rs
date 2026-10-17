use std::path::Path;

pub(super) fn ensure_original_gamefiles(game_directory: &Path, file_name: &str) {
    match shroudforge_game_assets::originals::ensure_originals(game_directory, file_name) {
        Ok(status) => tracing::info!(target: "shroudforge::backup", target = file_name,
            state = %status["status"], "Original game-file backup status checked"),
        Err(error) => tracing::warn!(target: "shroudforge::backup", target = file_name,
            %error, "Could not capture original game files"),
    }
}
