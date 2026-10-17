use std::path::Path;

pub fn set_mod_enabled(root: &Path, id: &str, enabled: bool, revision: &str) -> Result<(), String> {
    crate::update_mod_configuration::update_mod_configuration(root, id, revision, |extension| {
        extension["enabled"] = enabled.into();
        Ok(())
    })
}
