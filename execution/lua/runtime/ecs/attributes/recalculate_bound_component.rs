use super::Catalog;
use super::storage_range;
use super::validate_attribute_bits;
use crate::env::runtime_metrics;
use crate::runtime_resolution::attributes::read_header;
use crate::runtime_resolution::attributes::uint;
use serde_json::Value;

// Owned-data core shared by raw component writes and both named attribute writers.
// A failure preserves the caller's candidate and mask. Original programs may
// update additional scalars, so masks are rebuilt only after full validation.
pub(super) fn recalculate_bound_component_update(
    catalog: &Catalog,
    name: &str,
    before: &[u8],
    after: &mut [u8],
    mask: &mut [u8],
    calculation_model: &str,
) -> Result<Vec<Value>, String> {
    let _timing = runtime_metrics::span("attributes.recalculate_component");
    if calculation_model != "attribute-command-v1" {
        return Err("attribute calculations are not validated for this executable profile".into());
    }
    if before.len() != after.len()
        || before.len() != mask.len()
        || mask.iter().any(|byte| *byte > 1)
    {
        return Err("attribute component candidate or mask size is invalid".into());
    }
    let mut layouts = catalog
        .layouts
        .iter()
        .filter(|layout| layout["qualified_name"] == name);
    let component = layouts
        .next()
        .ok_or("attribute component has no verified inline storage layout")?;
    if layouts.next().is_some() {
        return Err("attribute component has multiple reflected storage layouts".into());
    }
    let root = read_header(&component["header"], "rootId", before)?;
    let entries = catalog.root_entries(root)?;
    storage_range(entries[0], component, before)?;
    let start = uint(&component["storage_offset"])? as usize;
    let length = uint(&component["storage_size"])? as usize;
    let end = start
        .checked_add(length)
        .ok_or("attribute storage range overflow")?;
    if length
        != entries
            .len()
            .checked_mul(4)
            .ok_or("attribute storage count overflow")?
        || end > before.len()
    {
        return Err("attribute root storage count changed".into());
    }
    for index in 0..before.len() {
        if !(start..end).contains(&index) && (before[index] != after[index] || mask[index] != 0) {
            return Err(
                "attribute headers, definition identity and storage layout are read-only".into(),
            );
        }
        if before[index] != after[index] && mask[index] == 0 {
            return Err("attribute candidate changes bytes outside its selected write mask".into());
        }
    }
    let scalar = component["storage_scalar_type"]
        .as_str()
        .ok_or("unresolved attribute scalar type")?;
    let values: Vec<_> = after[start..end]
        .chunks_exact(4)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
        .collect();
    let mut bound = Vec::with_capacity(entries.len());
    for (entry, bits) in entries.iter().zip(&values) {
        let descriptor = catalog.bind_storage(entry, component)?;
        if descriptor["storage_writable"] != true || descriptor["calculation_writable"] != true {
            return Err(descriptor["calculation_reason"]
                .as_str()
                .or_else(|| descriptor["storage_reason"].as_str())
                .unwrap_or("attribute storage or calculation is not writable")
                .into());
        }
        if descriptor["calculation_encoding_verified"] != true
            || descriptor["calculation"]["decoded"] != true
        {
            return Err("attribute root has an unverified calculation program".into());
        }
        validate_attribute_bits(&descriptor, scalar, *bits)?;
        bound.push(descriptor);
    }
    let (recalculated, trace) = catalog.recalculate(root, &values)?;
    for (descriptor, bits) in bound.iter().zip(&recalculated) {
        validate_attribute_bits(descriptor, scalar, *bits)?;
    }
    mask.fill(0);
    for (index, value) in recalculated.iter().enumerate() {
        let range = start + index * 4..start + (index + 1) * 4;
        after[range.clone()].copy_from_slice(&value.to_le_bytes());
        if after[range.clone()] != before[range.clone()] {
            mask[range].fill(1);
        }
    }
    Ok(trace)
}
