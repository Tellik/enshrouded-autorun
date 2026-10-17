use crate::*;

#[cfg(windows)]
pub(crate) fn with_log_lock<T>(
    _root: &Path,
    operation: impl FnOnce() -> io::Result<T>,
) -> io::Result<T> {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
    use windows_sys::Win32::{
        Foundation::{CloseHandle, WAIT_ABANDONED, WAIT_OBJECT_0},
        System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject},
    };
    let name = "Local\\ShroudForgeLog";
    let wide: Vec<u16> = OsStr::new(&name).encode_wide().chain(Some(0)).collect();
    let handle = unsafe { CreateMutexW(std::ptr::null(), 0, wide.as_ptr()) };
    if handle.is_null() {
        return Err(io::Error::last_os_error());
    }
    let wait = unsafe { WaitForSingleObject(handle, u32::MAX) };
    if wait != WAIT_OBJECT_0 && wait != WAIT_ABANDONED {
        unsafe {
            CloseHandle(handle);
        }
        return Err(io::Error::last_os_error());
    }
    let result = operation();
    unsafe {
        ReleaseMutex(handle);
        CloseHandle(handle);
    }
    result
}

#[cfg(not(windows))]
pub(crate) fn with_log_lock<T>(
    _root: &Path,
    operation: impl FnOnce() -> io::Result<T>,
) -> io::Result<T> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _guard = LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| io::Error::other("logging lock poisoned"))?;
    operation()
}
