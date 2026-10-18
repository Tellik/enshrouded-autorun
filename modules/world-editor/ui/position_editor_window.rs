use tao::dpi::LogicalPosition;
use tao::dpi::LogicalSize;
use tao::dpi::PhysicalPosition;
use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect;
use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

pub(crate) fn position_window(
    window: &tao::window::Window,
    game_pid: u32,
    panel_width: f64,
    panel_position_x: i32,
    panel_position_y: u32,
) {
    let physical_width = (panel_width * window.scale_factor()) as i32;
    let scale = window.scale_factor();
    let foreground = unsafe { GetForegroundWindow() };
    if !foreground.is_null() {
        let mut foreground_pid = 0;
        unsafe { GetWindowThreadProcessId(foreground, &mut foreground_pid) };
        let mut rect: RECT = unsafe { std::mem::zeroed() };
        if foreground_pid == game_pid && unsafe { GetWindowRect(foreground, &mut rect) } != 0 {
            let x = if panel_position_x == -1 {
                rect.left + ((rect.right - rect.left) - physical_width) / 2
            } else {
                rect.left + (panel_position_x as f64 * scale) as i32
            };
            let y = rect.top + (panel_position_y as f64 * scale) as i32;
            window.set_outer_position(PhysicalPosition::new(x, y));
            return;
        }
    }
    if let Some(monitor) = window.current_monitor() {
        let origin = monitor.position();
        let size = monitor.size();
        let x = if panel_position_x == -1 {
            origin.x + (size.width as i32 - physical_width) / 2
        } else {
            origin.x + (panel_position_x as f64 * scale) as i32
        };
        let y = origin.y + (panel_position_y as f64 * scale) as i32;
        window.set_outer_position(tao::dpi::PhysicalPosition::new(x, y));
    } else {
        let x = if panel_position_x == -1 {
            0.0
        } else {
            panel_position_x as f64
        };
        window.set_outer_position(LogicalPosition::new(x, panel_position_y as f64));
    }
}

pub(crate) fn set_fixed_size(window: &tao::window::Window, width: u32, height: f64) {
    let size = LogicalSize::new(width as f64, height);
    window.set_min_inner_size(Some(size));
    window.set_max_inner_size(Some(size));
}
