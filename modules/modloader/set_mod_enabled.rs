use crate::update_mod_extension;
use std::path::Path;

pub fn set_mod_enabled(root: &Path, id: &str, enabled: bool, revision: &str) -> Result<(), String> {
    update_mod_extension(root, id, revision, |extension| {
        extension["enabled"] = enabled.into();
        Ok(())
    })
}
