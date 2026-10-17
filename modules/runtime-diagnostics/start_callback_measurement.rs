use crate::Session;
use serde_json::Value;
use std::time::Instant;

impl Session {
    pub(super) fn start_measurement(&mut self) {
        self.started = Instant::now();
        self.next_sample = self.started;
        self.active = true;
        self.reason = "active";
        self.last_report = Value::Null;
        self.measurements.clear();
        tracing::info!(target:"shroudforge::diagnostics",maximum_seconds=self.limit(),"Diagnostic session started");
    }
}
