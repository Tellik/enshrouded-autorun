use crate::windows::provider::{CATALOG_ERRORS, fetch_json, provider};

pub fn resource_status() -> Option<serde_json::Value> {
    let fetch = provider()?.resource_status?;
    fetch_json(
        |buffer, capacity| unsafe { fetch(buffer, capacity) },
        1024 * 1024,
        CATALOG_ERRORS,
    )
    .ok()
}
