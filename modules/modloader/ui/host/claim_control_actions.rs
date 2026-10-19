use super::mark_control_action::mark_control_action;
use super::parse_ui_command::UiCommand;
use std::path::Path;

pub(crate) fn claim_control_actions(
    root: &Path,
    recover_interrupted: bool,
) -> Result<Vec<UiCommand>, String> {
    let actions = shroudforge_loader_config::config::claim_control_actions(
        root,
        shroudforge_loader_config::config::ControlActionOwner::Modloader,
        recover_interrupted,
    )?;
    let mut commands = Vec::new();
    for action in actions {
        let Some(command) = control_action_command(&action.name) else {
            mark_control_action(
                root,
                &action.request_id,
                "failed",
                "Unknown predefined control action",
            );
            continue;
        };
        let mut payload = action.parameters.as_object().cloned().unwrap_or_default();
        payload.insert("command".into(), command.into());
        payload.insert("requestId".into(), action.request_id.clone().into());
        match serde_json::from_value(serde_json::Value::Object(payload)) {
            Ok(command) => commands.push(command),
            Err(error) => mark_control_action(
                root,
                &action.request_id,
                "failed",
                &format!("Invalid control action: {error}"),
            ),
        }
    }
    Ok(commands)
}

pub(crate) fn control_action_command(action: &str) -> Option<&'static str> {
    Some(match action {
        "diagnostics" => "diagnostics",
        "refresh" => "refresh",
        "refreshMod" => "refresh-mod",
        "reloadSettings" => "reload-settings",
        "runModAction" => "run-mod-action",
        "removeMod" => "remove-mod",
        "setNewsRead" => "set-news-read",
        _ => return None,
    })
}
