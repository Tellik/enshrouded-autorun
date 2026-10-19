#![cfg_attr(windows, windows_subsystem = "windows")]

use std::io::Write;

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.iter().any(|arg| arg == "--request") {
        let root = arguments
            .windows(2)
            .find(|pair| pair[0] == "--root")
            .map(|pair| std::path::PathBuf::from(&pair[1]));
        let result = (|| {
            let root = root.ok_or("missing --root".to_owned())?;
            let mut input = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut input)
                .map_err(|error| format!("could not read updater request: {error}"))?;
            let request = serde_json::from_str(&input)
                .map_err(|error| format!("invalid updater request: {error}"))?;
            if shroudforge_updater::hand_off_request_to_worker(&root, &request, input.as_bytes())? {
                return Ok(None);
            }
            if let Some(pid) = arguments
                .windows(2)
                .find(|pair| pair[0] == "--wait-pid")
                .and_then(|pair| pair[1].parse::<u32>().ok())
            {
                shroudforge_updater::wait_for_request_parent(&root, pid)?;
            }
            shroudforge_updater::run_request(&root, &request).map(Some)
        })();
        let value = match result {
            Ok(None) => return,
            Ok(Some(value)) => value,
            Err(error) => {
                serde_json::json!({"ok": false, "error": error})
            }
        };
        let response = if value.get("ok").is_some() {
            value
        } else {
            serde_json::json!({"ok": true, "value": value})
        };
        let stdout = std::io::stdout();
        let mut output = stdout.lock();
        let _ = writeln!(output, "{response}");
        let _ = output.flush();
        drop(output);
        if response["ok"] != true {
            std::process::exit(1);
        }
        return;
    }
    let controller_mode = arguments.iter().any(|arg| arg == "--controller");
    let startup_check = arguments.iter().any(|arg| arg == "--startup-check");
    let result = if controller_mode {
        let root = arguments
            .windows(2)
            .find(|pair| pair[0] == "--root")
            .map(|pair| std::path::PathBuf::from(&pair[1]));
        let parent_pid = arguments
            .windows(2)
            .find(|pair| pair[0] == "--parent-pid")
            .and_then(|pair| pair[1].parse::<u32>().ok());
        match (root, parent_pid) {
            (Some(root), Some(pid)) => {
                let result = shroudforge_updater::run_headless_control_controller(&root, pid);
                if let Err(error) = &result {
                    let _ = shroudforge_logging::append(
                        &root,
                        'E',
                        "config-controller",
                        &format!("Updater controller stopped: {error}"),
                    );
                }
                result
            }
            _ => Err("missing or invalid --root/--parent-pid".into()),
        }
    } else if startup_check {
        let root = arguments
            .windows(2)
            .find(|pair| pair[0] == "--root")
            .map(|pair| std::path::PathBuf::from(&pair[1]));
        match root
            .ok_or("missing --root".to_owned())
            .and_then(|root| shroudforge_updater::startup_check(&root))
        {
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
            (Some(root), Some(pid)) => shroudforge_updater::run_request(
                std::path::Path::new(&root),
                &serde_json::json!({"command":"install-staged-update","waitPid":pid}),
            )
            .map(|_| ()),
            _ => Err("missing or invalid --root/--wait-pid".into()),
        }
    } else {
        shroudforge_updater::run_module()
    };
    if let Err(error) = result {
        // The bootstrap is waiting for this process. Return the failure before
        // it shows the startup error, rather than blocking it behind a dialog.
        #[cfg(windows)]
        if !startup_check && !controller_mode {
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
        if !controller_mode {
            eprintln!("ShroudForge updater: {error}");
        }
        std::process::exit(1);
    }
}
