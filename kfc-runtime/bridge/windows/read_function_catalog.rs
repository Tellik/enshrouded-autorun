use super::load_native_provider::provider;
use super::read_native_catalog::fetch_json;
use std::sync::OnceLock;

pub fn functions() -> Result<&'static serde_json::Value, String> {
    static CATALOG: OnceLock<serde_json::Value> = OnceLock::new();
    if let Some(catalog) = CATALOG.get() {
        return Ok(catalog);
    }
    let fetch = provider()
        .and_then(|p| p.functions)
        .ok_or("native provider lacks the function inventory extension")?;
    let value = fetch_json(fetch, 32 * 1024 * 1024)?;
    let _ = CATALOG.set(value);
    Ok(CATALOG.get().unwrap())
}
