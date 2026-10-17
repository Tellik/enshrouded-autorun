use super::claim_control_actions::claim_control_actions;
use super::mark_control_action::mark_control_action;
use super::parse_ui_command::{Command, parse_ui_command};
use super::read_host_arguments::Arguments;
use shroudforge_logging::write_activity;
use std::sync::mpsc;

pub(crate) fn dispatch_control_actions(
    arguments: &Arguments,
    control_sender: &mpsc::Sender<Command>,
    recovered_control_actions: &mut bool,
) {
    match claim_control_actions(&arguments.root, !*recovered_control_actions) {
        Ok(actions) => {
            *recovered_control_actions = true;
            for action in actions {
                let request_id = action.request_id.clone().unwrap_or_default();
                if arguments.server
                    && (matches!(
                        action.command.as_str(),
                        "drag" | "browse-path" | "open-path" | "open-url" | "ui-ready" | "ui-error"
                    ) || (action.command == "set-window-visibility"
                        && action.visible == Some(true)))
                {
                    mark_control_action(
                        &arguments.root,
                        &request_id,
                        "failed",
                        "This command requires a visible desktop. Use save-setting with an explicit path for storage paths.",
                    );
                    continue;
                }
                match parse_ui_command(
                    action,
                    arguments.desktop,
                    arguments.updater_window,
                    arguments.server,
                ) {
                    Some(command) => {
                        if control_sender.send(command).is_err() {
                            write_activity(
                                &arguments.root,
                                "Modloader UI",
                                "Dispatch command",
                                "Failed",
                                Some("Modloader command dispatcher is unavailable"),
                                "error",
                            );
                            mark_control_action(
                                &arguments.root,
                                &request_id,
                                "failed",
                                "Modloader command dispatcher is unavailable",
                            );
                        }
                    }
                    None => {
                        write_activity(
                            &arguments.root,
                            "Modloader UI",
                            "Dispatch command",
                            "Failed",
                            Some("Invalid or unsupported Modloader UI command payload"),
                            "error",
                        );
                        mark_control_action(
                            &arguments.root,
                            &request_id,
                            "failed",
                            "Invalid or unsupported Modloader UI command payload",
                        );
                    }
                }
            }
        }
        Err(error) => {
            *recovered_control_actions = true;
            let _ = shroudforge_logging::append(
                &arguments.root,
                'E',
                "modloader-config",
                &format!("Could not read control actions: {error}"),
            );
        }
    }
}
