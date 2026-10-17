use crate::originals::{originals_status, stem, target_name};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// One read-only inventory for the UI. Working `.bak` files and short-lived
/// transactions are listed separately from immutable installation originals.
pub fn management_status(root: &Path, target: &str) -> Value {
    let Ok(stem) = stem(target) else {
        return json!({"target":target,"originals":{"status":"unsupported"}});
    };
    let kfc_backup = root.join(format!("{stem}.kfc.bak"));
    let resources_backup = root.join(format!("{stem}.kfc_resources.bak"));
    let transaction = crate::transaction::directory(root, stem);
    let restore_status_path = crate::paths::backups_dir(root).join("gamefiles-restore-status.json");
    let restore = fs::read(&restore_status_path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .unwrap_or_else(|| json!({"status":"idle"}));
    let count_dirs = |path: PathBuf| -> usize {
        fs::read_dir(path)
            .ok()
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|entry| entry.path().is_dir())
                    .count()
            })
            .unwrap_or(0)
    };
    json!({
        "target":target_name(stem),
        "originals":originals_status(root, stem),
        "workingBaseline":{"kfc":kfc_backup.is_file(),"resources":resources_backup.is_file(),"complete":kfc_backup.is_file()&&resources_backup.is_file()},
        "transaction":{"pending":transaction.join("pending.json").is_file()},
        "rollback":{"systemUpdates":count_dirs(crate::paths::updates_dir(root).join("backups")),"modPackages":count_dirs(crate::paths::updates_dir(root).join("mod-backups"))},
        "restore":restore
    })
}
