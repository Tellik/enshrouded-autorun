use super::format_report;
use crate::LoaderError;
use std::path::Path;

pub(super) fn export_pass_needed(
    game_directory: &Path,
    file_name: &str,
) -> Result<bool, LoaderError> {
    let config = shroudforge_loader_config::config::read_loader(game_directory)
        .map_err(LoaderError::Pregame)?;
    if !config
        .pointer("/exports/enabled")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return Ok(false);
    }
    let root = game_directory.to_str().ok_or_else(|| {
        LoaderError::Environment(format!(
            "game directory is not valid UTF-8: {}",
            game_directory.display()
        ))
    })?;
    let environment = shroudforge_package::ModEnvironment::load(root)
        .map_err(|report| LoaderError::Environment(format_report(&report)))?;
    Ok(environment
        .plan(
            file_name == "enshrouded_server",
            shroudforge_api::API_VERSION,
        )
        .iter()
        .any(|item| {
            item.info().requires_pregame()
                && item
                    .info()
                    .capabilities
                    .contains(&shroudforge_package::Capability::Export)
        }))
}

pub(super) fn already_applied(game_directory: &Path, file_name: &str) -> Result<bool, LoaderError> {
    let game_utf8 = game_directory.to_str().ok_or_else(|| {
        LoaderError::Environment(format!(
            "game directory is not valid UTF-8: {}",
            game_directory.display()
        ))
    })?;
    let environment = shroudforge_package::ModEnvironment::load(game_utf8)
        .map_err(|report| LoaderError::Environment(format_report(&report)))?;
    let server = file_name == "enshrouded_server";
    let has_startup_mods = environment
        .plan(server, shroudforge_api::API_VERSION)
        .iter()
        .any(|item| item.info().requires_pregame());
    let has_previous_apply = shroudforge_loader_config::config::read_document(
        game_directory,
        shroudforge_game_assets::documents::APPLIED,
    )
    .ok()
    .is_some_and(|value| {
        matches!(
            value["status"].as_str(),
            Some("applied" | "preparing" | "failed")
        )
    });
    if !has_startup_mods && !has_previous_apply {
        return Ok(true);
    }
    let fingerprint = shroudforge_package::prepared::fingerprint(
        &environment,
        server,
        shroudforge_api::API_VERSION,
    )
    .map_err(LoaderError::Pregame)?;
    Ok(shroudforge_package::prepared::matches(
        game_directory,
        &fingerprint,
    ))
}
