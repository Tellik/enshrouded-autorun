#[cfg(windows)]
use crate::scheduled;
#[cfg(windows)]
use crate::update_queue_storage::update_update_queue;

#[cfg(windows)]
pub(crate) fn finish_cancelled_queue(
    root: &std::path::Path,
    ids: &[String],
) -> Result<(), String> {
    let mut cleanup_errors = Vec::new();
    let mut transitions = Vec::new();
    let update_result = update_update_queue(root, |queue| {
        let items = queue["items"]
            .as_array_mut()
            .ok_or("update queue has an invalid items list")?;
        for id in ids {
            let Some(item) = items
                .iter_mut()
                .find(|item| item["id"].as_str() == Some(id))
            else {
                cleanup_errors.push(format!(
                    "could not mark queue item {id} cancelled: update queue item '{id}' disappeared"
                ));
                continue;
            };
            let previous_state = item["state"].as_str().unwrap_or("unknown").to_owned();
            if matches!(previous_state.as_str(), "complete" | "error") {
                continue;
            }
            if previous_state != "cancelled" {
                transitions.push((id.clone(), previous_state));
            }
            item["state"] = serde_json::Value::String("cancelled".into());
            item["message"] = serde_json::Value::String("Cancelled by user".into());
        }
        Ok(())
    });
    match update_result {
        Ok(()) => {
            for (id, previous_state) in transitions {
                let _ = shroudforge_logging::append(
                    root,
                    'I',
                    "updater",
                    &format!("Update queue item {id}: {previous_state} -> cancelled"),
                );
            }
        }
        Err(error) => cleanup_errors.push(format!(
            "could not mark cancelled queue items: {error}"
        )),
    }
    let updates = shroudforge_loader_config::paths::updates_dir(root);
    remove_file_if_present(&updates.join("download.zip"), &mut cleanup_errors);
    remove_directory_if_present(&updates.join("pending-download"), &mut cleanup_errors);
    remove_directory_if_present(&updates.join("pending"), &mut cleanup_errors);
    remove_file_if_present(&updates.join("pending.ready"), &mut cleanup_errors);
    remove_file_if_present(&updates.join("cancel-system-update"), &mut cleanup_errors);
    let warning = (!cleanup_errors.is_empty()).then(|| cleanup_errors.join("; "));
    scheduled::write_status(root, "cancelled", "Selected update queue cancelled");
    update_update_queue(root, |queue| {
        queue["run"]["state"] = serde_json::Value::String("cancelled".into());
        queue["run"]["error"] = warning
            .clone()
            .map(serde_json::Value::String)
            .unwrap_or(serde_json::Value::Null);
        queue["run"]["activeItemId"] = serde_json::Value::Null;
        queue["run"]["installing"] = serde_json::Value::Bool(false);
        queue["run"]["finishedAt"] = serde_json::json!(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
        Ok(())
    })?;
    shroudforge_logging::write_activity(
        root,
        "Updates",
        "Cancel update queue",
        if warning.is_some() {
            "Cancelled with cleanup warnings"
        } else {
            "Cancelled"
        },
        warning
            .as_deref()
            .or(Some("Selected update queue cancelled")),
        if warning.is_some() { "warn" } else { "info" },
    );
    Ok(())
}

#[cfg(windows)]
fn remove_file_if_present(path: &std::path::Path, errors: &mut Vec<String>) {
    if let Err(error) = std::fs::remove_file(path)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        errors.push(format!("could not remove {}: {error}", path.display()));
    }
}

#[cfg(windows)]
fn remove_directory_if_present(path: &std::path::Path, errors: &mut Vec<String>) {
    if let Err(error) = std::fs::remove_dir_all(path)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        errors.push(format!("could not remove {}: {error}", path.display()));
    }
}
