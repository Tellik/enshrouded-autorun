use crate::LoaderError;

#[cfg(windows)]
pub(super) fn ensure_game_stopped(file_name: &str) -> Result<(), LoaderError> {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, INVALID_HANDLE_VALUE},
        System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
            TH32CS_SNAPPROCESS,
        },
    };
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(LoaderError::Pregame(
                "cannot verify that Enshrouded is stopped".into(),
            ));
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut found = Process32FirstW(snapshot, &mut entry) != 0;
        let mut running = false;
        while found {
            let len = entry
                .szExeFile
                .iter()
                .position(|value| *value == 0)
                .unwrap_or(entry.szExeFile.len());
            let name = String::from_utf16_lossy(&entry.szExeFile[..len]);
            if name.eq_ignore_ascii_case(&format!("{file_name}.exe")) {
                running = true;
                break;
            }
            found = Process32NextW(snapshot, &mut entry) != 0;
        }
        CloseHandle(snapshot);
        if running {
            return Err(LoaderError::Pregame("close Enshrouded before preparing asset mods; live runtime mods do not need prepare".into()));
        }
    }
    Ok(())
}

#[cfg(not(windows))]
pub(super) fn ensure_game_stopped(_file_name: &str) -> Result<(), LoaderError> {
    Ok(())
}
