use super::parse_ui_command::UiCommand;
use std::path::Path;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub(crate) fn claim_control_actions(
    root: &Path,
    recover_interrupted: bool,
) -> Result<Vec<UiCommand>, String> {
    let current = shroudforge_loader_config::config::read_loader(root)?;
    let has_pending = current
        .pointer("/control/actions")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|actions| {
            actions
                .values()
                .any(|action| action.as_bool() == Some(true))
        });
    if !recover_interrupted && !has_pending {
        return Ok(Vec::new());
    }

    let mut claimed = Vec::new();
    shroudforge_loader_config::config::update_loader(root, |config| {
        if recover_interrupted {
            let interrupted = config
                .pointer("/control/results")
                .and_then(serde_json::Value::as_object)
                .into_iter()
                .flat_map(|results| results.iter())
                .filter_map(|(name, result)| (result["state"] == "running").then(|| name.clone()))
                .collect::<Vec<_>>();
            for name in interrupted {
                config["control"]["actions"][&name] = serde_json::Value::Bool(false);
                config["control"]["results"][&name]["state"] =
                    serde_json::Value::String("failed".into());
                config["control"]["results"][&name]["result"] =
                    serde_json::Value::String("The command process stopped while this action was running. Inspect the update queue and log before retrying.".into());
            }
        }

        let names = config["control"]["actions"]
            .as_object()
            .ok_or("control.actions must be an object")?
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        for name in names {
            if config["control"]["actions"][&name].as_bool() != Some(true) {
                continue;
            }
            let Some(command_name) = control_action_command(&name) else {
                config["control"]["actions"][&name] = serde_json::Value::Bool(false);
                config["control"]["results"][&name]["state"] =
                    serde_json::Value::String("failed".into());
                config["control"]["results"][&name]["result"] =
                    serde_json::Value::String("Unknown predefined control action".into());
                continue;
            };

            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let request_id = format!("config-{name}-{}-{nonce}", std::process::id());
            let mut payload = config["control"]["parameters"][&name]
                .as_object()
                .cloned()
                .unwrap_or_default();
            payload.insert(
                "command".into(),
                serde_json::Value::String(command_name.into()),
            );
            payload.insert(
                "requestId".into(),
                serde_json::Value::String(request_id.clone()),
            );
            let command: UiCommand =
                match serde_json::from_value(serde_json::Value::Object(payload)) {
                    Ok(command) => command,
                    Err(error) => {
                        config["control"]["actions"][&name] = serde_json::Value::Bool(false);
                        config["control"]["results"][&name]["requestId"] =
                            serde_json::Value::String(request_id);
                        config["control"]["results"][&name]["state"] =
                            serde_json::Value::String("failed".into());
                        config["control"]["results"][&name]["result"] =
                            serde_json::Value::String(format!("Invalid control action: {error}"));
                        continue;
                    }
                };

            // Reset the one-shot trigger as it is claimed. The result record
            // remains visible in the same config while execution continues.
            config["control"]["actions"][&name] = serde_json::Value::Bool(false);
            config["control"]["results"][&name]["requestId"] =
                serde_json::Value::String(request_id);
            config["control"]["results"][&name]["state"] =
                serde_json::Value::String("running".into());
            config["control"]["results"][&name]["result"] =
                serde_json::Value::String("Accepted by Modloader command dispatcher".into());
            claimed.push(command);
        }
        Ok(())
    })?;
    Ok(claimed)
}

pub(crate) fn control_action_command(action: &str) -> Option<&'static str> {
    Some(match action {
        "diagnostics" => "diagnostics",
        "refresh" => "refresh",
        "refreshMod" => "refresh-mod",
        "reloadSettings" => "reload-settings",
        "checkUpdates" => "check-updates",
        "stageUpdate" => "stage-update",
        "queueSystemUpdate" => "queue-system-update",
        "selectUpdateQueueItems" => "select-update-queue-items",
        "removeUpdateQueueItem" => "remove-update-queue-item",
        "startUpdateQueue" => "start-update-queue",
        "clearUpdateQueue" => "clear-update-queue",
        "cancelUpdate" => "cancel-update",
        "restoreGamefiles" => "restore-gamefiles",
        "captureGamefileOriginals" => "capture-gamefile-originals",
        "searchCatalog" => "search-catalog",
        "installMod" => "install-mod",
        "updateMod" => "update-mod",
        "runModAction" => "run-mod-action",
        "removeMod" => "remove-mod",
        "setNewsRead" => "set-news-read",
        _ => return None,
    })
}
