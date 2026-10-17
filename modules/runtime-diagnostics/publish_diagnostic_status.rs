use crate::{Session, seconds};
use serde_json::json;

impl Session {
    pub(super) fn publish(&self) {
        let value = json!({"schemaVersion":1,"pid":std::process::id(),"updatedAt":seconds(),"active":self.active,
            "reason":self.reason,"remainingSeconds":if self.active {self.limit().saturating_sub(self.started.elapsed().as_secs())}else{0},
            "lastSampleAt":self.last_sample,"report":self.last_report.clone()});
        if let Err(error) = shroudforge_loader_config::config::write_document(
            &self.root,
            crate::documents::STATUS,
            &value,
        ) {
            tracing::error!(target:"shroudforge::diagnostics","Status publication failed: {error}");
        }
    }
}
