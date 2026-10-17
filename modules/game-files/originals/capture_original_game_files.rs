use super::{game_executable, originals_dir, originals_status, pair, sha256, stem, target_name};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

/// Captures the files only when no legacy working backup or prior application
/// indicates they may already have been modified. Existing originals are immutable.
pub fn ensure_originals(root: &Path, target: &str) -> Result<Value, String> {
    let stem = stem(target)?;
    let status = originals_status(root, stem);
    if status["status"] != "missing" {
        return Ok(status);
    }
    capture_originals_inner(root, stem, false)?;
    Ok(originals_status(root, stem))
}

/// Explicit capture is intended for users who have verified the game files.
/// It still refuses to replace an existing originals directory.
pub fn capture_verified_originals(root: &Path, target: &str) -> Result<Value, String> {
    let stem = stem(target)?;
    if originals_dir(root, stem).exists() {
        return Err("an original-backup set already exists and will not be overwritten".into());
    }
    capture_originals_inner(root, stem, true)?;
    Ok(originals_status(root, stem))
}

fn capture_originals_inner(root: &Path, stem: &str, user_verified: bool) -> Result<(), String> {
    let status = originals_status(root, stem);
    if !user_verified && status["status"] != "missing" {
        return Err("cannot automatically mark an uncertain game-file set as original".into());
    }
    if !user_verified {
        let legacy_backup = pair(stem)
            .iter()
            .any(|name| root.join(format!("{name}.bak")).exists());
        let previous_apply = crate::config::read_document(root, crate::documents::APPLIED)
            .ok()
            .is_some_and(|value| {
                matches!(
                    value["status"].as_str(),
                    Some("applied" | "preparing" | "failed")
                )
            });
        if legacy_backup || previous_apply {
            return Err("existing mod or backup state makes the clean origin uncertain".into());
        }
    }
    let files = pair(stem);
    for name in &files {
        if !root.join(name).is_file() {
            return Err(format!("required game file is missing: {name}"));
        }
    }
    let exe_name = if stem == "enshrouded_server" {
        "enshrouded_server.exe"
    } else {
        "enshrouded.exe"
    };
    let exe = game_executable(root, stem);
    let exe_meta = fs::metadata(&exe).map_err(|error| format!("{}: {error}", exe.display()))?;
    let exe_modified = exe_meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|time| time.as_secs());
    let final_dir = originals_dir(root, stem);
    let parent = final_dir
        .parent()
        .ok_or("invalid originals directory")?
        .to_path_buf();
    fs::create_dir_all(&parent).map_err(|error| error.to_string())?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let stage = parent.join(format!(".capture-{}-{nonce}", std::process::id()));
    fs::create_dir(&stage).map_err(|error| error.to_string())?;
    let result = (|| {
        let mut entries = Vec::new();
        for name in &files {
            let source = root.join(name);
            let target = stage.join(name);
            fs::copy(&source, &target).map_err(|error| error.to_string())?;
            let copied_hash = sha256(&target)?;
            if copied_hash != sha256(&source)? {
                return Err(format!(
                    "game file changed while its original was being captured: {name}"
                ));
            }
            let size = fs::metadata(&target)
                .map_err(|error| error.to_string())?
                .len();
            entries.push(json!({"name":name,"size":size,"sha256":copied_hash}));
        }
        let game_executable = json!({"name":exe_name,"size":exe_meta.len(),"modified":exe_modified,"sha256":sha256(&exe)?});
        let metadata = json!({
            "schemaVersion":1,
            "target":target_name(stem),
            "source":if user_verified {"user-verified"} else {"first-managed-start"},
            "capturedAt":SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            "gameExecutable":game_executable,
            "files":entries
        });
        crate::config::write_json(&stage.join("manifest.json"), &metadata)?;
        match fs::rename(&stage, &final_dir) {
            Ok(()) => Ok(()),
            Err(_error) if final_dir.exists() => Err(
                "Original backup set appeared concurrently. Existing originals were preserved"
                    .into(),
            ),
            Err(error) => Err(error.to_string()),
        }
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&stage);
    }
    result
}
