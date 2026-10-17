use std::path::PathBuf;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::GetCurrentProcessId;
use windows_sys::Win32::System::Threading::OpenProcess;
use windows_sys::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION;

pub(crate) const SYNCHRONIZE_ACCESS: u32 = 0x0010_0000;

pub(crate) struct Arguments {
    pub(crate) root: PathBuf,
    pub(crate) game_pid: u32,
    pub(crate) owner_process: HANDLE,
    pub(crate) game_process: HANDLE,
    pub(crate) stop_name: Option<String>,
    pub(crate) show_name: Option<String>,
    pub(crate) log_start_offset: u64,
    pub(crate) desktop: bool,
    pub(crate) startup_watch: bool,
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
    let desktop = values.iter().any(|value| value == "--desktop");
    let startup_watch = values.iter().any(|value| value == "--startup-watch");
    let owner_pid = value("--owner-pid").and_then(|pid| pid.parse::<u32>().ok());
    let game_pid = if desktop {
        owner_pid.unwrap_or_else(|| unsafe { GetCurrentProcessId() })
    } else {
        value("--game-pid").ok_or("missing --game-pid")?.parse()?
    };
    let owner_process = if desktop && owner_pid.is_some() {
        let process = unsafe { OpenProcess(SYNCHRONIZE_ACCESS, 0, game_pid) };
        if process.is_null() {
            return Err(std::io::Error::last_os_error().into());
        }
        process
    } else {
        std::ptr::null_mut()
    };
    let game_process = if desktop {
        std::ptr::null_mut()
    } else {
        let process = unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE_ACCESS,
                0,
                game_pid,
            )
        };
        if process.is_null() {
            return Err(std::io::Error::last_os_error().into());
        }
        process
    };
    let stop_name = if desktop {
        None
    } else {
        Some(value("--stop-event").ok_or("missing --stop-event")?)
    };
    let show_name = value("--show-event");
    let log_start_offset = value("--log-start-offset")
        .and_then(|offset| offset.parse::<u64>().ok())
        .unwrap_or_else(|| {
            shroudforge_loader_config::paths::current_log(&root)
                .metadata()
                .map(|metadata| metadata.len())
                .unwrap_or(0)
        });
    Ok(Arguments {
        root,
        game_pid,
        owner_process,
        game_process,
        stop_name,
        show_name,
        log_start_offset,
        desktop,
        startup_watch,
    })
}
