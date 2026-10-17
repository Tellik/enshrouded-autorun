use super::ui_snapshot_types::ModuleConfig;
use super::ui_snapshot_types::UserConfig;
use super::window::DEFAULT_TOGGLE_KEY;
use shroudforge_package::config_revision;
use shroudforge_updater::provider_config::CatalogProviderConfig;
use std::path::Path;

pub(crate) fn default_toggle_key() -> u32 {
    DEFAULT_TOGGLE_KEY
}

pub(crate) fn default_refresh() -> u64 {
    1000
}

pub(crate) fn load_module_config(root: &Path) -> Result<ModuleConfig, String> {
    let value = shroudforge_loader_config::config::read_loader(root)?;
    let mut config: ModuleConfig = serde_json::from_value(value["modules"]["modloaderUi"].clone())
        .map_err(|e| e.to_string())?;
    if let Some(enabled) = value
        .pointer("/modules/updates/system/enabled")
        .and_then(|v| v.as_bool())
    {
        config.system_update_provider.enabled = enabled;
    }
    if let Some(minutes) = value
        .pointer("/modules/updates/system/checkMinutes")
        .and_then(|v| v.as_u64())
    {
        config.system_update_provider.check_minutes = minutes;
    }
    if let Some(include_prereleases) = value
        .pointer("/modules/updates/system/includePrereleases")
        .and_then(|v| v.as_bool())
    {
        config.system_update_provider.include_prereleases = include_prereleases;
    }
    Ok(config)
}

pub(crate) fn read_central_config(root: &Path) -> serde_json::Value {
    shroudforge_loader_config::config::read_loader(root).unwrap_or_else(|error| {
        eprintln!("loader configuration: {error}");
        serde_json::json!({"configurationError": error})
    })
}

pub(crate) fn load_user_config(root: &Path) -> UserConfig {
    let value = read_central_config(root);
    UserConfig {
        update_provider: value
            .pointer("/catalog/provider")
            .cloned()
            .and_then(|provider| serde_json::from_value(provider).ok()),
        locale: value
            .pointer("/general/locale")
            .and_then(|value| value.as_str())
            .map(str::to_owned),
    }
}

pub(crate) fn effective_provider(user: &UserConfig) -> CatalogProviderConfig {
    user.update_provider.clone().unwrap_or_default()
}

pub(crate) fn live_settings_revision(root: &Path) -> String {
    let config = shroudforge_loader_config::config::read_loader(root).unwrap_or_default();
    config_revision(&serde_json::json!({
        "general": config["general"],
        "logging": config["logging"],
        "exports": config["exports"],
        "paths": config["paths"],
        "runtime": config["runtime"],
        "catalog": config["catalog"],
        "modules": config["modules"]
    }))
}
