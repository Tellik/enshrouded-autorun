use crate::LoaderError;
use kfc::{GameFiles, GameParser, KfcParser};
use shroudforge_api::ShroudForgeApi;
use shroudforge_compatibility::Compatibility;
use std::path::Path;

pub(super) fn parse_game_assets(
    game_directory: &Path,
    file_name: &str,
) -> Result<ShroudForgeApi, LoaderError> {
    let files = if file_name == "enshrouded" {
        GameFiles::client(game_directory)
    } else {
        GameFiles::server(game_directory)
    };
    let game_file = game_directory.join(if file_name == "enshrouded" {
        "enshrouded.exe"
    } else {
        "enshrouded_server.exe"
    });
    let metadata =
        std::fs::metadata(&game_file).map_err(|error| LoaderError::Pregame(error.to_string()))?;
    let modified = metadata
        .modified()
        .map_err(|error| LoaderError::Pregame(error.to_string()))?
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let completed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let schema = match KfcParser.parse(&files) {
        Ok(schema) => {
            shroudforge_loader_config::config::write_document(game_directory,shroudforge_game_assets::documents::PARSER_STATUS,&serde_json::json!({
        "status":"parsed","target":file_name,"completedAt":completed,"gameFileSize":metadata.len(),"gameFileModified":modified
    })).map_err(LoaderError::Pregame)?;
            schema
        }
        Err(error) => {
            let detail = error.to_string();
            let _ = shroudforge_loader_config::config::write_document(
                game_directory,
                shroudforge_game_assets::documents::PARSER_STATUS,
                &serde_json::json!({
                    "status":"failed","target":file_name,"completedAt":completed,"gameFileSize":metadata.len(),"gameFileModified":modified,"error":detail
                }),
            );
            return Err(LoaderError::Pregame(detail));
        }
    };
    let api = ShroudForgeApi::new(Compatibility::default().resolve(schema));

    Ok(api)
}
