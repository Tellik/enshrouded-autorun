use crate::*;

pub(crate) fn configured_level(root: &Path) -> LevelFilter {
    let Ok(value) = shroudforge_loader_config::config::read_loader(root) else {
        return LevelFilter::INFO;
    };
    match value
        .pointer("/logging/minimumLevel")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("INFO")
        .to_ascii_uppercase()
        .as_str()
    {
        "ALL" | "TRACE" => LevelFilter::TRACE,
        "DEBUG" => LevelFilter::DEBUG,
        "WARNING" | "WARN" => LevelFilter::WARN,
        "ERROR" => LevelFilter::ERROR,
        _ => LevelFilter::INFO,
    }
}

pub fn allows(root: &Path, level: char) -> bool {
    let level = match level {
        'T' => LevelFilter::TRACE,
        'D' => LevelFilter::DEBUG,
        'W' => LevelFilter::WARN,
        'E' => LevelFilter::ERROR,
        _ => LevelFilter::INFO,
    };
    level <= configured_level(root)
}
