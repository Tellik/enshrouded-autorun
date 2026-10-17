#[cfg(windows)]
mod character_name;
#[cfg(windows)]
mod host;

#[cfg(windows)]
pub fn run_module() -> Result<(), Box<dyn std::error::Error>> {
    host::window::run()
}

#[cfg(windows)]
pub use shroudforge_updater::mod_installation::install::run_catalog_install_worker;

#[cfg(not(windows))]
pub fn run_module() -> Result<(), Box<dyn std::error::Error>> {
    Err(std::io::Error::other("ShroudForge Modloader UI is available on Windows only").into())
}

#[cfg(not(windows))]
pub fn run_catalog_install_worker() -> Result<(), String> {
    Err("catalog install worker is available on Windows only".into())
}
