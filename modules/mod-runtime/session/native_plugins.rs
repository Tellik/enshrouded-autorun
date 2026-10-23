use super::*;

impl AppState {
    pub(super) fn native_dll_state(&self, id: &str) -> Option<crate::native_dlls::NativeDllStatus> {
        crate::native_dlls::statuses()
            .into_iter()
            .find(|status| status.id == id)
    }

    /// Lua registrations retain their EML execution phase and package boundary.
    /// Loading and lifetime are shared with INI, profile and manual selections.
    pub(crate) fn load_mod_native_dll(
        &self,
        target_mod: &Mod,
        relative_path: &str,
    ) -> Result<(), String> {
        if !self.has_feature(AppFeatures::RUNTIME_DLL) {
            return Err("native DLL loading is unavailable in this execution phase".into());
        }
        if !target_mod.info().enabled {
            self.report_runtime_effect(&target_mod.info().id, "restart-required",
                "Native DLL was skipped because the mod is disabled. Enable it and restart the game");
            return Ok(());
        }
        let directory = target_mod
            .fs()
            .root()
            .file_name()
            .unwrap_or_default()
            .to_owned();
        if mod_loader::dll_manager::read_selections(self.env.game_dir().as_std_path())
            .iter()
            .find(|selection| selection.package_directory.eq_ignore_ascii_case(&directory))
            .is_some_and(|selection| !selection.target.applies_to(self.is_server()))
        {
            return Err("The DLL Manager selection targets a different game process".into());
        }
        let relative_path = relative_path.replace('\\', "/");
        let relative_path = relative_path.trim_start_matches("./").to_owned();
        let path = mod_loader::dll_manager::resolve_package_dll_path(
            self.env.game_dir().as_std_path(),
            &directory,
            &mod_loader::dll_manager::DllProfileFile {
                path: relative_path.clone(),
                mode: mod_loader::dll_manager::DllLoadMode::Library,
            },
        )?;
        let package_directory = directory;
        let mod_id = target_mod.info().id.clone();
        let status_id = format!(
            "dll:{}:{}",
            package_directory.to_ascii_lowercase(),
            relative_path.to_ascii_lowercase()
        );
        crate::native_dlls::queue(
            self.env.game_dir().as_std_path(),
            mod_loader::dll_manager::DllLoadEntry {
                status_id,
                mod_id: Some(mod_id),
                package_directory,
                path,
                mode: mod_loader::dll_manager::DllLoadMode::Library,
                source: "Lua register_dll".into(),
            },
        )?;
        self.poll_native_dll_loads();
        Ok(())
    }

    pub(crate) fn poll_native_dll_loads(&self) {
        for status in crate::native_dlls::statuses() {
            self.report_native_dll_effect(&status);
        }
    }

    pub(super) fn report_native_dll_effect(&self, status: &crate::native_dlls::NativeDllStatus) {
        self.report_runtime_effect(&status.id, status.state, &status.detail);
        if let Some(effect) = self.runtime_effects.borrow_mut().get_mut(&status.id) {
            effect["nativeSource"] = status.source.clone().into();
        }
    }
}
