use super::*;
impl AppState {
    pub(crate) fn load_native_dll_declaration(&self, target_mod: &Mod) -> Result<bool, String> {
        if !self.has_feature(AppFeatures::RUNTIME_DLL)
            || !target_mod
                .info()
                .capabilities
                .contains(&Capability::RuntimeRegisterDll)
        {
            return Ok(false);
        }
        let config_path = camino::Utf8Path::new("native-plugin.ini");
        let config = {
            let mut filesystem = target_mod.fs();
            if !filesystem.is_file(config_path) {
                return Ok(false);
            }
            let mut config = String::new();
            filesystem
                .read_file(config_path)
                .map_err(|error| format!("could not read native-plugin.ini: {error}"))?
                .read_to_string(&mut config)
                .map_err(|error| format!("could not decode native-plugin.ini: {error}"))?;
            config
        };
        let mut in_plugin_section = false;
        let mut enabled = false;
        let mut dll = None;
        for raw_line in config.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                in_plugin_section = line[1..line.len() - 1]
                    .trim()
                    .eq_ignore_ascii_case("Plugin");
                continue;
            }
            if !in_plugin_section {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            let value = if value.len() >= 2
                && ((value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\'')))
            {
                &value[1..value.len() - 1]
            } else {
                value
            };
            match key.trim().to_ascii_lowercase().as_str() {
                "enabled" => {
                    enabled = matches!(
                        value.to_ascii_lowercase().as_str(),
                        "1" | "true" | "yes" | "on"
                    )
                }
                "dll" => dll = Some(value.to_owned()),
                _ => {}
            }
        }
        if !enabled {
            return Ok(false);
        }
        let dll = dll.ok_or_else(|| {
            "native-plugin.ini enables DLL loading but does not name a DLL".to_owned()
        })?;
        tracing::debug!(target: "shroudforge::runtime", mod_id = %target_mod.info().id,
            dll = %dll, "Native DLL declaration parsed. Queuing isolated load");
        self.queue_mod_native_dll(target_mod, &dll)?;
        Ok(true)
    }
}
