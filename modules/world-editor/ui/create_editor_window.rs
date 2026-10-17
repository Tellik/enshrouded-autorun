use crate::blueprint_library::ViewState;
use tao::dpi::LogicalSize;
use tao::event_loop::EventLoop;
use tao::platform::windows::WindowBuilderExtWindows;
use tao::window::WindowBuilder;

pub(crate) fn create_editor_window(
    event_loop: &EventLoop<()>,
    initial_state: &ViewState,
) -> Result<tao::window::Window, Box<dyn std::error::Error>> {
    let initial_height = initial_state.panel_height as f64;
    let (icon_rgba, icon_width, icon_height) = shroudforge_ui_resources::rgba()?;
    let icon = tao::window::Icon::from_rgba(icon_rgba, icon_width, icon_height)?;
    let window = WindowBuilder::new()
        .with_window_icon(Some(icon.clone()))
        .with_taskbar_icon(Some(icon))
        .with_title("ShroudForge | World Editor")
        .with_decorations(false)
        .with_always_on_top(true)
        .with_visible(false)
        .with_inner_size(LogicalSize::new(
            initial_state.panel_width as f64,
            initial_height,
        ))
        // Keep the same native style as Modloader UI. Equal min/max bounds
        // preserve a fixed-size overlay without removing WS_SIZEBOX.
        .with_min_inner_size(LogicalSize::new(
            initial_state.panel_width as f64,
            initial_height,
        ))
        .with_max_inner_size(LogicalSize::new(
            initial_state.panel_width as f64,
            initial_height,
        ))
        .build(&event_loop)?;
    Ok(window)
}
