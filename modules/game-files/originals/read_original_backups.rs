use super::{manifest, originals_dir, originals_target_dir, pair, stem, target_name};
use serde_json::{Value, json};
use std::{fs, path::Path};

pub fn originals_status(root: &Path, target: &str) -> Value {
    let Ok(stem) = stem(target) else {
        return json!({"target":target,"status":"unsupported","files":[]});
    };
    let dir = originals_dir(root, stem);
    if !dir.exists() {
        let files = pair(stem);
        let ambiguous = files
            .iter()
            .any(|name| root.join(format!("{name}.bak")).exists())
            || crate::config::read_document(root, crate::documents::APPLIED)
                .ok()
                .is_some_and(|value| {
                    matches!(
                        value["status"].as_str(),
                        Some("applied" | "preparing" | "failed")
                    )
                });
        let old_builds = fs::read_dir(originals_target_dir(root, stem))
            .ok()
            .is_some_and(|mut entries| entries.next().is_some());
        return json!({"target":target_name(stem),"status":if ambiguous || old_builds {"verification-required"} else {"missing"},"files":[]});
    }
    let value = match manifest(root, stem) {
        Ok(value) => value,
        Err(error) => {
            return json!({"target":target_name(stem),"status":"invalid","detail":error,"files":[]});
        }
    };
    let Some(entries) = value["files"].as_array() else {
        return json!({"target":target_name(stem),"status":"invalid","detail":"manifest has no file list","files":[]});
    };
    let mut valid = entries.len() == 2;
    let mut found = std::collections::HashSet::new();
    let files: Vec<Value> = entries
        .iter()
        .map(|entry| {
            let name = entry["name"].as_str().unwrap_or_default();
            if !pair(stem).iter().any(|expected| expected == name) {
                valid = false;
                return json!({"name":name,"valid":false});
            }
            if !found.insert(name.to_owned()) {
                valid = false;
                return json!({"name":name,"valid":false});
            }
            let path = dir.join(name);
            let actual_size = fs::metadata(&path).ok().map(|metadata| metadata.len());
            let expected = entry["sha256"].as_str().unwrap_or_default();
            let expected_size = entry["size"].as_u64();
            // Full checksums are verified in the updater before restoration. Avoid
            // hashing potentially large resources files during every UI refresh.
            let ok = actual_size.is_some() && actual_size == expected_size && expected.len() == 64;
            valid &= ok;
            json!({"name":name,"size":entry["size"],"valid":ok})
        })
        .collect();
    valid &= pair(stem).iter().all(|name| found.contains(name));
    json!({"target":target_name(stem),"status":if valid {"ready"} else {"invalid"},"capturedAt":value["capturedAt"],"source":value["source"],"gameExecutable":value["gameExecutable"],"files":files})
}
