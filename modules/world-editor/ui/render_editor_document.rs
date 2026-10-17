use base64::Engine as _;

pub(crate) fn render_editor_document() -> String {
    let styles = format!(
        "{}\n{}",
        include_str!("../../ui-resources/tokens.css"),
        include_str!("styles.css")
    );
    let icon =
        base64::engine::general_purpose::STANDARD.encode(include_bytes!("../assets/icon.svg"));
    let html = include_str!("index.html")
        .replace("__SHROUDFORGE_STYLES__", &styles)
        .replace(
            "__WORLD_EDITOR_ICON__",
            &format!("data:image/svg+xml;base64,{icon}"),
        );
    html
}
