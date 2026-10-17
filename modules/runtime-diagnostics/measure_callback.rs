use crate::Session;
use std::time::Duration;

impl Session {
    pub fn measure(&mut self, area: &str, id: &str, phase: &str, elapsed: Duration, failed: bool) {
        if !self.enabled(area) {
            return;
        }
        let key = format!("{area}/{id}/{phase}");
        if self.measurements.len() >= 1024 && !self.measurements.contains_key(&key) {
            return;
        }
        let entry = self.measurements.entry(key).or_default();
        let ms = elapsed.as_secs_f64() * 1000.0;
        entry.count += 1;
        entry.errors += u64::from(failed);
        entry.total_ms += ms;
        entry.maximum_ms = entry.maximum_ms.max(ms);
        entry.slow += u64::from(
            ms >= self.settings["slowCallbackMilliseconds"]
                .as_f64()
                .unwrap_or(20.0),
        );
    }
}
