use crate::Session;

impl Session {
    pub fn enabled(&self, area: &str) -> bool {
        self.active
            && self.started.elapsed().as_secs() < self.limit()
            && self.settings["areas"]
                .as_array()
                .is_some_and(|areas| areas.iter().any(|item| item == area))
    }
    pub(super) fn limit(&self) -> u64 {
        self.settings["maximumDurationSeconds"]
            .as_u64()
            .unwrap_or(120)
            .clamp(1, 3600)
    }
}
