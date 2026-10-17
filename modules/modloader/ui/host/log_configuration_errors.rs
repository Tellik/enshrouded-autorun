use super::ui_snapshot_types::Snapshot;
use shroudforge_logging::write_activity;

pub(crate) fn log_configuration_errors(
    root: &std::path::Path,
    snapshot: &Snapshot,
    previous_configuration_errors: &mut std::collections::HashSet<String>,
) {
    let current_errors: std::collections::HashSet<String> = snapshot
        .configuration
        .get("errors")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .map(str::to_owned)
        .collect();
    for error in current_errors.difference(&previous_configuration_errors) {
        write_activity(
            root,
            "Configuration",
            "Configuration issue",
            "Failed",
            Some(error),
            "error",
        );
    }
    for error in previous_configuration_errors.difference(&current_errors) {
        write_activity(
            root,
            "Configuration",
            "Configuration issue resolved",
            "Succeeded",
            Some(error),
            "success",
        );
    }
    *previous_configuration_errors = current_errors;
}
