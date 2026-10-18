use shroudforge_logging::write_activity;
use shroudforge_player_name::{DetectionReport, DetectionStatus};

pub(crate) fn log_player_name_detection(
    root: &std::path::Path,
    report: &DetectionReport,
    previous_report: &mut Option<DetectionReport>,
) {
    if previous_report.as_ref() == Some(report) {
        return;
    }

    let detected = report.status == DetectionStatus::Detected;
    let details = format!(
        "{}. Source: live Enshrouded UI. Selected name: {}. Process ID: {}.",
        report.status.description(),
        report.name.as_deref().unwrap_or("unavailable"),
        report
            .process_id
            .map(|process_id| process_id.to_string())
            .unwrap_or_else(|| "unavailable".into()),
    );
    write_activity(
        root,
        "Modloader",
        "Player name detection",
        if detected {
            "Character name detected"
        } else {
            "Using configured fallback"
        },
        Some(&details),
        if detected { "info" } else { "warn" },
    );
    *previous_report = Some(report.clone());
}
