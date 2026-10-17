//! Bounded operation timings for the existing opt-in runtime diagnostics.
//!
//! All state belongs to the current loader worker. Spans never acquire engine
//! locks or retain game pointers. A disabled span does not read the clock.
use serde_json::{Value, json};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    marker::PhantomData,
    rc::Rc,
    time::Instant,
};

const MAX_OPERATIONS: usize = 128;
const HISTOGRAM_BUCKETS: usize = 65;

#[derive(Default)]
struct Measurements {
    operations: BTreeMap<&'static str, Measurement>,
    dropped_spans: u64,
}

struct Measurement {
    count: u64,
    total_ns: u64,
    maximum_ns: u64,
    histogram: [u64; HISTOGRAM_BUCKETS],
}

impl Default for Measurement {
    fn default() -> Self {
        Self {
            count: 0,
            total_ns: 0,
            maximum_ns: 0,
            histogram: [0; HISTOGRAM_BUCKETS],
        }
    }
}

impl Measurement {
    fn record(&mut self, ns: u64) {
        self.count = self.count.saturating_add(1);
        self.total_ns = self.total_ns.saturating_add(ns);
        self.maximum_ns = self.maximum_ns.max(ns);
        // Bucket 0 is exactly zero. Bucket n includes [2^(n-1), 2^n-1].
        let bucket = (u64::BITS - ns.leading_zeros()) as usize;
        self.histogram[bucket] = self.histogram[bucket].saturating_add(1);
    }

    fn percentile_upper_bound(&self, percent: u8) -> u64 {
        if self.count == 0 {
            return 0;
        }
        let threshold = ((u128::from(self.count) * u128::from(percent) + 99) / 100) as u64;
        let mut count = 0u64;
        for (bucket, samples) in self.histogram.iter().enumerate() {
            count = count.saturating_add(*samples);
            if count >= threshold {
                return match bucket {
                    0 => 0,
                    64 => u64::MAX,
                    _ => (1u64 << bucket) - 1,
                };
            }
        }
        u64::MAX
    }
}

impl Measurements {
    fn record(&mut self, name: &'static str, ns: u64) {
        if self.operations.len() == MAX_OPERATIONS && !self.operations.contains_key(name) {
            self.dropped_spans = self.dropped_spans.saturating_add(1);
            return;
        }
        self.operations.entry(name).or_default().record(ns);
    }
}

thread_local! {
    static ENABLED: Cell<bool> = const { Cell::new(false) };
    static GENERATION: Cell<u64> = const { Cell::new(0) };
    static MEASUREMENTS: RefCell<Measurements> = RefCell::new(Measurements::default());
}

/// Start a fresh session only on a disabled-to-enabled transition. Stopping
/// retains its completed measurements for status publication.
pub fn set_enabled(enabled: bool) {
    let was_enabled = ENABLED.replace(enabled);
    if enabled && !was_enabled {
        GENERATION.set(GENERATION.get().wrapping_add(1));
        MEASUREMENTS.with(|measurements| *measurements.borrow_mut() = Measurements::default());
    }
}

/// Thread-bound so dropping a span on another worker cannot mix measurements.
pub struct Span {
    name: &'static str,
    started: Option<Instant>,
    generation: u64,
    _worker: PhantomData<Rc<()>>,
}

#[inline]
pub fn span(name: &'static str) -> Span {
    if !ENABLED.get() {
        return Span {
            name,
            started: None,
            generation: 0,
            _worker: PhantomData,
        };
    }
    Span {
        name,
        started: Some(Instant::now()),
        generation: GENERATION.get(),
        _worker: PhantomData,
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        let Some(started) = self.started else {
            return;
        };
        if !ENABLED.get() || GENERATION.get() != self.generation {
            return;
        }
        let ns = started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
        MEASUREMENTS.with(|measurements| measurements.borrow_mut().record(self.name, ns));
    }
}

pub fn snapshot() -> Value {
    MEASUREMENTS.with(|measurements| {
        let measurements = measurements.borrow();
        let operations: BTreeMap<_, _> = measurements
            .operations
            .iter()
            .map(|(name, entry)| {
                (
                    *name,
                    json!({
                        "count":entry.count,"totalNs":entry.total_ns,"maximumNs":entry.maximum_ns,
                        "p50UpperBoundNs":entry.percentile_upper_bound(50),
                        "p95UpperBoundNs":entry.percentile_upper_bound(95),
                        "p99UpperBoundNs":entry.percentile_upper_bound(99)
                    }),
                )
            })
            .collect();
        json!({"enabled":ENABLED.get(),"session":GENERATION.get(),
            "maximumOperations":MAX_OPERATIONS,"operationCount":operations.len(),
            "droppedSpans":measurements.dropped_spans,
            "percentiles":"power-of-two nanosecond bucket upper bounds",
            "nestedSpans":"inclusive timings, do not sum nested operations",
            "operations":operations})
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn histogram_bounds_and_saturated_totals_are_explicit() {
        let mut measurement = Measurement::default();
        for ns in [0, 1, 2, 3, 4] {
            measurement.record(ns);
        }
        assert_eq!(measurement.count, 5);
        assert_eq!(measurement.total_ns, 10);
        assert_eq!(measurement.maximum_ns, 4);
        assert_eq!(measurement.percentile_upper_bound(50), 3);
        assert_eq!(measurement.percentile_upper_bound(95), 7);
        measurement.record(u64::MAX);
        assert_eq!(measurement.total_ns, u64::MAX);
        assert_eq!(measurement.percentile_upper_bound(99), u64::MAX);
    }

    #[test]
    fn disabled_spans_and_old_session_spans_do_not_record() {
        set_enabled(false);
        let disabled = span("disabled");
        assert!(disabled.started.is_none());
        drop(disabled);
        set_enabled(true);
        let old = span("old-session");
        set_enabled(false);
        set_enabled(true);
        drop(old);
        drop(span("current-session"));
        let current = snapshot();
        assert_eq!(current["operationCount"], 1);
        assert_eq!(current["operations"]["current-session"]["count"], 1);
        set_enabled(false);
        assert_eq!(snapshot()["operationCount"], 1);
    }
}
