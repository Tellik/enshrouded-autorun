use crate::windows::provider::{CATALOG_ERRORS, fetch_json, provider};

pub fn registry() -> Result<serde_json::Value, String> {
    let _span =
        shroudforge_runtime_diagnostics::operation_timings::span("native.ecs.registry_catalog");
    let fetch = provider().and_then(|p| p.registry).ok_or(
        "Native provider lacks the component registry extension. Install the matching provider",
    )?;
    fetch_json(
        |buffer, capacity| unsafe { fetch(buffer, capacity) },
        8 * 1024 * 1024,
        CATALOG_ERRORS,
    )
}
