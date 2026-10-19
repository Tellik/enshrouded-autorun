use std::path::PathBuf;
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
    CoTaskMemFree, CoUninitialize,
};
use windows::Win32::UI::Shell::{
    FOS_FORCEFILESYSTEM, FOS_PICKFOLDERS, FileOpenDialog, IFileOpenDialog, SIGDN_FILESYSPATH,
};
use windows::core::PCWSTR;

pub fn choose_folder(title: &str) -> Result<Option<PathBuf>, String> {
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
    }
    if unsafe { dialog.Show(None) }.is_err() {
        return Ok(None);
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
    Ok(Some(path))
}
