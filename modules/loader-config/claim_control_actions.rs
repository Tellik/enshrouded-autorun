use std::{path::Path, time::SystemTime, time::UNIX_EPOCH};

#[derive(Clone, Copy)]
pub enum ControlActionOwner {
    Modloader,
    Updater,
}

impl ControlActionOwner {
    fn owns(self, name: &str) -> bool {
        let is_updater = crate::UPDATER_CONTROL_ACTIONS.contains(&name);
        match self {
            Self::Modloader => !is_updater,
            Self::Updater => is_updater,
        }
    }

    fn request_prefix(self) -> &'static str {
        match self {
            Self::Modloader => "config",
            Self::Updater => "headless",
        }
    }

    fn accepted_message(self) -> &'static str {
        match self {
            Self::Modloader => "Accepted by Modloader command dispatcher",
            Self::Updater => "Accepted by headless update controller",
        }
    }
}

pub struct ClaimedControlAction {
    pub name: String,
    pub request_id: String,
    pub parameters: serde_json::Value,
}

pub fn claim_control_actions(
    root: &Path,
    owner: ControlActionOwner,
    recover_interrupted: bool,
) -> Result<Vec<ClaimedControlAction>, String> {
    let current = crate::config::read_loader(root)?;
    let has_pending = current
        .pointer("/control/actions")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|actions| {
            actions
                .iter()
                .any(|(name, value)| owner.owns(name) && value.as_bool() == Some(true))
        });
    if !recover_interrupted && !has_pending {
        return Ok(Vec::new());
    }

    let mut claimed = Vec::new();
    crate::config::update_loader(root, |config| {
        if recover_interrupted {
            let interrupted = config
                .pointer("/control/results")
                .and_then(serde_json::Value::as_object)
                .into_iter()
                .flat_map(|results| results.iter())
                .filter_map(|(name, result)| {
                    (owner.owns(name) && result["state"] == "running").then(|| name.clone())
                })
                .collect::<Vec<_>>();
            for name in interrupted {
                config["control"]["actions"][&name] = serde_json::Value::Bool(false);
                config["control"]["results"][&name]["state"] =
                    serde_json::Value::String("failed".into());
                config["control"]["results"][&name]["result"] = serde_json::Value::String(
                    "The command process stopped while this action was running. Inspect the update queue and log before retrying.".into(),
                );
            }
        }

        let names = config["control"]["actions"]
            .as_object()
            .ok_or("control.actions must be an object")?
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        for name in names {
            if !owner.owns(&name) || config["control"]["actions"][&name].as_bool() != Some(true) {
                continue;
            }
            let request_id = format!(
                "{}-{name}-{}-{}",
                owner.request_prefix(),
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            );
            let parameters = config
                .pointer(&format!("/control/parameters/{name}"))
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            config["control"]["actions"][&name] = serde_json::Value::Bool(false);
            config["control"]["results"][&name] = serde_json::json!({
                "requestId": request_id,
                "state": "running",
                "result": owner.accepted_message()
            });
            claimed.push(ClaimedControlAction {
                name,
                request_id,
                parameters,
            });
        }
        Ok(())
    })?;
    Ok(claimed)
}
