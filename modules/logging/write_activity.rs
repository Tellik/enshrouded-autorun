use crate::*;

pub fn activity_log_level(action: &str, outcome: &str) -> LogLevel {
    match outcome {
        "warn" => LogLevel::Warn,
        "error" => LogLevel::Error,
        "debug" => LogLevel::Debug,
        "trace" => LogLevel::Trace,
        outcome
            if matches!(outcome, "info" | "success")
                && matches!(
                    action,
                    "WebView rendered" | "Search catalog" | "Save language" | "Open external link"
                ) =>
        {
            LogLevel::Debug
        }
        _ => LogLevel::Info,
    }
}

pub fn write_activity(
    root: &Path,
    source: &str,
    action: &str,
    result: &str,
    details: Option<&str>,
    outcome: &str,
) {
    let level = activity_log_level(action, outcome);
    write_activity_at_level(root, source, action, result, details, level);
}

pub fn write_activity_at_level(
    root: &Path,
    source: &str,
    action: &str,
    result: &str,
    details: Option<&str>,
    level: LogLevel,
) {
    let mut message = format!("{action}: {result}");
    if let Some(details) = details {
        message.push_str(" — ");
        message.push_str(details);
    }
    let _ = crate::append(root, level.as_char(), source, &message);
}
