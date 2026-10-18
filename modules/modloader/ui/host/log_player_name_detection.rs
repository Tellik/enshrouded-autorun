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
        "{}. Searched {} directories, found {} character indexes, and could not read {} directories.",
        report.status.description(),
        report.directories_scanned,
        report.indexes_found,
        report.inaccessible_directories,
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
