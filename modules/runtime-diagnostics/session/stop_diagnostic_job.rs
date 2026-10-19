use crate::Session;
use serde_json::json;
use std::time::Instant;

impl Session {
    pub(super) fn stop_diagnostic_job(&mut self) {
        self.stop_job_execution("stopped");
        self.sample();
        self.active = false;
        self.reason = "stopped";
        self.next_sample = Instant::now();
        tracing::info!(target:"shroudforge::diagnostics","Diagnostic session stopped");
    }

    pub(super) fn complete_diagnostic_job(&mut self) {
        let reason = if self.settings["continuous"] == true
            || self.job.as_ref().is_some_and(|job| job.execution.is_some())
        {
            "duration-complete"
        } else {
            "snapshot-complete"
        };
        self.finish_diagnostic_job(reason);
    }

    pub(crate) fn finish_diagnostic_job(&mut self, reason: &'static str) {
        self.stop_job_execution(reason);
        self.sample();
        self.active = false;
        self.reason = reason;
        self.next_sample = Instant::now();
        tracing::info!(target:"shroudforge::diagnostics",reason=self.reason,"Diagnostic session completed");
        // Keep the same persisted source of truth; do not overwrite a newer UI request.
        let request = self.settings["requestId"].clone();
        if let Err(error) = shroudforge_loader_config::config::update_loader(&self.root, |value| {
            if value["modules"]["runtimeDiagnostics"]
                .get("requestId")
                .cloned()
                .unwrap_or(json!(0))
                == request
            {
                value["modules"]["runtimeDiagnostics"]["sessionActive"] = false.into();
            }
            Ok(())
        }) {
            tracing::error!(target:"shroudforge::diagnostics","Could not persist completed session: {error}");
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.stop_job_execution("process-stopped");
        if self.active {
            self.sample();
        }
        self.active = false;
        self.reason = "process-stopped";
        self.publish();
    }
}
