use super::*;

impl AppState {
    pub(super) fn native_dll_state(&self, id: &str) -> Option<&'static str> {
        if self.native_dlls.borrow().iter().any(|dll| dll.mod_id == id) {
            Some("loaded")
        } else if self
            .pending_native_dlls
            .borrow()
            .iter()
            .any(|dll| dll.mod_id == id)
        {
            Some("loading")
        } else {
            None
        }
    }

    pub(crate) fn load_mod_native_dll(
        &self,
        target_mod: &Mod,
        relative_path: &str,
    ) -> Result<(), String> {
        if !target_mod.info().enabled {
            self.report_runtime_effect(
                &target_mod.info().id,
                "restart-required",
                "Native DLL was skipped because the mod is disabled. Enable it and restart the game to load the DLL.",
            );
            tracing::info!(target: "shroudforge::runtime", mod_id = %target_mod.info().id,
                dll = %relative_path, "Native DLL skipped because the mod is disabled. Restart required after enabling");
            return Ok(());
        }
        self.queue_mod_native_dll(target_mod, relative_path)
    }

    fn queue_mod_native_dll(&self, target_mod: &Mod, relative_path: &str) -> Result<(), String> {
        if !self.has_feature(AppFeatures::RUNTIME_DLL) {
            return Err("native DLL loading is unavailable in this execution phase".into());
        }
        let package_path = camino::Utf8Path::new(relative_path);
        if package_path.as_str().is_empty()
            || package_path.is_absolute()
            || relative_path.contains(':')
            || package_path.components().any(|part| part.as_str() == "..")
        {
            return Err("DLL path must stay inside the mod package".into());
        }
        let mut filesystem = target_mod.fs();
        let (load_path, temporary_directory) = if let Some(path) = filesystem
            .absolute_path(package_path)
            .map_err(|error| error.to_string())?
        {
            if !path.is_file() {
                return Err(format!("registered DLL does not exist: {path}"));
            }
            (path.into_std_path_buf(), None)
        } else {
            let directory = std::env::temp_dir()
                .join("shroudforge")
                .join("native-mods")
                .join(target_mod.info().id.as_str())
                .join(uuid::Uuid::new_v4().to_string());
            std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
            let extraction = (|| {
                for file in filesystem.files() {
                    let file = file.map_err(|error| error.to_string())?;
                    if file.is_absolute() || file.components().any(|part| part.as_str() == "..") {
                        continue;
                    }
                    let destination = directory.join(file.as_std_path());
                    if let Some(parent) = destination.parent() {
                        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
                    }
                    let mut reader = filesystem
                        .read_file(&file)
                        .map_err(|error| error.to_string())?;
                    let mut output =
                        std::fs::File::create(&destination).map_err(|error| error.to_string())?;
                    std::io::copy(&mut reader, &mut output).map_err(|error| error.to_string())?;
                }
                let destination = directory.join(package_path.as_std_path());
                if !destination.is_file() {
                    return Err("registered DLL was not found in the mod archive".into());
                }
                Ok(destination)
            })();
            match extraction {
                Ok(path) => (path, Some(directory.clone())),
                Err(error) => {
                    let _ = std::fs::remove_dir_all(directory);
                    return Err(error);
                }
            }
        };
        let canonical = match load_path.canonicalize() {
            Ok(path) => path,
            Err(error) => {
                if let Some(directory) = temporary_directory {
                    let _ = std::fs::remove_dir_all(directory);
                }
                return Err(format!("could not resolve DLL path: {error}"));
            }
        };
        let mod_id = target_mod.info().id.to_string();
        let already_loaded = {
            self.native_dlls
                .borrow()
                .iter()
                .find(|item| item.path == canonical)
                .is_some()
        };
        if already_loaded {
            if let Some(directory) = temporary_directory {
                let _ = std::fs::remove_dir_all(directory);
            }
            return Ok(());
        }
        if let Some(_pending) = self
            .pending_native_dlls
            .borrow_mut()
            .iter_mut()
            .find(|item| item.path == canonical)
        {
            if let Some(directory) = temporary_directory {
                let _ = std::fs::remove_dir_all(directory);
            }
            return Ok(());
        }
        let (sender, receiver) = mpsc::channel();
        let worker_mod_id = mod_id.clone();
        let worker_path = canonical.clone();
        let worker_tempdir = temporary_directory.clone();
        tracing::debug!(target: "shroudforge::runtime", mod_id, dll = %canonical.display(),
            "Queuing native DLL load on isolated worker");
        std::thread::Builder::new()
            .name(format!(
                "sf-native-{}",
                mod_id.chars().take(24).collect::<String>()
            ))
            .spawn(move || {
                tracing::debug!(target: "shroudforge::runtime", mod_id = %worker_mod_id,
                    dll = %worker_path.display(), "Isolated native DLL load started");
                let result = load_native_library(&worker_mod_id, &worker_path);
                match &result {
                    Ok(_) => {
                        tracing::debug!(target: "shroudforge::runtime", mod_id = %worker_mod_id,
                        dll = %worker_path.display(), "Isolated native DLL load completed")
                    }
                    Err(error) => {
                        tracing::error!(target: "shroudforge::runtime", mod_id = %worker_mod_id,
                        dll = %worker_path.display(), "Isolated native DLL load failed: {error}")
                    }
                }
                if let Err(send_error) = sender.send(result) {
                    if let Ok(handle) = send_error.0 {
                        unload_native_library(handle);
                    }
                    if let Some(directory) = worker_tempdir {
                        let _ = std::fs::remove_dir_all(directory);
                    }
                }
            })
            .map_err(|error| {
                if let Some(directory) = &temporary_directory {
                    let _ = std::fs::remove_dir_all(directory);
                }
                format!("could not start isolated DLL loader thread: {error}")
            })?;
        self.pending_native_dlls
            .borrow_mut()
            .push(PendingNativeDll {
                mod_id,
                path: canonical,
                temporary_directory,
                queued_at: std::time::Instant::now(),
                warned_stall: false,
                receiver,
            });
        Ok(())
    }

    pub(crate) fn poll_native_dll_loads(&self) {
        let mut completed = Vec::new();
        {
            let mut pending = self.pending_native_dlls.borrow_mut();
            let mut index = 0;
            while index < pending.len() {
                match pending[index].receiver.try_recv() {
                    Ok(result) => completed.push((pending.remove(index), Some(result))),
                    Err(TryRecvError::Disconnected) => {
                        completed.push((pending.remove(index), None))
                    }
                    Err(TryRecvError::Empty) => {
                        if !pending[index].warned_stall
                            && pending[index].queued_at.elapsed()
                                >= std::time::Duration::from_secs(5)
                        {
                            tracing::warn!(target: "shroudforge::runtime", mod_id = %pending[index].mod_id,
                                dll = %pending[index].path.display(), elapsed_ms = pending[index].queued_at.elapsed().as_millis(),
                                "Native DLL load is still running on an isolated worker. Runtime updates continue");
                            pending[index].warned_stall = true;
                        }
                        index += 1;
                    }
                }
            }
        }
        for (item, result) in completed {
            match result
                .unwrap_or_else(|| Err("native DLL loader worker exited without a result".into()))
            {
                Ok(handle) => {
                    self.native_dlls.borrow_mut().push(NativeDll {
                        mod_id: item.mod_id.clone(),
                        path: item.path.clone(),
                        handle,
                        temporary_directory: item.temporary_directory.clone(),
                    });
                    tracing::info!(target: "shroudforge::runtime", mod_id = %item.mod_id,
                        dll = %item.path.display(), "Native DLL load completed");
                    self.report_runtime_effect(&item.mod_id, "loaded", "native DLL loaded");
                }
                Err(error) => {
                    if let Some(directory) = item.temporary_directory {
                        let _ = std::fs::remove_dir_all(directory);
                    }
                    tracing::error!(target: "shroudforge::runtime", mod_id = %item.mod_id,
                        dll = %item.path.display(), "Native DLL load failed: {error}");
                    self.report_runtime_effect(&item.mod_id, "error", &error);
                }
            }
        }
    }

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

pub(super) struct NativeDll {
    mod_id: String,
    path: NativePathBuf,
    #[allow(dead_code)]
    handle: isize,
    temporary_directory: Option<NativePathBuf>,
}

pub(super) struct PendingNativeDll {
    mod_id: String,
    path: NativePathBuf,
    temporary_directory: Option<NativePathBuf>,
    queued_at: std::time::Instant,
    warned_stall: bool,
    receiver: Receiver<Result<isize, String>>,
}

#[cfg(windows)]
fn load_native_library(mod_id: &str, path: &std::path::Path) -> Result<isize, String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::System::LibraryLoader::{
        LOAD_LIBRARY_SEARCH_DEFAULT_DIRS, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR, LoadLibraryExW,
    };

    tracing::debug!(target: "shroudforge::runtime", mod_id, dll = %path.display(),
        "Calling LoadLibraryExW on isolated worker");
    let wide_path = path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let handle = unsafe {
        LoadLibraryExW(
            wide_path.as_ptr(),
            std::ptr::null_mut(),
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
        )
    };
    if handle.is_null() {
        Err(format!(
            "Windows could not load native DLL for mod '{mod_id}': {}",
            std::io::Error::last_os_error()
        ))
    } else {
        tracing::debug!(target: "shroudforge::runtime", mod_id, dll = %path.display(),
            "LoadLibraryExW returned on isolated worker");
        Ok(handle as isize)
    }
}

#[cfg(windows)]
fn unload_native_library(handle: isize) {
    unsafe {
        windows_sys::Win32::Foundation::FreeLibrary(
            handle as windows_sys::Win32::Foundation::HMODULE,
        );
    }
}

#[cfg(not(windows))]
fn load_native_library(mod_id: &str, _path: &std::path::Path) -> Result<isize, String> {
    Err(format!(
        "native DLL loading for mod '{mod_id}' is supported only on Windows"
    ))
}

#[cfg(not(windows))]
fn unload_native_library(_handle: isize) {}

impl AppState {
    pub(super) fn shutdown_native_dlls(&mut self) {
        for dll in self.native_dlls.get_mut().drain(..).rev() {
            #[cfg(windows)]
            unsafe {
                windows_sys::Win32::Foundation::FreeLibrary(
                    dll.handle as windows_sys::Win32::Foundation::HMODULE,
                );
            }
            if let Some(directory) = dll.temporary_directory {
                if let Err(error) = std::fs::remove_dir_all(directory) {
                    tracing::warn!(target: "shroudforge::runtime", %error, "Could not remove extracted native DLL files");
                }
            }
        }
    }
}
