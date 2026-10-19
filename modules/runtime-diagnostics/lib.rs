//! Selectable diagnostic jobs with one catalog, lifecycle and result contract.
mod collectors;
#[path = "session/create_diagnostic_session.rs"]
mod create_diagnostic_session;
#[path = "status/documents.rs"]
pub mod documents;
#[path = "session/is_job_capability_enabled.rs"]
mod is_job_capability_enabled;
#[path = "catalog/load_diagnostic_jobs.rs"]
mod job_catalog;
#[path = "catalog/diagnostic_job.rs"]
mod job_definition;
#[path = "status/publish_diagnostic_status.rs"]
mod publish_diagnostic_status;
#[path = "status/read_diagnostic_status.rs"]
mod read_diagnostic_status;
#[path = "session/refresh_session_settings.rs"]
mod refresh_session_settings;
#[path = "session/request_diagnostic_action.rs"]
mod request_diagnostic_action;
#[path = "session/run_diagnostic_command.rs"]
mod run_diagnostic_command;
#[path = "execution/run_diagnostic_job.rs"]
mod run_diagnostic_job;
#[path = "execution/run_python_diagnostic_job.rs"]
mod run_python_diagnostic_job;
#[path = "session/start_diagnostic_job.rs"]
mod start_diagnostic_job;
#[path = "session/stop_diagnostic_job.rs"]
mod stop_diagnostic_job;
#[path = "session/tick_diagnostic_session.rs"]
mod tick_diagnostic_session;
#[path = "execution/write_job_exports.rs"]
mod write_job_exports;

pub use collectors::operation_timings;
pub use create_diagnostic_session::Session;
pub use job_catalog::catalog as diagnostic_job_catalog;
pub use job_definition::{DiagnosticExecution, DiagnosticJob, DiagnosticJobCatalog};
pub use read_diagnostic_status::status;
pub use request_diagnostic_action::{request, request_job};
pub use run_diagnostic_command::run_module;
pub use run_diagnostic_job::{DiagnosticHost, JobExecutionRequest, JobOutput, RunningJob};
pub use write_job_exports::{JobExports, MAX_EXPORT_BYTES};

fn seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
#[path = "tests/diagnostic_jobs.rs"]
mod tests;
#[cfg(test)]
#[path = "tests/world_editor_jobs.rs"]
mod world_editor_tests;
