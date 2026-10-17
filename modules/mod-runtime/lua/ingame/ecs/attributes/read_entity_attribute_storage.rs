use super::Catalog;
use super::resolve_storage;
use crate::env::loader;
use crate::runtime_resolution::attributes::uint;
use serde_json::Value;

pub(super) fn entity_storage(
    catalog: &Catalog,
    descriptor: &Value,
    entity: u32,
) -> Result<(Value, Value, Vec<u8>), String> {
    let _timing =
        shroudforge_runtime_diagnostics::operation_timings::span("attributes.entity_storage");
    let count = catalog.root_entries(uint(&descriptor["root_hash"])?)?.len();
    let candidates = catalog
        .layouts
        .iter()
        .filter_map(|component| {
            if component["storage_size"].as_u64() != Some(count as u64 * 4) {
                return None;
            }
            let name = component["qualified_name"].as_str()?;
            let layout = loader::runtime_provider::resolve(name)?;
            (Some(u64::from(layout.size)) == component["size"].as_u64())
                .then(|| (name.to_owned(), layout.size))
        })
        .collect::<Vec<_>>();
    let snapshots = loader::runtime_provider::read_many(entity, &candidates)?;
    // Every compatible initialized root is still checked. Batching changes
    // owner-resolution cost, not the existing uniqueness/header contract.
    resolve_storage(catalog, descriptor, |component| {
        snapshots
            .get(component["qualified_name"].as_str()?)
            .cloned()
    })
}
