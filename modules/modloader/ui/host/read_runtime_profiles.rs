use std::fs;
use std::path::Path;

pub(crate) fn runtime_profile_ids(root: &Path) -> Vec<String> {
    let directory = shroudforge_loader_config::paths::runtime_profiles_dir(root);
    let mut profiles = Vec::new();
    let mut pending = vec![directory];
    while let Some(current) = pending.pop() {
        let Ok(entries) = fs::read_dir(current) else {
            continue;
        };
        for entry in entries.flatten() {
            if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                pending.push(entry.path());
            } else if entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                if let Ok(bytes) = fs::read(entry.path()) {
                    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                        if let Some(id) = value.get("id").and_then(serde_json::Value::as_str) {
                            profiles.push(id.to_owned());
                        }
                    }
                }
            }
        }
    }
    profiles.sort();
    profiles.dedup();
    profiles
}

pub(crate) fn active_runtime_profile(root: &Path) -> Option<String> {
    let status = shroudforge_loader_config::config::read_document(root, shroudforge_runtime_diagnostics::documents::MOD_STATUS).ok()?;
    let detail = status.pointer("/runtimeProvider/detail")?.as_str()?;
    let profile = detail.split("profile=").nth(1)?.split_whitespace().next()?;
    Some(profile.to_owned())
}
