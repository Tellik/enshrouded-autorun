pub(crate) fn render_console_document() -> String {
    let styles = format!(
        "{}\n{}",
        include_str!("../../ui-resources/tokens.css"),
        include_str!("styles.css")
    );
    let html = include_str!("index.html")
        .replace("__SHROUDFORGE_STYLES__", &styles)
        .replace("__SHROUDFORGE_SCRIPT__", include_str!("app.ts"));
    html
}
