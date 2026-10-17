use std::fs;
use std::path::Path;

pub(crate) fn read_game_version(root: &Path) -> String {
    fs::read(shroudforge_loader_config::paths::version_file(root))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|value| {
            value
                .get("gameVersion")
                .and_then(|value| value.as_str())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "Automatisch erkannt".into())
}
