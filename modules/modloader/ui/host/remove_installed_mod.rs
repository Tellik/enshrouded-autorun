use shroudforge_package::package_lookup::find_mod_package;
use shroudforge_package::package_lookup::valid_identifier;
use shroudforge_updater::mod_installation::catalog::forget_catalog_install;
use std::fs;
use std::path::Path;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub(crate) fn remove_mod(root: &Path, id: &str) -> Result<(), String> {
    if !valid_identifier(id) {
        return Err("invalid mod id".into());
    }
    let source = find_mod_package(root, id).ok_or("mod package not found")?;
    let trash = shroudforge_loader_config::paths::ui_data_dir(root).join("trash");
    fs::create_dir_all(&trash).map_err(|error| error.to_string())?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let suffix = source.extension().and_then(|value| value.to_str());
    let destination = trash.join(match suffix {
        Some(extension) if source.is_file() => format!("{id}-{timestamp}.{extension}"),
        _ => format!("{id}-{timestamp}"),
    });
    fs::rename(source, destination).map_err(|error| error.to_string())?;
    forget_catalog_install(root, id)
}
