use shroudforge_package::package_lookup::valid_identifier;
use std::fs;
use std::path::Path;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub(crate) fn dispatch_action(
    root: &Path,
    action: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !valid_identifier(action) {
        return Err("Invalid mod action".into());
    }
    let directory = shroudforge_loader_config::paths::ui_data_dir(root).join("actions");
    fs::create_dir_all(&directory)?;
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = directory.join(format!("world-editor-{}-{nonce}.json", std::process::id()));
    let payload = serde_json::json!({ "modId": "world-editor", "action": action, "value": value });
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, serde_json::to_vec(&payload)?)?;
    fs::rename(temporary, path)?;
    Ok(())
}
