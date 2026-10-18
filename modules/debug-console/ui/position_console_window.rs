use super::create_console_window::DEFAULT_HEIGHT;
use super::create_console_window::DEFAULT_WIDTH;
use serde_json::Value;
use tao::window::Window;

pub(crate) fn position_window(window: &Window, saved: &Value, right: bool) -> Value {
    shroudforge_window_management::apply_window_geometry(
        window,
        saved,
        right,
        shroudforge_window_management::WindowDefaults {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
        },
    )
}
