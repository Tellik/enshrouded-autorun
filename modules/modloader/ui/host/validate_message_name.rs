pub(crate) fn default_message_name_fallback() -> String {
    "Flameborn".into()
}

pub(crate) fn default_username_mode() -> String {
    "auto".into()
}

pub(crate) fn normalize_message_name(value: &str) -> Result<String, String> {
    let name = value.trim();
    if name.is_empty() {
        return Err("message name must not be empty".into());
    }
    if name.chars().count() > 40 || name.chars().any(char::is_control) {
        return Err(
            "message name must be at most 40 characters and contain no control characters".into(),
        );
    }
    Ok(name.to_owned())
}
