//! Bounded, opt-in instrumentation hosted by the loader. No scans or game writes.
mod create_diagnostic_session;
// The hyphenated directory is the source boundary for built-in job handlers.
#[path = "diagnostics-jobs/mod.rs"]
mod diagnostics_jobs;
pub mod documents;
mod is_job_capability_enabled;
mod publish_diagnostic_status;
mod read_diagnostic_status;
mod refresh_session_settings;
mod request_diagnostic_action;
mod run_diagnostic_command;
mod start_diagnostic_job;
mod stop_diagnostic_job;
mod tick_diagnostic_session;

pub use create_diagnostic_session::Session;
pub use diagnostics_jobs::catalog as diagnostic_job_catalog;
pub use diagnostics_jobs::operation_timings;
pub use diagnostics_jobs::{DiagnosticJob, DiagnosticJobCatalog};
pub use read_diagnostic_status::status;
pub use request_diagnostic_action::{request, request_job};
pub use run_diagnostic_command::run_module;

pub fn selected_job_id(root: &std::path::Path, settings: &serde_json::Value) -> String {
    diagnostics_jobs::selected_job_id(root, settings)
}

use std::time::{SystemTime, UNIX_EPOCH};

fn seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests;
