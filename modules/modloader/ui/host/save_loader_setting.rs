use super::read_runtime_profiles::runtime_profile_ids;
use super::validate_message_name::normalize_message_name;
use std::path::Path;
use std::path::PathBuf;

pub(crate) fn save_setting(
    root: &Path,
    scope: &str,
    module: Option<&str>,
    key: &str,
    setting: &serde_json::Value,
) -> Result<(), String> {
    if scope == "general" {
        if !matches!(
            key,
            "compactMode" | "reducedMotion" | "usernameMode" | "messageNameFallback"
        ) {
            return Err("unsupported general setting".into());
        }
        let setting = if key == "messageNameFallback" {
            normalize_message_name(setting.as_str().ok_or("message name must be text")?)?.into()
        } else if key == "usernameMode" {
            match setting.as_str() {
                Some("auto") => "auto".into(),
                Some("manual") => "manual".into(),
                _ => return Err("username mode must be auto or manual".into()),
            }
        } else {
            setting.clone()
        };
        return shroudforge_loader_config::config::update_loader(root, |value| {
            value["general"][key] = setting;
            Ok(())
        });
    }
    if scope == "logging" {
        if key != "minimumLevel" {
            return Err("unsupported logging setting".into());
        }
        return shroudforge_loader_config::config::update_loader(root, |value| {
            value["logging"][key] = setting.clone();
            Ok(())
        });
    }
    if scope == "exports" {
        if key != "enabled" {
            return Err("unsupported export setting".into());
        }
        return shroudforge_loader_config::config::update_loader(root, |value| {
            value["exports"][key] = setting.clone();
            Ok(())
        });
    }
    if scope == "paths" {
        if !matches!(
            key,
            "mods" | "state" | "logs" | "cache" | "exports" | "updates" | "ui" | "runtime"
        ) {
            return Err("unsupported storage path".into());
        }
        let normalized = if setting.is_null() {
            serde_json::Value::Null
        } else {
            let text = setting
                .as_str()
                .ok_or("storage path must be a folder path or null")?;
            let selected = PathBuf::from(text);
            if !selected.is_absolute() || !selected.is_dir() {
                return Err("select an existing folder".into());
            }
            let selected = selected.canonicalize().map_err(|error| error.to_string())?;
            if selected == root.canonicalize().unwrap_or_else(|_| root.to_path_buf()) {
                return Err("a storage folder cannot be the game installation root".into());
            }
            serde_json::Value::String(selected.to_string_lossy().into_owned())
        };
        return shroudforge_loader_config::config::update_loader(root, |value| {
            value["paths"][key] = normalized;
            Ok(())
        });
    }
    if scope == "runtime" {
        if key != "profileId" {
            return Err("unsupported runtime setting".into());
        }
        if let Some(id) = setting.as_str() {
            if !runtime_profile_ids(root)
                .iter()
                .any(|profile| profile == id)
            {
                return Err(
                    "runtime profile was not found under shroudforge/runtime/profiles".into(),
                );
            }
        } else if !setting.is_null() {
            return Err(
                "runtime profile selection must be a profile id or null for automatic".into(),
            );
        }
        return shroudforge_loader_config::config::update_loader(root, |value| {
            value["runtime"]["profileId"] = setting.clone();
            Ok(())
        });
    }
    if scope != "module" {
        return Err("unsupported settings scope".into());
    }
    let module = module.ok_or("missing module setting target")?;
    let valid = match module {
        "debugConsole" => {
            matches!(
                key,
                "enabled"
                    | "defaultSource"
                    | "levelFilter"
                    | "autoScroll"
                    | "toggleKey"
                    | "refreshMilliseconds"
                    | "tailBytes"
            ) || key == "window.position"
        }
        "modloaderUi" => {
            matches!(
                key,
                "enabled" | "startPage" | "toggleKey" | "refreshMilliseconds"
            ) || key == "window.position"
        }
        "worldEditor" => matches!(key, "enabled" | "toggleKey" | "refreshMilliseconds"),
        "network" => matches!(key, "serverSteamId" | "allowedClientSteamIds"),
        "runtimeDiagnostics" => matches!(
            key,
            "intervalMilliseconds"
                | "maximumDurationSeconds"
                | "slowCallbackMilliseconds"
                | "continuous"
                | "onlyChanges"
                | "areas"
        ),
        "updates.system" => matches!(
            key,
            "enabled" | "checkMinutes" | "channel" | "includePrereleases"
        ),
        _ => false,
    };
    if !valid {
        return Err("unsupported module setting".into());
    }
    if module == "network" {
        let text = setting.as_str().ok_or("Network settings must be text")?;
        if key == "serverSteamId"
            && !text.is_empty()
            && (!(16..=20).contains(&text.len())
                || !text.bytes().all(|byte| byte.is_ascii_digit())
                || text.parse::<u64>().map_or(true, |id| id == 0))
        {
            return Err("server SteamID64 must be empty or contain 16 to 20 decimal digits".into());
        }
        if key == "allowedClientSteamIds" {
            if text.len() > 4096
                || text.split([',', ';']).map(str::trim).any(|peer| {
                    !peer.is_empty()
                        && (!(16..=20).contains(&peer.len())
                            || !peer.bytes().all(|byte| byte.is_ascii_digit())
                            || peer.parse::<u64>().map_or(true, |id| id == 0))
                })
            {
                return Err("authorized client IDs must be decimal SteamID64 values separated by commas or semicolons".into());
            }
        }
    }
    shroudforge_loader_config::config::update_loader(root, |config| {
        if key == "window.position" {
            config["modules"][module]["window"]["position"] = setting.clone();
        } else if module == "updates.system" {
            config["modules"]["updates"]["system"][key] = setting.clone();
        } else {
            config["modules"][module][key] = setting.clone();
        }
        Ok(())
    })
}
