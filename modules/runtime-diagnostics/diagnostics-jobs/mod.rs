pub(super) mod capture;
mod catalog;
mod definition;
pub(super) mod mod_callbacks;
pub mod operation_timings;
pub(super) mod queue_health;
pub(super) mod runtime_health;

pub use definition::{DiagnosticJob, DiagnosticJobCatalog};

use serde_json::Value;
use std::path::Path;

pub fn catalog(root: &Path, settings: &Value) -> DiagnosticJobCatalog {
    catalog::discover(root, settings["jobId"].as_str())
}

pub fn selected_job_id(root: &Path, settings: &Value) -> String {
    catalog(root, settings).selected_job_id
}

pub(super) fn resolve(
    root: &Path,
    settings: &Value,
    id: Option<&str>,
) -> Result<DiagnosticJob, String> {
    catalog::resolve(root, settings, id)
}
