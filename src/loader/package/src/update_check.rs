//! Coordinates system update checks across ShroudForge processes.

use std::path::Path;

#[cfg(windows)]
use sha2::{Digest, Sha256};

pub const SYSTEM_UPDATE_CHECK_COOLDOWN_SECONDS: u64 = 5 * 60;
const SYSTEM_UPDATE_CHECK_LOCK_WAIT_MILLIS: u32 = 45_000;

pub enum SystemUpdateCheckStart {
    Started(SystemUpdateCheckLock),
    CoolingDown { remaining_seconds: u64 },
    AlreadyRunning,
}

/// Acquires the per-installation check lock and applies the automatic cooldown.
///
/// `manual` bypasses the cooldown, but never starts a parallel check.
pub fn begin_system_update_check(
    root: &Path,
    manual: bool,
) -> Result<SystemUpdateCheckStart, String> {
    let request_started_at = unix_timestamp();
    let Some((lock, waited_for_running_check)) = SystemUpdateCheckLock::try_acquire(root)? else {
        return Ok(SystemUpdateCheckStart::AlreadyRunning);
    };

    let last_check_at = crate::config::read_loader(root).ok().and_then(|value| {
        value
            .pointer("/modules/updates/system/lastCheckAt")
            .and_then(serde_json::Value::as_u64)
    });
    if let Some(checked_at) = last_check_at {
        let now = unix_timestamp();
        let elapsed = now.saturating_sub(checked_at);
        let recent_automatic_result = !manual && elapsed < SYSTEM_UPDATE_CHECK_COOLDOWN_SECONDS;
        let completed_while_manual_waited =
            manual && waited_for_running_check && checked_at >= request_started_at;
        if recent_automatic_result || completed_while_manual_waited {
            drop(lock);
            return Ok(SystemUpdateCheckStart::CoolingDown {
                remaining_seconds: SYSTEM_UPDATE_CHECK_COOLDOWN_SECONDS.saturating_sub(elapsed),
            });
        }
    }

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
