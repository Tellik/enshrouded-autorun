#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let startup_check = arguments.iter().any(|arg| arg == "--startup-check");
    let result = if startup_check {
        let root = arguments
            .windows(2)
            .find(|pair| pair[0] == "--root")
            .map(|pair| std::path::PathBuf::from(&pair[1]));
        match root.ok_or("missing --root".to_owned())
            .and_then(|root| shroudforge_updater::startup_check(&root)) {
            Ok(required) => std::process::exit(if required { 2 } else { 0 }),
            Err(error) => Err(error),
        }
    } else if arguments.iter().any(|arg| arg == "--queue-install") {
        let value = |name: &str| {
            arguments
                .windows(2)
                .find(|pair| pair[0] == name)
                .map(|pair| pair[1].clone())
        };
        match (
            value("--root"),
            value("--wait-pid").and_then(|pid| pid.parse::<u32>().ok()),
        ) {
            (Some(root), Some(pid)) => {
                shroudforge_updater::request_install_after_game(std::path::Path::new(&root), pid)
            }
            _ => Err("missing or invalid --root/--wait-pid".into()),
        }
    } else {
        shroudforge_updater::run_module()
    };
    if let Err(error) = result {
        // The bootstrap is waiting for this process. Return the failure before
        // it shows the startup error, rather than blocking it behind a dialog.
        #[cfg(windows)]
        if !startup_check
        {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
            let message: Vec<u16> = std::ffi::OsStr::new(&format!("ShroudForge updater: {error}"))
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let title: Vec<u16> = "ShroudForge Updater"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            unsafe {
                MessageBoxW(
                    std::ptr::null_mut(),
                    message.as_ptr(),
                    title.as_ptr(),
                    MB_OK | MB_ICONERROR,
                );
            }
        }
        #[cfg(not(windows))]
        eprintln!("ShroudForge updater: {error}");
        std::process::exit(1);
    }
}
