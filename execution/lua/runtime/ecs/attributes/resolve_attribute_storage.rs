use super::Catalog;
use super::storage_range;
use crate::env::runtime_metrics;
use crate::runtime_resolution::attributes::uint;
use serde_json::Value;
use serde_json::json;

/// Resolve ownership from the initialized component header on this entity.
/// Type names select engine-registered storage, never infer attribute identity.
pub(super) fn resolve_storage(
    catalog: &Catalog,
    descriptor: &Value,
    mut read: impl FnMut(&Value) -> Option<Vec<u8>>,
) -> Result<(Value, Value, Vec<u8>), String> {
    let _timing = runtime_metrics::span("attributes.resolve_storage");
    let count = catalog.root_entries(uint(&descriptor["root_hash"])?)?.len();
    let mut found = None;
    let mut readable = 0;
    for component in &catalog.layouts {
        if component["storage_size"].as_u64() != Some(count as u64 * 4) {
            continue;
        }
        let Some(bytes) = read(component) else {
            continue;
        };
        readable += 1;
        if storage_range(descriptor, component, &bytes).is_err() {
            continue;
        }
        if found.is_some() {
            return Err(
                "attribute root belongs to multiple initialized components on this entity".into(),
            );
        }
        let mut component = component.clone();
        component["association"] = json!("entity-initialized-root-id-and-reflected-static-array");
        let bound = catalog.bind_storage(descriptor, &component)?;
        found = Some((bound, component, bytes));
    }
    found.ok_or_else(|| format!("no initialized component on this entity matches attribute root {:#010x} ({readable} compatible component snapshots read)", uint(&descriptor["root_hash"]).unwrap()))
}
