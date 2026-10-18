use crate::check_update_cancelled::check_update_cancelled;
use crate::write_update_status::{write_download_progress, write_status};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};
const MAX_UPDATE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_ARCHIVE_ENTRIES: usize = 4096;

pub(crate) fn stage_system_update(root: &Path, request: &serde_json::Value) -> Result<(), String> {
    if !crate::update_check::system_updates_enabled(root) {
        return Err("System updates module is disabled".into());
    }
    let version = request["version"]
        .as_str()
        .ok_or("stage request has no version")?;
    let url = request["downloadUrl"]
        .as_str()
        .ok_or("stage request has no download URL")?;
    let checksum = request["checksum"]
        .as_str()
        .ok_or("stage request has no checksum")?
        .to_ascii_lowercase();
    let wait_pid = request["waitPid"].as_u64().unwrap_or(0) as u32;
    if !url.starts_with("https://")
        || checksum.len() != 64
        || !checksum.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("stage request URL or SHA-256 checksum is invalid".into());
    }
    let updates = shroudforge_loader_config::paths::updates_dir(root);
    check_update_cancelled(root)?;
    fs::create_dir_all(&updates).map_err(|e| e.to_string())?;
    let download = updates.join("download.zip");
    let extraction = updates.join("pending-download");
    let pending = updates.join("pending");
    let _ = fs::remove_dir_all(&extraction);
    fs::create_dir_all(&extraction).map_err(|e| e.to_string())?;
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .user_agent(format!("ShroudForge/{version} updater"))
        .build()
        .map_err(|e| e.to_string())?;
    let mut response = client
        .get(url)
        .send()
        .map_err(|e| format!("download failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "release download returned HTTP {}",
            response.status()
        ));
    }
    let content_length = response.content_length();
    if content_length.is_some_and(|length| length > MAX_UPDATE_BYTES) {
        return Err("update package exceeds 2 GiB".into());
    }
    let mut file = fs::File::create(&download).map_err(|e| e.to_string())?;
    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    let mut total = 0u64;
    let mut last_progress = std::time::Instant::now();
    let mut last_bytes = 0u64;
    write_download_progress(root, version, 0, content_length, Some(0));
    // Keep the 1 MiB transfer buffer off the Windows worker's limited stack.
    let mut buffer = vec![0u8; 1024 * 1024];
    loop {
        check_update_cancelled(root)?;
        let count = response.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        total = total.saturating_add(count as u64);
        if total > MAX_UPDATE_BYTES {
            return Err("update package exceeds 2 GiB".into());
        }
        file.write_all(&buffer[..count])
            .map_err(|e| e.to_string())?;
        hasher.update(&buffer[..count]);
        if last_progress.elapsed() >= std::time::Duration::from_millis(300) {
            let elapsed = last_progress.elapsed().as_secs_f64().max(0.001);
            let speed = ((total - last_bytes) as f64 / elapsed) as u64;
            write_download_progress(root, version, total, content_length, Some(speed));
            last_progress = std::time::Instant::now();
            last_bytes = total;
        }
    }
    file.flush().map_err(|e| e.to_string())?;
    write_download_progress(root, version, total, content_length, Some(0));
    write_status(
        root,
        "verifying",
        "Verifying the downloaded package checksum",
    );
    if format!("{:x}", hasher.finalize()) != checksum {
        return Err("update package SHA-256 checksum does not match".into());
    }
    write_status(root, "extracting", "Extracting and validating update files");
    let source = fs::File::open(&download).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(source).map_err(|e| e.to_string())?;
    if archive.len() > MAX_ARCHIVE_ENTRIES {
        return Err("update archive contains too many entries".into());
    }
    let mut expanded = 0u64;
    for index in 0..archive.len() {
        check_update_cancelled(root)?;
        let mut entry = archive.by_index(index).map_err(|e| e.to_string())?;
        let relative = entry
            .enclosed_name()
            .ok_or_else(|| format!("unsafe archive path: {}", entry.name()))?
            .to_path_buf();
        if relative
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
        {
            return Err(format!("unsafe archive path: {}", entry.name()));
        }
        expanded = expanded.saturating_add(entry.size());
        if expanded > 8 * MAX_UPDATE_BYTES {
            return Err("expanded update archive exceeds 16 GiB".into());
        }
        let output = extraction.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&output).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut file = fs::File::create(&output).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut file).map_err(|e| e.to_string())?;
            file.flush().map_err(|e| e.to_string())?;
        }
    }
    for required in [
        "shroudforge/version.json",
        "shroudforge/shroudforge.exe",
        "shroudforge/shroudforge-updater.exe",
    ] {
        if !extraction.join(required).is_file() {
            return Err(format!("update package is missing {required}"));
        }
    }
    let _ = fs::remove_file(updates.join("pending.ready"));
    if pending.exists() {
        fs::remove_dir_all(&pending).map_err(|e| e.to_string())?;
    }
    fs::rename(&extraction, &pending)
        .map_err(|e| format!("could not promote verified update: {e}"))?;
    let ready = serde_json::json!({"version":version,"checksumAlgorithm":"SHA-256","checksum":checksum,"stagedAt":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()});
    shroudforge_loader_config::config::write_json(&updates.join("pending.ready"), &ready)?;
    let _ = fs::remove_file(download);
    let queue =
        serde_json::json!({"schemaVersion":1,"operation":"installPending","waitPid":wait_pid});
    shroudforge_loader_config::config::write_json(&updates.join("worker-queue.json"), &queue)?;
    Ok(())
}
