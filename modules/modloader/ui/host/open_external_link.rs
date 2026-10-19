use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::UI::Shell::ShellExecuteW;

pub(crate) fn open_url(url: &str) -> Result<(), String> {
    if !url.starts_with("https://") && url != "mailto:support@shroudedit.com" {
        return Err("Only HTTPS links and the ShroudEdit support address can be opened.".into());
    }
    let operation: Vec<u16> = std::ffi::OsStr::new("open")
        .encode_wide()
        .chain(Some(0))
        .collect();
    let target: Vec<u16> = std::ffi::OsStr::new(url)
        .encode_wide()
        .chain(Some(0))
        .collect();
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            operation.as_ptr(),
            target.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1,
        ) as isize
    };
    if result <= 32 {
        return Err(format!(
            "Windows could not open the link (ShellExecuteW returned {result})."
        ));
    }
    Ok(())
}
