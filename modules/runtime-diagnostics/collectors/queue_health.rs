use serde_json::{Value, json};

pub(crate) fn project(snapshot: &Value) -> Value {
    json!({"dispatcher":snapshot["dispatcher"],"error":snapshot["error"]})
}

pub(crate) fn annotate_drain_age(snapshot: &mut Value) {
    if let Some(age) = snapshot
        .pointer("/dispatcher/lastDrainAgeMs")
        .and_then(Value::as_u64)
    {
        snapshot["dispatcher"]["drainObservedWithinFiveSeconds"] = json!(age <= 5000);
    }
}

pub(crate) fn suppress_unchanging_counters(signature: &mut Value) {
    if let Some(dispatcher) = signature
        .pointer_mut("/native/dispatcher")
        .and_then(Value::as_object_mut)
    {
        for key in [
            "lastDrainAgeMs",
            "drainCount",
            "completed",
            "lastManagerObservationAgeMs",
        ] {
            dispatcher.remove(key);
        }
    }
}
