#[cfg(windows)]
#[path = "ui/mod.rs"]
mod host;

#[cfg(windows)]
pub fn run_module() -> Result<(), Box<dyn std::error::Error>> {
    host::window::run()
}

#[cfg(not(windows))]
pub fn run_module() -> Result<(), Box<dyn std::error::Error>> {
    Err(std::io::Error::other("ShroudForge Debug Console is available on Windows only").into())
}

#[cfg(windows)]
mod read_log_tail;
#[cfg(windows)]
mod save_console_preferences;
#[cfg(windows)]
mod scan_log_for_alert;
