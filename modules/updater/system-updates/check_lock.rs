//! Coordinates system update checks across ShroudForge processes.

use std::path::Path;

#[cfg(windows)]
use sha2::{Digest, Sha256};

const SYSTEM_UPDATE_CHECK_LOCK_WAIT_MILLIS: u32 = 45_000;

#[derive(Clone, Copy)]
pub enum SystemUpdateCheckMode {
    Automatic,
    Manual,
}

pub enum SystemUpdateCheckStart {
    Started(SystemUpdateCheckLock),
    CoolingDown { remaining_seconds: u64 },
    AlreadyRunning,
    Disabled,
}

pub fn updates_enabled(root: &Path) -> bool {
    shroudforge_loader_config::config::read_loader(root)
        .ok()
        .and_then(|value| value.pointer("/modules/updates/enabled").cloned())
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

pub(crate) fn system_update_check_interval_seconds(settings: Option<&serde_json::Value>) -> u64 {
    settings
        .and_then(|value| value.pointer("/modules/updates/automatic/checkMinutes"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(60)
        .clamp(5, 1440)
        * 60
}

pub(crate) fn last_system_update_check_at(settings: Option<&serde_json::Value>) -> Option<u64> {
    let system = settings?.pointer("/modules/updates/system")?;
    ["lastCheckAt", "lastCheckAttemptAt"]
        .into_iter()
        .filter_map(|key| system.get(key).and_then(serde_json::Value::as_u64))
        .max()
}

/// Acquires the per-installation lock and records the start of an allowed check.
///
/// Automatic checks use the configured interval, including after failed attempts.
/// Manual checks bypass that interval, but never start a parallel check.
pub fn begin_system_update_check(
    root: &Path,
    mode: SystemUpdateCheckMode,
) -> Result<SystemUpdateCheckStart, String> {
    let config = shroudforge_loader_config::config::read_loader(root).ok();
    let module_enabled = config
        .as_ref()
        .and_then(|value| value.pointer("/modules/updates/enabled"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if !module_enabled {
        return Ok(SystemUpdateCheckStart::Disabled);
    }

    let enabled = config
        .as_ref()
        .and_then(|value| value.pointer("/modules/updates/automatic/enabled"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    if matches!(mode, SystemUpdateCheckMode::Automatic) && !enabled {
        return Ok(SystemUpdateCheckStart::Disabled);
    }

    let request_started_at = unix_timestamp();
    let Some((lock, waited_for_running_check)) = SystemUpdateCheckLock::try_acquire(root)? else {
        return Ok(SystemUpdateCheckStart::AlreadyRunning);
    };

    let interval_seconds = system_update_check_interval_seconds(config.as_ref());
    let last_check_at = last_system_update_check_at(config.as_ref());
    if let Some(checked_at) = last_check_at {
        let now = unix_timestamp();
        let elapsed = now.saturating_sub(checked_at);
        let recent_automatic_result =
            matches!(mode, SystemUpdateCheckMode::Automatic) && elapsed < interval_seconds;
        let completed_while_manual_waited = matches!(mode, SystemUpdateCheckMode::Manual)
            && waited_for_running_check
            && checked_at >= request_started_at;
        if recent_automatic_result || completed_while_manual_waited {
            drop(lock);
            return Ok(SystemUpdateCheckStart::CoolingDown {
                remaining_seconds: interval_seconds.saturating_sub(elapsed),
            });
        }
    }

    let started_at = unix_timestamp();
    shroudforge_loader_config::config::update_loader(root, |value| {
        value["modules"]["updates"]["system"]["lastCheckAttemptAt"] = serde_json::json!(started_at);
        Ok(())
    })?;
    Ok(SystemUpdateCheckStart::Started(lock))
}

pub fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(windows)]
pub struct SystemUpdateCheckLock {
    handle: windows_sys::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl SystemUpdateCheckLock {
    fn try_acquire(root: &Path) -> Result<Option<(Self, bool)>, String> {
        use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
        use windows_sys::Win32::{
            Foundation::{CloseHandle, WAIT_ABANDONED, WAIT_OBJECT_0, WAIT_TIMEOUT},
            System::Threading::{CreateMutexW, WaitForSingleObject},
        };

        let normalized_root = root
            .canonicalize()
            .unwrap_or_else(|_| root.to_path_buf())
            .to_string_lossy()
            .replace('/', "\\")
            .to_ascii_lowercase();
        let name = format!(
            "Local\\ShroudForgeSystemUpdateCheck_{:x}",
            Sha256::digest(normalized_root.as_bytes())
        );
        let wide: Vec<u16> = OsStr::new(&name).encode_wide().chain(Some(0)).collect();
        let handle = unsafe { CreateMutexW(std::ptr::null(), 0, wide.as_ptr()) };
        if handle.is_null() {
            return Err(std::io::Error::last_os_error().to_string());
        }

        let immediate = unsafe { WaitForSingleObject(handle, 0) };
        let (wait, waited) = if immediate == WAIT_TIMEOUT {
            (
                unsafe { WaitForSingleObject(handle, SYSTEM_UPDATE_CHECK_LOCK_WAIT_MILLIS) },
                true,
            )
        } else {
            (immediate, false)
        };

        match wait {
            WAIT_OBJECT_0 | WAIT_ABANDONED => Ok(Some((Self { handle }, waited))),
            WAIT_TIMEOUT => {
                unsafe { CloseHandle(handle) };
                Ok(None)
            }
            _ => {
                let error = std::io::Error::last_os_error().to_string();
                unsafe { CloseHandle(handle) };
                Err(error)
            }
        }
    }
}

#[cfg(windows)]
impl Drop for SystemUpdateCheckLock {
    fn drop(&mut self) {
        use windows_sys::Win32::{Foundation::CloseHandle, System::Threading::ReleaseMutex};
        unsafe {
            ReleaseMutex(self.handle);
            CloseHandle(self.handle);
        }
    }
}

#[cfg(not(windows))]
pub struct SystemUpdateCheckLock {
    _guard: parking_lot::MutexGuard<'static, ()>,
}

#[cfg(not(windows))]
impl SystemUpdateCheckLock {
    fn try_acquire(_: &Path) -> Result<Option<(Self, bool)>, String> {
        use std::sync::OnceLock;
        static LOCK: OnceLock<parking_lot::Mutex<()>> = OnceLock::new();
        Ok(LOCK
            .get_or_init(|| parking_lot::Mutex::new(()))
            .try_lock()
            .map(|guard| Self { _guard: guard })
            .map(|lock| (lock, false)))
    }
}
