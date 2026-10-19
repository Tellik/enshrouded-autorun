use crate::Session;
use serde_json::{Value, json};
use std::{collections::BTreeMap, time::Duration};

#[derive(Default)]
pub(crate) struct Measurement {
    pub(super) count: u64,
    pub(super) errors: u64,
    pub(super) total_ms: f64,
    pub(super) maximum_ms: f64,
    pub(super) slow: u64,
}

impl Session {
    pub fn measure(
        &mut self,
        capability: &str,
        id: &str,
        phase: &str,
        elapsed: Duration,
        failed: bool,
    ) {
        if !self.enabled(capability) {
            return;
        }
        let key = format!("{capability}/{id}/{phase}");
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

pub(crate) fn report(measurements: &BTreeMap<String, Measurement>) -> Value {
    let mut report = json!({});
    for (key, entry) in measurements {
        report[key] = json!({"count":entry.count,"errors":entry.errors,"totalMs":entry.total_ms,"maximumMs":entry.maximum_ms,"slowCalls":entry.slow});
    }
    report
}

pub(crate) fn failed(report: &Value) -> bool {
    report.as_object().is_some_and(|entries| {
        entries.values().any(|entry| {
            entry["errors"].as_u64().unwrap_or(0) > 0
                || entry["slowCalls"].as_u64().unwrap_or(0) > 0
        })
    })
}

pub(crate) fn enabled(session: &Session) -> bool {
    session.includes_capability("mods")
}
