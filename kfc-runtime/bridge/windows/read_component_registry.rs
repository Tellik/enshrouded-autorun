use super::load_native_provider::provider;
use super::read_native_catalog::fetch_json;

pub fn registry() -> Result<serde_json::Value, String> {
    let _span = shroudforge_runtime_diagnostics::measure_runtime_operations::span(
        "native.ecs.registry_catalog",
    );
    let fetch = provider().and_then(|p| p.registry).ok_or(
        "Native provider lacks the component registry extension. Install the matching provider",
    )?;
    fetch_json(fetch, 8 * 1024 * 1024)
}
