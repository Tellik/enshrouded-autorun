use super::loader_settings_input::UiSettings;
use super::validate_message_name::normalize_message_name;
use shroudforge_updater::provider_config::ProviderConfig;
use shroudforge_updater::provider_config::default_loader;
use shroudforge_updater::provider_config::default_provider_kind;
use std::path::Path;

pub(crate) fn save_settings(root: &Path, settings: &UiSettings) -> Result<(), String> {
    let provider = ProviderConfig {
        enabled: settings.update_enabled,
        kind: default_provider_kind(),
        base_url: settings.base_url.trim_end_matches('/').to_owned(),
        project_id: settings.project_id.trim().to_owned(),
        loader: default_loader(),
        check_minutes: settings.check_minutes.clamp(5, 1440),
        include_prereleases: false,
    };
    if provider.base_url.is_empty() || provider.project_id.is_empty() {
        return Err("update provider fields must not be empty".into());
    }
    shroudforge_loader_config::config::update_loader(root, |value| {
        if let Some(preferences) = &settings.module_preferences {
            for (module, keys) in [
                (
                    "debugConsole",
                    &[
                        "enabled",
                        "defaultSource",
                        "levelFilter",
                        "autoScroll",
                        "toggleKey",
                        "refreshMilliseconds",
                        "tailBytes",
                    ][..],
                ),
                (
                    "modloaderUi",
                    &["enabled", "startPage", "toggleKey", "refreshMilliseconds"][..],
                ),
                (
                    "worldEditor",
                    &["enabled", "toggleKey", "refreshMilliseconds"][..],
                ),
                (
                    "runtimeDiagnostics",
                    &[
                        "intervalMilliseconds",
                        "maximumDurationSeconds",
                        "slowCallbackMilliseconds",
                        "continuous",
                        "onlyChanges",
                        "areas",
                    ][..],
                ),
            ] {
                for key in keys {
                    if let Some(setting) = preferences.get(module).and_then(|value| value.get(*key))
                    {
                        value["modules"][module][*key] = setting.clone();
                    }
                }
                if let Some(position) = preferences
                    .get(module)
                    .and_then(|value| value.pointer("/window/position"))
                {
                    value["modules"][module]["window"]["position"] = position.clone();
                }
            }
            for key in ["enabled", "checkMinutes", "channel", "includePrereleases"] {
                if let Some(setting) = preferences.pointer(&format!("/updates/system/{key}")) {
                    value["modules"]["updates"]["system"][key] = setting.clone();
                }
            }
        }
        let logging = value
            .get_mut("logging")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or("logging config must be an object")?;
        logging.insert("minimumLevel".into(), settings.log_level.clone().into());
        let object = value
            .as_object_mut()
            .ok_or("shroudforge/config/modloader-config.json must contain an object")?;
        let exports = object
            .entry("exports")
            .or_insert_with(|| serde_json::json!({}));
        exports["enabled"] = settings.export_enabled.into();
        object.insert(
            "catalog".into(),
            serde_json::json!({ "provider": provider }),
        );
        let general = object
            .entry("general")
            .or_insert_with(|| serde_json::json!({}));
        let general = general
            .as_object_mut()
            .ok_or("general config must be an object")?;
        general.insert("compactMode".into(), settings.compact_mode.into());
        general.insert("reducedMotion".into(), settings.reduced_motion.into());
        general.insert(
            "usernameMode".into(),
            if settings.username_mode == "manual" {
                "manual"
            } else {
                "auto"
            }
            .into(),
        );
        general.insert(
            "messageNameFallback".into(),
            normalize_message_name(&settings.message_name_fallback)?.into(),
        );
        Ok(())
    })
}
