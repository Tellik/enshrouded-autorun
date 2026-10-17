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

pub(crate) fn choose_screenshot_folder() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let initialized = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    initialized.ok().map_err(|error| {
        format!("could not initialize the Windows screenshot-folder picker: {error}")
    })?;
    struct ComUninitialize;
    impl Drop for ComUninitialize {
        fn drop(&mut self) {
            unsafe { CoUninitialize() };
        }
    }
    let _com = ComUninitialize;

    let title: Vec<u16> = "Choose the folder containing your screenshots"
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let dialog = unsafe {
        CoCreateInstance::<_, IFileOpenDialog>(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
    }?;
    unsafe {
        dialog
            .SetTitle(PCWSTR(title.as_ptr()))
            .and_then(|()| dialog.SetOptions(FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM))?;
        dialog.Show(None)?;
    }
    let selected = unsafe { dialog.GetResult() }?;
    let display_name = unsafe { selected.GetDisplayName(SIGDN_FILESYSPATH) }?;
    let path_string: Result<String, std::string::FromUtf16Error> =
        unsafe { display_name.to_string() };
    unsafe { CoTaskMemFree(Some(display_name.0.cast())) };
    let folder = PathBuf::from(path_string?);
    if !folder.is_dir() {
        return Err("The selected screenshot folder is unavailable".into());
    }
    Ok(folder)
}
