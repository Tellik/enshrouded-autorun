use std::path::Path;

pub(crate) fn log_world_editor(root: &Path, level: char, category: &str, message: &str) {
    if let Err(error) = shroudforge_logging::append(
        root,
        level,
        "world-editor",
        &format!("[{category}] {message}"),
    ) {
        eprintln!("World Editor log write failed ({category}): {error}");
    }
}
