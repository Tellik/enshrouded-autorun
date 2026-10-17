use super::read_host_arguments::Arguments;
use super::read_ui_configuration::read_central_config;
use super::report_command_result::report_command_result;
use std::process::Child;
use std::time::Instant;
use windows_sys::Win32::System::Threading::GetCurrentProcessId;

pub(crate) fn request_module_visibility(
    module: String,
    visible: bool,
    request_id: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    desktop_debug_console: &mut Option<Child>,
    next_refresh: &mut Instant,
) {
    let result = if module == "debugConsole" && visible && arguments.desktop {
        if read_central_config(&arguments.root)["modules"]["debugConsole"]["enabled"] == false {
            Err("Debug Console is disabled in module settings.".into())
        } else if desktop_debug_console.is_none() {
            let executable = shroudforge_loader_config::paths::loader_executable(&arguments.root);
            match std::process::Command::new(executable)
                .args(["--debug-console", "--desktop", "--root"])
                .arg(&arguments.root)
                .arg("--owner-pid")
                .arg(unsafe { GetCurrentProcessId() }.to_string())
                .spawn()
            {
                Ok(child) => {
                    *desktop_debug_console = Some(child);
                    Ok(())
                }
                Err(error) => Err(format!("Could not start Desktop Debug Console: {error}")),
            }
        } else {
            shroudforge_loader_config::config::request_window_visibility(
                &arguments.root,
                &module,
                true,
            )
        }
    } else if module == "debugConsole"
        && visible
        && shroudforge_loader_config::config::read_loader(&arguments.root)
            .is_ok_and(|config| config["modules"]["debugConsole"]["enabled"] == false)
    {
        Err("Debug Console is disabled for game startup.".into())
    } else {
        shroudforge_loader_config::config::request_window_visibility(
            &arguments.root,
            &module,
            visible,
        )
    };
    let action = if visible {
        "Show window"
    } else {
        "Hide window"
    };
    report_command_result(
        &arguments.root,
        &webview,
        &request_id,
        &module,
        action,
        result,
        "settings.visibilityRequested",
        None,
        false,
    );
    *next_refresh = Instant::now();
}
