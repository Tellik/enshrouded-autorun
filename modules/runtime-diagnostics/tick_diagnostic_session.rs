use crate::Session;
use std::time::{Duration, Instant};

impl Session {
    pub fn tick(&mut self) {
        self.refresh();
        crate::diagnostics_jobs::operation_timings::set_enabled(self.operation_timings_enabled());
        let now = Instant::now();
        if now < self.next_sample
            && !(self.active && self.started.elapsed().as_secs() >= self.limit())
        {
            return;
        }
        self.next_sample = now
            + Duration::from_millis(
                self.settings["intervalMilliseconds"]
                    .as_u64()
                    .unwrap_or(2000)
                    .clamp(500, 60000),
            );
        if self.active {
            self.sample();
            if self.started.elapsed().as_secs() >= self.limit()
                || self.settings["continuous"] != true
            {
                self.complete_diagnostic_job();
            }
        }
        crate::diagnostics_jobs::operation_timings::set_enabled(self.operation_timings_enabled());
        self.publish();
    }
}
