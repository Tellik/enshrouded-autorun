use crate::Session;

impl Session {
    pub(crate) fn includes_capability(&self, capability: &str) -> bool {
        self.job
            .as_ref()
            .is_some_and(|job| job.capabilities.iter().any(|item| item == capability))
    }

    pub fn enabled(&self, capability: &str) -> bool {
        self.active
            && self.started.elapsed().as_secs() < self.limit()
            && self.includes_capability(capability)
    }

    pub(crate) fn operation_timings_enabled(&self) -> bool {
        self.enabled("operations")
    }
    pub(super) fn limit(&self) -> u64 {
        self.settings["maximumDurationSeconds"]
            .as_u64()
            .unwrap_or(120)
            .clamp(1, 3600)
    }
}
