#[cfg(windows)]
use crate::finish_cancelled_queue::finish_cancelled_queue;
#[cfg(windows)]
use crate::scheduled;
#[cfg(windows)]
use crate::set_update_item_state::set_update_item_state;
#[cfg(windows)]
use crate::update_queue_storage::read_update_queue_value;
#[cfg(windows)]
use crate::update_queue_storage::update_update_queue;

#[cfg(windows)]
pub(crate) fn run_update_queue(root: &std::path::Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let mut queue = read_update_queue_value(root);
    let run = queue["run"].clone();
    let ids = run["itemIds"]
        .as_array()
        .ok_or("update queue run has no item list")?
        .iter()
        .filter_map(|id| id.as_str().map(str::to_owned))
        .collect::<Vec<_>>();
    let wait_for_game = run["waitForGame"].as_bool().unwrap_or(false);
    let show_window = run["showWindow"].as_bool().unwrap_or(true);
    if wait_for_game {
        scheduled::write_status(
            root,
            "waitingForGame",
            "Waiting for Enshrouded to close before starting selected downloads",
        );
        if let Err(error) = scheduled::wait_for_game_processes(root) {
            if error == "UPDATE_CANCELLED" {
                finish_cancelled_queue(root, &ids);
                return Ok(());
            }
            return Err(error);
        }
    }
    if show_window {
        scheduled::open_update_window(root)?;
    }

    let mut failures = Vec::new();
    for id in &ids {
        if scheduled::check_update_cancelled(root).is_err() {
            finish_cancelled_queue(root, &ids);
            return Ok(());
        }
        queue = read_update_queue_value(root);
        let Some(item) = queue["items"]
            .as_array()
            .and_then(|items| items.iter().find(|item| item["id"].as_str() == Some(id)))
            .cloned()
        else {
            failures.push(format!("queue item '{id}' disappeared"));
            continue;
        };
        if item["state"] == "cancelled" {
            continue;
        }
        set_update_item_state(root, id, "downloading", None)?;
        let result = match item["kind"].as_str() {
            Some("system") => {
                let request = item["payload"].clone();
                let version = request["version"].as_str().unwrap_or_default();
                scheduled::write_status_with_version(
                    root,
                    "downloading",
                    "Downloading selected ShroudForge update",
                    Some(version),
                );
                scheduled::stage_system_update(root, &request).and_then(|()| {
                    if scheduled::check_update_cancelled(root).is_err() {
                        return Err("UPDATE_CANCELLED".into());
                    }
                    set_update_item_state(
                        root,
                        id,
                        "waitingForGame",
                        Some("Download verified. Installation waits for Enshrouded to close"),
                    )?;
                    scheduled::write_status(
                        root,
                        "waitingForGame",
                        "Download verified. Waiting to install ShroudForge",
                    );
                    let staged =
                        shroudforge_loader_config::paths::updates_dir(root).join("pending");
                    let status = std::process::Command::new(
                        std::env::current_exe().map_err(|error| error.to_string())?,
                    )
                    .args(["--update-worker", "--root"])
                    .arg(root)
                    .arg("--staged")
                    .arg(staged)
                    .status()
                    .map_err(|error| format!("could not launch validated installer: {error}"))?;
                    let _ = std::fs::remove_file(
                        shroudforge_loader_config::paths::updates_dir(root)
                            .join("worker-queue.json"),
                    );
                    if status.success() {
                        Ok(())
                    } else if scheduled::check_update_cancelled(root).is_err() {
                        Err("UPDATE_CANCELLED".into())
                    } else {
                        Err(format!("installer returned {status}"))
                    }
                })
            }
            Some("mod") => {
                let payload = &item["payload"];
                let request = serde_json::json!({
                    "schemaVersion":1,
                    "queueItemId":id,
                    "operation":item["operation"],
                    "projectId":payload["projectId"],
                    "modId":payload["modId"],
                    "provider":payload["provider"]
                });
                let request_path = shroudforge_loader_config::paths::updates_dir(root)
                    .join("mod-install-queue.json");
                shroudforge_loader_config::config::write_json(&request_path, &request)
                    .map_err(|error| error.to_string())?;
                let status = std::process::Command::new(
                    shroudforge_loader_config::paths::loader_executable(root),
                )
                .args(["--catalog-install-worker", "--root"])
                .arg(root)
                .creation_flags(0x08000000)
                .status()
                .map_err(|error| format!("could not launch catalog installer: {error}"))?;
                if status.success() {
                    Ok(())
                } else if scheduled::check_update_cancelled(root).is_err() {
                    Err("UPDATE_CANCELLED".into())
                } else {
                    Err(format!("catalog installer returned {status}"))
                }
            }
            _ => Err(format!("unsupported update queue item kind for '{id}'")),
        };
        match result {
            Ok(()) => set_update_item_state(root, id, "complete", Some("Update completed"))?,
            Err(error) if error == "UPDATE_CANCELLED" => {
                finish_cancelled_queue(root, &ids);
                return Ok(());
            }
            Err(error) => {
                set_update_item_state(root, id, "error", Some(&error))?;
                failures.push(format!("{id}: {error}"));
            }
        }
    }
    update_update_queue(root, |queue| {
        queue["run"]["state"] = serde_json::Value::String("complete".into());
        queue["run"]["finishedAt"] = serde_json::json!(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
        Ok(())
    })?;
    let _ = std::fs::remove_file(
        shroudforge_loader_config::paths::updates_dir(root).join("cancel-system-update"),
    );
    let contains_system_update = ids.iter().any(|id| id.starts_with("system:"));
    if failures.is_empty() {
        if contains_system_update {
            scheduled::write_status(root, "installed", "Selected update queue completed");
        } else {
            scheduled::write_status(root, "ready", "Selected mod queue completed");
        }
        Ok(())
    } else {
        if contains_system_update {
            scheduled::write_status(root, "error", &failures.join(", "));
        } else {
            scheduled::write_status(root, "ready", "One or more mod queue items failed");
        }
        Err(failures.join(", "))
    }
}
