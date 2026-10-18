use serde_json::Value;
use std::path::Path;
use tao::dpi::LogicalSize;
use tao::dpi::PhysicalPosition;
use tao::platform::windows::WindowExtWindows;
use tao::window::Window;
use windows_sys::Win32::Graphics::Gdi::GetMonitorInfoW;
use windows_sys::Win32::Graphics::Gdi::MONITOR_DEFAULTTONEAREST;
use windows_sys::Win32::Graphics::Gdi::MONITORINFO;
use windows_sys::Win32::Graphics::Gdi::MonitorFromWindow;

#[derive(Clone, Copy)]
pub struct WindowDefaults {
    pub width: f64,
    pub height: f64,
    pub minimum_width: f64,
    pub minimum_height: f64,
}

#[derive(Clone, Copy)]
struct WindowSize {
    width: f64,
    height: f64,
}

fn fit_window_size(
    requested: WindowSize,
    minimum: WindowSize,
    available: WindowSize,
) -> (WindowSize, WindowSize) {
    let requested = WindowSize {
        width: requested.width.max(1.0),
        height: requested.height.max(1.0),
    };
    let scale = (available.width.max(1.0) / requested.width)
        .min(available.height.max(1.0) / requested.height)
        .min(1.0);
    let size = WindowSize {
        width: requested.width * scale,
        height: requested.height * scale,
    };
    let minimum = WindowSize {
        width: minimum.width.clamp(1.0, requested.width) * scale,
        height: minimum.height.clamp(1.0, requested.height) * scale,
    };
    (size, minimum)
}

pub fn apply_window_geometry(
    window: &Window,
    saved: &Value,
    right: bool,
    defaults: WindowDefaults,
) -> Value {
    let saved_x = saved["x"]
        .as_i64()
        .and_then(|value| i32::try_from(value).ok());
    let saved_y = saved["y"]
        .as_i64()
        .and_then(|value| i32::try_from(value).ok());
    if let (Some(x), Some(y)) = (saved_x, saved_y) {
        window.set_outer_position(PhysicalPosition::new(x, y));
    }

    let Some(monitor) = window
        .current_monitor()
        .or_else(|| window.primary_monitor())
    else {
        return saved.clone();
    };
    let scale = monitor.scale_factor().max(1.0);
    let monitor_position = monitor.position();
    let monitor_size = monitor.size();
    let work_area = monitor_work_area(window).unwrap_or((
        monitor_position.x,
        monitor_position.y,
        monitor_size.width,
        monitor_size.height,
    ));
    let available = WindowSize {
        width: work_area.2 as f64 / scale,
        height: work_area.3 as f64 / scale,
    };
    let requested = WindowSize {
        width: positive_size(saved["width"].as_f64()).unwrap_or(defaults.width),
        height: positive_size(saved["height"].as_f64()).unwrap_or(defaults.height),
    };
    let minimum = WindowSize {
        width: defaults.minimum_width,
        height: defaults.minimum_height,
    };
    let (size, minimum) = fit_window_size(requested, minimum, available);
    window.set_min_inner_size(Some(LogicalSize::new(minimum.width, minimum.height)));
    window.set_inner_size(LogicalSize::new(size.width, size.height));

    let width = (size.width * scale).ceil() as u32;
    let height = (size.height * scale).ceil() as u32;
    let available_x = work_area.2.saturating_sub(width) as i32;
    let available_y = work_area.3.saturating_sub(height) as i32;
    let x = saved_x.unwrap_or_else(|| {
        work_area.0
            + if right {
                available_x.saturating_sub(20)
            } else {
                available_x / 2
            }
    });
    let y = saved_y.unwrap_or(work_area.1 + available_y / 2);
    let x = x.clamp(work_area.0, work_area.0 + available_x);
    let y = y.clamp(work_area.1, work_area.1 + available_y);
    window.set_outer_position(PhysicalPosition::new(x, y));

    geometry_value(x, y, size)
}

pub fn current_window_geometry(window: &Window) -> Option<Value> {
    let position = window.outer_position().ok()?;
    let size = window.inner_size().to_logical::<f64>(window.scale_factor());
    Some(geometry_value(
        position.x,
        position.y,
        WindowSize {
            width: size.width,
            height: size.height,
        },
    ))
}

pub fn save_window_geometry(
    root: &Path,
    module: &str,
    window: &Window,
) -> Result<Option<Value>, String> {
    if window.is_maximized() || window.is_minimized() {
        return Ok(None);
    }
    let Some(geometry) = current_window_geometry(window) else {
        return Ok(None);
    };
    shroudforge_loader_config::config::update_loader(root, |value| {
        value["modules"][module]["window"]["position"] = geometry.clone();
        Ok(())
    })?;
    Ok(Some(geometry))
}

fn geometry_value(x: i32, y: i32, size: WindowSize) -> Value {
    serde_json::json!({
        "x": x,
        "y": y,
        "width": size.width.round() as u64,
        "height": size.height.round() as u64
    })
}

fn positive_size(value: Option<f64>) -> Option<f64> {
    value.filter(|value| value.is_finite() && *value > 0.0)
}

fn monitor_work_area(window: &Window) -> Option<(i32, i32, u32, u32)> {
    let monitor = unsafe { MonitorFromWindow(window.hwnd() as _, MONITOR_DEFAULTTONEAREST) };
    if monitor.is_null() {
        return None;
    }
    let mut info: MONITORINFO = unsafe { std::mem::zeroed() };
    info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
    if unsafe { GetMonitorInfoW(monitor, &mut info) } == 0 {
        return None;
    }
    Some((
        info.rcWork.left,
        info.rcWork.top,
        (info.rcWork.right - info.rcWork.left).max(1) as u32,
        (info.rcWork.bottom - info.rcWork.top).max(1) as u32,
    ))
}
