use std::path::Path;

pub(crate) fn save_language(root: &Path, locale: &str) -> Result<(), String> {
    if !is_valid_locale(locale) {
        return Err("invalid locale".into());
    }
    shroudforge_loader_config::config::update_loader(root, |value| {
        let object = value
            .as_object_mut()
            .ok_or("central config must be an object")?;
        let general = object
            .entry("general")
            .or_insert_with(|| serde_json::json!({}));
        let general = general
            .as_object_mut()
            .ok_or("general config must be an object")?;
        general.insert(
            "locale".into(),
            serde_json::Value::String(locale.to_owned()),
        );
        Ok(())
    })
}

pub(crate) fn is_valid_locale(locale: &str) -> bool {
    (2..=35).contains(&locale.len())
        && !locale.starts_with('-')
        && !locale.ends_with('-')
        && !locale.contains("--")
        && locale
            .bytes()
            .all(|character| character.is_ascii_alphanumeric() || character == b'-')
}
