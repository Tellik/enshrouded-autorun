use super::read_console_arguments::Arguments;
use tao::dpi::LogicalSize;
use tao::event_loop::EventLoop;
use tao::platform::windows::WindowBuilderExtWindows;
use tao::window::WindowBuilder;

pub(crate) const DEFAULT_WIDTH: f64 = 960.0;
pub(crate) const DEFAULT_HEIGHT: f64 = 580.0;

pub(crate) fn create_console_window(
    event_loop: &EventLoop<()>,
    arguments: &Arguments,
) -> Result<tao::window::Window, Box<dyn std::error::Error>> {
    let (icon_rgba, icon_width, icon_height) = shroudforge_ui_resources::rgba()?;
    let icon = tao::window::Icon::from_rgba(icon_rgba, icon_width, icon_height)?;
    let window = WindowBuilder::new()
        .with_window_icon(Some(icon.clone()))
        .with_taskbar_icon(Some(icon))
        .with_title("ShroudForge | Debug Console")
        .with_decorations(false)
        .with_always_on_top(false)
        .with_resizable(true)
        .with_visible(arguments.desktop)
        .with_inner_size(LogicalSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))
        .build(&event_loop)?;
    Ok(window)
}
