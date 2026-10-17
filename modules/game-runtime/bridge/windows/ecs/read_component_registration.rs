use super::registry;
use crate::windows::provider::{REGISTRATION_ERRORS, fetch_json, provider};
use std::ffi::CString;

pub fn registration(name: &str) -> Result<serde_json::Value, String> {
    let _span = shroudforge_runtime_diagnostics::operation_timings::span("native.ecs.registration");
    let Some(fetch) = provider().and_then(|provider| provider.registration) else {
        return registry();
    };
    let name = CString::new(name).map_err(|_| "invalid registration name")?;
    fetch_json(
        |buffer, capacity| unsafe { fetch(name.as_ptr(), buffer, capacity) },
        1024 * 1024,
        REGISTRATION_ERRORS,
    )
}
