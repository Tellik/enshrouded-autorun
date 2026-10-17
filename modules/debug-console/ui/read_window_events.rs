use super::read_console_arguments::SYNCHRONIZE_ACCESS;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::OpenEventW;
use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

pub(crate) fn open_event(name: &str) -> HANDLE {
    let mut wide: Vec<u16> = name.encode_utf16().collect();
    wide.push(0);
    unsafe { OpenEventW(SYNCHRONIZE_ACCESS, 0, wide.as_ptr()) }
}

pub(crate) fn foreground_process() -> u32 {
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(GetForegroundWindow(), &mut pid) };
    pid
}
