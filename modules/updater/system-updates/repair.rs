use crate::*;
/// Reuses verified release metadata for repairs without another network check.
/// Returns whether the installed files currently require repair.
pub fn refresh_cached_repair_status(root: &std::path::Path) -> Result<bool, String> {
    if missing_required_files(root).is_empty() {
        return Ok(false);
    }
    let config = shroudforge_loader_config::config::read_loader(root)?;
    let latest = &config["modules"]["updates"]["system"]["latestRelease"];
    if latest.is_object()
        && (latest["repairRequired"].as_bool() != Some(true)
            || latest["mandatory"].as_bool() != Some(true)
            || latest["updateAvailable"].as_bool() != Some(true))
    {
        shroudforge_loader_config::config::update_loader(root, |config| {
            let latest = &mut config["modules"]["updates"]["system"]["latestRelease"];
            if latest.is_object() {
                latest["repairRequired"] = serde_json::Value::Bool(true);
                latest["mandatory"] = serde_json::Value::Bool(true);
                latest["updateAvailable"] = serde_json::Value::Bool(true);
            }
            Ok(())
        })?;
    }
    Ok(true)
}
