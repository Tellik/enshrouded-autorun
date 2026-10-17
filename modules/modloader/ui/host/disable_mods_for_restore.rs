use super::read_installed_mods::read_mods;
use shroudforge_package::set_mod_enabled;
use std::path::Path;

pub(crate) fn disable_all_mods_for_gamefile_restore(root: &Path) -> Result<(), String> {
    let enabled = read_mods(root)
        .into_iter()
        .filter(|mod_info| mod_info.enabled)
        .collect::<Vec<_>>();
    let mut errors = Vec::new();
    let mut disabled = Vec::new();
    for item in &enabled {
        match set_mod_enabled(root, &item.id, false, &item.revision) {
            Ok(()) => disabled.push(item.id.clone()),
            Err(error) => errors.push(format!("{}: {error}", item.id)),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        let current = read_mods(root);
        for id in &disabled {
            if let Some(item) = current.iter().find(|item| &item.id == id) {
                if let Err(error) = set_mod_enabled(root, id, true, &item.revision) {
                    errors.push(format!(
                        "could not restore {} to its previous enabled state: {error}",
                        id
                    ));
                }
            }
        }
        Err(format!(
            "Could not disable every mod. Game files were not queued for restoration. {}",
            errors.join(", ")
        ))
    }
}
