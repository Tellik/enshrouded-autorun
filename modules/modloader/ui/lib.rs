#[cfg(windows)]
mod host;

#[cfg(windows)]
pub fn run_module() -> Result<(), Box<dyn std::error::Error>> {
    host::window::run()
}

#[cfg(not(windows))]
pub fn run_module() -> Result<(), Box<dyn std::error::Error>> {
    Err(std::io::Error::other("ShroudForge Modloader UI is available on Windows only").into())
}
