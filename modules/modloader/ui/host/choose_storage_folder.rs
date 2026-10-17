use std::path::PathBuf;
use windows::Win32::System::Com::CLSCTX_INPROC_SERVER;
use windows::Win32::System::Com::COINIT_APARTMENTTHREADED;
use windows::Win32::System::Com::CoCreateInstance;
use windows::Win32::System::Com::CoInitializeEx;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::Com::CoUninitialize;
use windows::Win32::UI::Shell::FOS_FORCEFILESYSTEM;
use windows::Win32::UI::Shell::FOS_PICKFOLDERS;
use windows::Win32::UI::Shell::FileOpenDialog;
use windows::Win32::UI::Shell::IFileOpenDialog;
use windows::Win32::UI::Shell::SIGDN_FILESYSPATH;
use windows::core::PCWSTR;

pub(crate) fn choose_folder(key: &str) -> Result<serde_json::Value, String> {
    let title = match key {
        "mods" => "Choose the mods folder",
        "state" => "Choose the status folder",
        "logs" => "Choose the logs folder",
        "cache" => "Choose the cache folder",
        "exports" => "Choose the exports folder",
        "updates" => "Choose the updates folder",
        "ui" => "Choose the UI data folder",
        "runtime" => "Choose the runtime data folder",
        _ => return Err("unsupported storage folder".into()),
    };
    let initialized = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    initialized
        .ok()
        .map_err(|error| format!("could not initialize the Windows folder picker: {error}"))?;
    struct ComUninitialize;
    impl Drop for ComUninitialize {
        fn drop(&mut self) {
            unsafe { CoUninitialize() };
        }
    }
    let _com = ComUninitialize;

    let title: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();
    let dialog = unsafe {
        CoCreateInstance::<_, IFileOpenDialog>(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
    }
    .map_err(|error| format!("could not open the Windows folder picker: {error}"))?;
    unsafe {
        dialog
            .SetTitle(PCWSTR(title.as_ptr()))
            .and_then(|()| dialog.SetOptions(FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM))
            .map_err(|error| format!("could not configure the Windows folder picker: {error}"))?;
        dialog
            .Show(None)
            .map_err(|_| "folder selection was cancelled".to_owned())?;
    }
    let selected = unsafe { dialog.GetResult() }
        .map_err(|error| format!("could not read the selected folder: {error}"))?;
    let display_name = unsafe { selected.GetDisplayName(SIGDN_FILESYSPATH) }
        .map_err(|error| format!("could not read the selected folder: {error}"))?;
    let path_string: Result<String, std::string::FromUtf16Error> =
        unsafe { display_name.to_string() };
    unsafe { CoTaskMemFree(Some(display_name.0.cast())) };
    let path = PathBuf::from(path_string.map_err(|error| error.to_string())?);
    if !path.is_dir() {
        return Err("select an existing folder".into());
    }
    Ok(serde_json::Value::String(
        path.to_string_lossy().into_owned(),
    ))
}
