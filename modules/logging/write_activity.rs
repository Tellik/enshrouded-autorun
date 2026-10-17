use crate::*;

pub fn write_activity(
    root: &Path,
    source: &str,
    action: &str,
    result: &str,
    details: Option<&str>,
    level: &str,
) {
    let severity = match level {
        "warn" => 'W',
        "error" => 'E',
        "debug" => 'D',
        "trace" => 'T',
        level
            if matches!(level, "info" | "success")
                && matches!(
                    action,
                    "WebView rendered" | "Search catalog" | "Save language" | "Open external link"
                ) =>
        {
            'D'
        }
        _ => 'I',
    };
    let mut message = format!("{action}: {result}");
    if let Some(details) = details {
        message.push_str(" — ");
        message.push_str(details);
    }
    let _ = crate::append(root, severity, source, &message);
}
