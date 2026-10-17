use crate::Session;
use crate::diagnostics_jobs::resolve;
use serde_json::Value;
use std::time::Instant;

impl Session {
    pub(super) fn start_diagnostic_job(&mut self) {
        let job = match resolve(&self.root, &self.settings, None) {
            Ok(job) => job,
            Err(error) => {
                self.active = false;
                self.job = None;
                self.reason = "job-unavailable";
                if error != self.last_error {
                    tracing::error!(target:"shroudforge::diagnostics",%error,"Diagnostic job could not start");
                    self.last_error = error;
                }
                return;
            }
        };
        self.started = Instant::now();
        self.next_sample = self.started;
        self.active = true;
        self.reason = "active";
        let collects_operations = job
            .capabilities
            .iter()
            .any(|capability| capability == "operations");
        self.job = Some(job);
        if collects_operations {
            crate::diagnostics_jobs::operation_timings::reset_session();
        }
        self.last_report = Value::Null;
        self.last_signature = Value::Null;
        self.measurements.clear();
        tracing::info!(target:"shroudforge::diagnostics",maximum_seconds=self.limit(),"Diagnostic session started");
    }
}
