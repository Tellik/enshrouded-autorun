
pub(crate) fn position_window(
    window: &tao::window::Window,
    saved: &serde_json::Value,
    right: bool,
) {
    if let Some(monitor) = window.current_monitor() {
        let origin = monitor.position();
        let size = monitor.size();
        let extent = window.outer_size();
        let available_x = size.width.saturating_sub(extent.width) as i32;
        let available_y = size.height.saturating_sub(extent.height) as i32;
        let x = saved["x"].as_i64().unwrap_or(
            (origin.x
                + if right {
                    available_x.saturating_sub(20)
                } else {
                    available_x / 2
                }) as i64,
        );
        let y = saved["y"]
            .as_i64()
            .unwrap_or((origin.y + available_y / 2) as i64);
        window.set_outer_position(tao::dpi::PhysicalPosition::new(
            (x as i32).clamp(origin.x, origin.x + available_x),
            (y as i32).clamp(origin.y, origin.y + available_y),
        ));
    }
}
