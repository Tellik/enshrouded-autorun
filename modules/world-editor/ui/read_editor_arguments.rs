use std::path::PathBuf;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::OpenProcess;

pub(crate) struct Arguments {
    pub(crate) root: PathBuf,
    pub(crate) game_pid: u32,
    pub(crate) game_process: HANDLE,
    pub(crate) stop_name: Option<String>,
    pub(crate) desktop: bool,
}

pub(crate) fn arguments() -> Result<Arguments, Box<dyn std::error::Error>> {
    let values: Vec<String> = std::env::args().collect();
    let value = |name: &str| {
        values
            .windows(2)
            .find(|pair| pair[0] == name)
            .map(|pair| pair[1].clone())
    };
    let root = PathBuf::from(value("--root").ok_or("missing --root")?);
    let desktop = values.iter().any(|arg| arg == "--desktop");
    let game_pid = if desktop {
        0
    } else {
        value("--game-pid").ok_or("missing --game-pid")?.parse()?
    };
    let game_process = if desktop {
        std::ptr::null_mut()
    } else {
        unsafe { OpenProcess(0x0010_0000, 0, game_pid) }
    };
    if !desktop && game_process.is_null() {
        return Err("could not open Enshrouded process for lifetime tracking".into());
    }
    Ok(Arguments {
        root,
        game_pid,
        game_process,
        stop_name: value("--stop-event"),
        desktop,
    })
}
