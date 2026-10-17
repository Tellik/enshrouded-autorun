use crate::{Session, seconds};
use serde_json::{Value, json};

impl Session {
    pub(super) fn sample(&mut self) {
        let mut report = json!({});
        if self.enabled("runtime") || self.enabled("queue") {
            let observed = native_snapshot();
            // The provider owns the runtime report. Preserve its complete
            // schema so profile, registry, world and resource evidence cannot
            // disappear when the provider adds or changes diagnostic fields.
            let mut native = if self.enabled("runtime") {
                observed
            } else {
                json!({"dispatcher":observed["dispatcher"],"error":observed["error"]})
            };
            if let Some(age) = native
                .pointer("/dispatcher/lastDrainAgeMs")
                .and_then(Value::as_u64)
            {
                native["dispatcher"]["drainObservedWithinFiveSeconds"] = json!(age <= 5000);
            }
            if !self.enabled("queue") {
                native
                    .as_object_mut()
                    .map(|object| object.remove("dispatcher"));
            }
            report["native"] = native;
        }
        for (key, entry) in &self.measurements {
            report["measurements"][key] = json!({"count":entry.count,"errors":entry.errors,"totalMs":entry.total_ms,"maximumMs":entry.maximum_ms,"slowCalls":entry.slow});
        }
        // Compare state, not ever-changing ages/counters, to suppress identical health reports.
        let mut signature = report.clone();
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
        if self.settings["onlyChanges"] != true || signature != self.last_report {
            let slow = self
                .measurements
                .values()
                .any(|entry| entry.slow > 0 || entry.errors > 0)
                || report
                    .pointer("/native/error")
                    .is_some_and(|error| !error.is_null());
            if slow {
                tracing::warn!(target:"shroudforge::diagnostics",report=%report,"Diagnostic measurements");
            } else {
                tracing::debug!(target:"shroudforge::diagnostics",report=%report,"Diagnostic measurements");
            }
            self.last_report = signature;
        }
        self.last_sample = seconds();
    }
}

#[cfg(windows)]
fn native_snapshot() -> Value {
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
    let name: Vec<u16> = "kfc-runtime.dll\0".encode_utf16().collect();
    unsafe {
        let module = GetModuleHandleW(name.as_ptr());
        if module.is_null() {
            return json!({"error":"runtime-provider-not-loaded"});
        }
        let Some(symbol) = GetProcAddress(module, c"KfcRuntimeDiagnostics".as_ptr().cast()) else {
            return json!({"error":"runtime-provider-missing-diagnostics-export"});
        };
        let function: unsafe extern "C" fn(*mut u8, usize) = std::mem::transmute(symbol);
        let mut buffer = vec![0u8; 8 << 20];
        function(buffer.as_mut_ptr(), buffer.len());
        let length = buffer
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(buffer.len());
        serde_json::from_slice(&buffer[..length])
            .unwrap_or_else(|error| json!({"error":format!("invalid-provider-report: {error}")}))
    }
}
#[cfg(not(windows))]
fn native_snapshot() -> Value {
    json!({"error":"native-provider-requires-windows"})
}
