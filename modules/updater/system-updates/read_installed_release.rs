use crate::release::InstalledRelease;
use std::fs;
use std::path::Path;

pub fn installed_release(root: &Path) -> InstalledRelease {
    let value = fs::read(shroudforge_loader_config::paths::version_file(root))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
    let version = value
        .as_ref()
        .and_then(|value| value.get("version"))
        .and_then(|value| value.as_str())
        .map(str::to_owned)
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").into());
    let build = value
        .as_ref()
        .and_then(|value| value.get("build"))
        .and_then(|value| {
            value
                .as_u64()
                .or_else(|| value.as_str().and_then(|value| value.parse().ok()))
        })
        .unwrap_or(0);
    InstalledRelease { version, build }
}

pub fn installed_version(root: &Path) -> String {
    installed_release(root).version
}
