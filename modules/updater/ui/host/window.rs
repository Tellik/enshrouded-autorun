pub fn window_geometry() -> (&'static str, f64, f64, f64, f64) {
    ("ShroudForge | Update", 640.0, 620.0, 540.0, 480.0)
}

pub fn launch_html(html: &str) -> String {
    html.replace("<head>", "<head><script>window.__shroudforgeLaunchMode='updater';</script>")
}
