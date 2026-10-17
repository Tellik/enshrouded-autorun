pub(crate) enum Command {
    Hide,
    Drag,
    Preferences(serde_json::Value),
}

pub(crate) fn parse_console_message(body: &str) -> Option<Command> {
    let command = match body {
        "hide" => Command::Hide,
        "drag" => Command::Drag,
        _ => match serde_json::from_str(body) {
            Ok(value) => Command::Preferences(value),
            Err(_) => return None,
        },
    };
    Some(command)
}
