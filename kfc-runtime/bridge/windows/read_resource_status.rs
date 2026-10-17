use super::load_native_provider::provider;
use super::read_native_catalog::fetch_json;

pub fn resource_status() -> Option<serde_json::Value> {
    let fetch = provider()?.resource_status?;
    fetch_json(fetch, 1024 * 1024).ok()
}
