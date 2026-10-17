use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::OpenEventW;

pub(crate) fn open_event(name: &str) -> HANDLE {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = std::ffi::OsStr::new(name)
        .encode_wide()
        .chain(Some(0))
        .collect();
    unsafe { OpenEventW(0x0010_0000, 0, wide.as_ptr()) }
}
