//! Lua attribute bindings. Resolution lives in runtime_resolution.
use super::{AppState, loader, registry, runtime_metrics};
pub(crate) use crate::runtime_resolution::attributes::Catalog;
use crate::runtime_resolution::attributes::{read_header, uint};
use kfc::guid::Guid;
use kfc::reflection::{LookupKey, TypeRegistry};
use mlua::{Lua, Table, Value as LuaValue};
use mod_loader::Mod;
use serde_json::{Value, json};

impl Catalog {
    pub(crate) fn get(&self, selector: &LuaValue) -> Result<&Value, String> {
        match selector {
            LuaValue::Integer(id) => self
                .entries
                .get(&u32::try_from(*id).map_err(|_| "attribute ID outside u32")?)
                .ok_or_else(|| "attribute ID not found".into()),
            LuaValue::String(name) => {
                let name = name.to_str().map_err(|e| e.to_string())?;
                self.by_name(name.as_ref())
            }
            _ => Err("expected original attribute name or unsigned attribute ID".into()),
        }
    }
}

/// Work on an owned snapshot. Reject relocated/external storage and mismatched
/// definitions instead of dereferencing any engine pointer from Lua.
fn storage_range(
    descriptor: &Value,
    component: &Value,
    bytes: &[u8],
) -> Result<std::ops::Range<usize>, String> {
    if bytes.len() != uint(&component["size"])? as usize {
        return Err("attribute component size changed".into());
    }
    let header = &component["header"];
    if read_header(header, "rootId", bytes)? != uint(&descriptor["root_hash"])? {
        return Err("live attribute root ID differs from KFC definition".into());
    }
    if !component["definition_offset"].is_null() {
        let definition = uint(&component["definition_offset"])? as usize;
        let expected = Guid::parse(
            component["definition_guid"]
                .as_str()
                .ok_or("missing GUID")?,
        )
        .ok_or("invalid GUID")?;
        if bytes.get(definition..definition + 16) != Some(expected.data().as_slice()) {
            return Err("live attribute definition differs from KFC definition".into());
        }
    }
    let offset = read_header(header, "storageOffset", bytes)? as usize;
    // Both storageSize (KFC3) and storageCount (KFC4) count elements.
    let count = read_header(header, "storageCount", bytes)? as usize;
    let flags = read_header(header, "flags", bytes)?;
    // Current KFC4 native adders independently confirm the reflected bit with
    // test byte [component+6],1 at 0x21bdf5. Future layouts use their own enum.
    let initialized = uint(&header["flags"]["initialized_mask"])?;
    if initialized == 0
        || flags & initialized == 0
        || offset != uint(&component["storage_offset"])? as usize
        || count.checked_mul(4) != Some(uint(&component["storage_size"])? as usize)
    {
        return Err("attribute is uninitialized or uses an unverified storage layout".into());
    }
    let start = offset
        .checked_add(uint(&descriptor["index"])? as usize * 4)
        .ok_or("attribute offset overflow")?;
    if start + 4 > bytes.len() || start + 4 > offset + count * 4 {
        return Err("attribute element outside storage".into());
    }
    Ok(start..start + 4)
}

/// Resolve ownership from the initialized component header on this entity.
/// Type names select engine-registered storage, never infer attribute identity.
fn resolve_storage(
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

fn entity_storage(
    catalog: &Catalog,
    descriptor: &Value,
    entity: u32,
) -> Result<(Value, Value, Vec<u8>), String> {
    let _timing = runtime_metrics::span("attributes.entity_storage");
    let count = catalog.root_entries(uint(&descriptor["root_hash"])?)?.len();
    let candidates = catalog.layouts.iter().filter_map(|component| {
        if component["storage_size"].as_u64() != Some(count as u64 * 4) { return None; }
        let name = component["qualified_name"].as_str()?;
        let layout = loader::runtime_provider::resolve(name)?;
        (Some(u64::from(layout.size)) == component["size"].as_u64())
            .then(|| (name.to_owned(), layout.size))
    }).collect::<Vec<_>>();
    let snapshots = loader::runtime_provider::read_many(entity, &candidates)?;
    // Every compatible initialized root is still checked. Batching changes
    // owner-resolution cost, not the existing uniqueness/header contract.
    resolve_storage(catalog, descriptor, |component| {
        snapshots.get(component["qualified_name"].as_str()?).cloned()
    })
}

fn is_attribute_component(registry: &TypeRegistry, name: &str) -> Result<bool, String> {
    let metadata = registry
        .get_by_name(LookupKey::Qualified(name))
        .ok_or_else(|| format!("unresolved component type: {name}"))?;
    Ok(crate::runtime_resolution::types::inheritance(registry, metadata.index)?
        .iter()
        .any(|index| registry.get(*index).is_some_and(|base| {
            base.qualified_name == "keen::ecs::AttributeComponent"
        })))
}

fn validate_attribute_bits(descriptor: &Value, scalar: &str, bits: u32) -> Result<(), String> {
    let value = match scalar {
        "keen::sint32" => LuaValue::Integer(i64::from(bits as i32)),
        "keen::uint32" => LuaValue::Integer(i64::from(bits)),
        "keen::float32" => LuaValue::Number(f64::from(f32::from_bits(bits))),
        _ => return Err("unsupported attribute scalar".into()),
    };
    encode_attribute_scalar(descriptor, scalar, &value).map(|_| ())
}

// Owned-data core shared by raw component writes and both named attribute writers.
// A failure preserves the caller's candidate and mask. Original programs may
// update additional scalars, so masks are rebuilt only after full validation.
fn recalculate_bound_component_update(
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
    if before.len() != after.len() || before.len() != mask.len()
        || mask.iter().any(|byte| *byte > 1)
    {
        return Err("attribute component candidate or mask size is invalid".into());
    }
    let mut layouts = catalog.layouts.iter().filter(|layout| layout["qualified_name"] == name);
    let component = layouts.next().ok_or("attribute component has no verified inline storage layout")?;
    if layouts.next().is_some() {
        return Err("attribute component has multiple reflected storage layouts".into());
    }
    let root = read_header(&component["header"], "rootId", before)?;
    let entries = catalog.root_entries(root)?;
    storage_range(entries[0], component, before)?;
    let start = uint(&component["storage_offset"])? as usize;
    let length = uint(&component["storage_size"])? as usize;
    let end = start.checked_add(length).ok_or("attribute storage range overflow")?;
    if length != entries.len().checked_mul(4).ok_or("attribute storage count overflow")?
        || end > before.len()
    {
        return Err("attribute root storage count changed".into());
    }
    for index in 0..before.len() {
        if !(start..end).contains(&index) && (before[index] != after[index] || mask[index] != 0) {
            return Err("attribute headers, definition identity and storage layout are read-only".into());
        }
        if before[index] != after[index] && mask[index] == 0 {
            return Err("attribute candidate changes bytes outside its selected write mask".into());
        }
    }
    let scalar = component["storage_scalar_type"].as_str().ok_or("unresolved attribute scalar type")?;
    let values: Vec<_> = after[start..end].chunks_exact(4)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap())).collect();
    let mut bound = Vec::with_capacity(entries.len());
    for (entry, bits) in entries.iter().zip(&values) {
        let descriptor = catalog.bind_storage(entry, component)?;
        if descriptor["storage_writable"] != true || descriptor["calculation_writable"] != true {
            return Err(descriptor["calculation_reason"].as_str()
                .or_else(|| descriptor["storage_reason"].as_str())
                .unwrap_or("attribute storage or calculation is not writable").into());
        }
        if descriptor["calculation_encoding_verified"] != true || descriptor["calculation"]["decoded"] != true {
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

/// Generic `ecs.write` and `ecs.write_fields` preserve AttributeComponent
/// headers and execute the original initialized root's full calculation before
/// the native expected-snapshot CAS. Other component types keep their codec.
pub(crate) fn recalculate_component_update(
    state: &AppState,
    name: &str,
    before: &[u8],
    after: &mut [u8],
    mask: &mut [u8],
) -> Result<(), String> {
    if !is_attribute_component(state.type_registry(), name)? {
        return Ok(());
    }
    let catalog = state.attribute_catalog()?;
    let functions = loader::runtime_provider::functions()?;
    recalculate_bound_component_update(catalog, name, before, after, mask,
        functions["attribute_calculation_model"].as_str().unwrap_or("")).map(|_| ())
}

/// Bind decisions from an earlier Lua read to this fresh root snapshot. The
/// native compare-exchange then guards the complete component until mutation.
fn expect_root_values(
    catalog: &Catalog,
    root: u32,
    scalar: &str,
    values: &[u32],
    expected: &Table,
) -> Result<(), String> {
    let mut count = 0;
    for pair in expected.clone().pairs::<LuaValue, LuaValue>() {
        let (selector, value) = pair.map_err(|e| e.to_string())?;
        if !matches!(selector, LuaValue::String(_)) {
            return Err("expected_values keys must be original attribute names".into());
        }
        let descriptor = catalog.get(&selector)?;
        if uint(&descriptor["root_hash"])? != root {
            return Err("expected attribute belongs to a different root".into());
        }
        let index = uint(&descriptor["index"])? as usize;
        let expected_bits = u32::from_le_bytes(encode_attribute_scalar(descriptor, scalar, &value)?);
        if values.get(index) != Some(&expected_bits) {
            return Err("Attribute snapshot changed before update. Read current values and retry".into());
        }
        count += 1;
        if count > values.len() {
            return Err("too many expected root attributes".into());
        }
    }
    if count == 0 {
        return Err("expected_values must contain at least one attribute".into());
    }
    Ok(())
}

pub(crate) fn attach(lua: &Lua, ecs: &Table, r#mod: &Mod) -> mlua::Result<()> {
    ecs.raw_set(
        "evaluate_attributes",
        lua.create_function(|lua, (selector, values): (LuaValue, Table)| {
            let _timing = runtime_metrics::span("attributes.evaluate_attributes");
            let execute = || -> Result<LuaValue, String> {
                let state = lua.app_data_ref::<AppState>().unwrap();
                let catalog = state.attribute_catalog()?;
                let descriptor = catalog.get(&selector)?;
                let root = uint(&descriptor["root_hash"])?;
                let entries = catalog.root_entries(root)?;
                let scalar = descriptor["calculation_scalar_type"]
                    .as_str()
                    .ok_or("unknown scalar type")?;
                let mut input = Vec::new();
                for entry in &entries {
                    if entry["calculation_encoding_verified"] != true || entry["calculation"]["decoded"] != true {
                        return Err("attribute root has an unverified calculation program".into());
                    }
                    if entry["calculation_scalar_type"].as_str() != Some(scalar) {
                        return Err("attribute root has conflicting scalar interpretations".into());
                    }
                    let name = entry["name"].as_str().unwrap();
                    let value = values
                        .raw_get::<LuaValue>(name)
                        .map_err(|e| e.to_string())?;
                    input.push(u32::from_le_bytes(
                        encode_attribute_scalar(entry, scalar, &value).map_err(|e| format!("{name}: {e}"))?,
                    ));
                }
                if values.pairs::<LuaValue, LuaValue>().count() != entries.len() {
                    return Err("unexpected attribute names in root values".into());
                }
                let (result, trace) = catalog.recalculate(root, &input)?;
                for (entry, bits) in entries.iter().zip(&result) {
                    validate_attribute_bits(entry, scalar, *bits)?;
                }
                registry::json_to_lua(lua, &root_result(&entries, scalar, &result, trace)?)
                    .map_err(|e| e.to_string())
            };
            Ok(match execute() {
                Ok(v) => (v, None),
                Err(e) => (LuaValue::Nil, Some(e)),
            })
        })?,
    )?;
    for update in [false, true] {
        let r#mod = r#mod.clone();
        ecs.raw_set(if update { "update_attribute" } else { "read_attributes" }, lua.create_function(move |lua, args: mlua::MultiValue| {
            let _timing = runtime_metrics::span(if update { "attributes.update_attribute" } else { "attributes.read_attributes" });
            let execute = || -> Result<LuaValue, String> {
                let state = lua.app_data_ref::<AppState>().unwrap();
                let feature = if update { "runtime.ecs.write" } else { "runtime.ecs.read" };
                if let Some(reason) = loader::runtime_denial_reason(&state, &r#mod, feature) { return Err(reason); }
                if !loader::available(&state, &r#mod, feature) { return Err(format!("{feature} unavailable")); }
                if (!update && args.len() != 2) || (update && args.len() != 3 && args.len() != 4) { return Err("expected entity, attribute name/ID, and (for updates) scalar value plus optional expected_values".into()); }
                let LuaValue::Integer(entity) = args[0] else { return Err("expected entity integer".into()); };
                let entity = u32::try_from(entity).map_err(|_| "entity outside u32")?;
                let catalog = state.attribute_catalog()?;
                let (descriptor, component, before) = entity_storage(catalog, catalog.get(&args[1])?, entity)?;
                let name = component["qualified_name"].as_str().unwrap();
                let scalar = component["storage_scalar_type"].as_str().unwrap();
                let root = uint(&descriptor["root_hash"])?;
                let entries = catalog.root_entries(root)?;
                let start = uint(&component["storage_offset"])? as usize;
                let end = start + uint(&component["storage_size"])? as usize;
                let mut values: Vec<u32> = before[start..end].chunks_exact(4).map(|v| u32::from_le_bytes(v.try_into().unwrap())).collect();
                if entries.len() != values.len() { return Err("attribute root storage count changed".into()); }
                let mut trace = Vec::new();
                if update {
                    if args.len() == 4 {
                        let LuaValue::Table(expected) = &args[3] else { return Err("expected_values must be a table of original attribute names and values".into()); };
                        expect_root_values(catalog, root, scalar, &values, expected)?;
                    }
                    let mut after = before.clone();
                    let mut mask = vec![0; before.len()];
                    let range = storage_range(&descriptor, &component, &before)?;
                    after[range.clone()].copy_from_slice(&encode_attribute_scalar(&descriptor, scalar, &args[2])?);
                    mask[range].fill(1);
                    let functions = loader::runtime_provider::functions()?;
                    trace = recalculate_bound_component_update(catalog, name, &before, &mut after, &mut mask,
                        functions["attribute_calculation_model"].as_str().unwrap_or(""))?;
                    values = after[start..end].chunks_exact(4).map(|v| u32::from_le_bytes(v.try_into().unwrap())).collect();
                    super::runtime_ownership::check_write(&state, &r#mod,
                        &super::runtime_ownership::component_target(entity, name))?;
                    loader::runtime_provider::compare_exchange(entity, name, &before, &mask, &after)?;
                }
                let mut result = root_result(&entries, scalar, &values, trace)?;
                result["entity"] = json!(entity);
                result["component"] = component;
                result["applied"] = json!(update);
                result["effect_scope"] = json!("local process, no automatic client/server RPC");
                registry::json_to_lua(lua, &result).map_err(|e| e.to_string())
            };
            Ok(match execute() { Ok(v) => (v, None), Err(e) => (LuaValue::Nil, Some(e)) })
        })?)?;
    }
    ecs.raw_set("get_attributes", lua.create_function(|lua, ()| {
        let state = lua.app_data_ref::<AppState>().unwrap();
        let catalog = match state.attribute_catalog() { Ok(c) => c, Err(reason) => return Ok((LuaValue::Nil, Some(reason))) };
        let result = json!({"version":state.type_registry().version,"count":catalog.entries.len(),
            "entries":catalog.entries.values().collect::<Vec<_>>(),"component_layouts":catalog.layouts,
            "storage_resolution":"per-entity-initialized-root-id"});
        Ok((registry::json_to_lua(lua, &result)?, None))
    })?)?;
    ecs.raw_set(
        "get_attribute",
        lua.create_function(|lua, selector: LuaValue| {
            let state = lua.app_data_ref::<AppState>().unwrap();
            let descriptor = state.attribute_catalog().and_then(|c| c.get(&selector));
            match descriptor {
                Ok(d) => Ok((registry::json_to_lua(lua, d)?, None)),
                Err(e) => Ok((LuaValue::Nil, Some(e))),
            }
        })?,
    )?;
    for write in [false, true] {
        let r#mod = r#mod.clone();
        ecs.raw_set(
            if write {
                "write_attribute_storage"
            } else {
                "read_attribute"
            },
            lua.create_function(move |lua, args: mlua::MultiValue| {
                let _timing = runtime_metrics::span(if write { "attributes.write_attribute_storage" } else { "attributes.read_attribute" });
                let state = lua.app_data_ref::<AppState>().unwrap();
                let execute = || -> Result<LuaValue, String> {
                    let feature = if write {
                        "runtime.ecs.write"
                    } else {
                        "runtime.ecs.read"
                    };
                    if let Some(reason) = loader::runtime_denial_reason(&state, &r#mod, feature) {
                        return Err(reason);
                    }
                    if !loader::available(&state, &r#mod, feature) {
                        return Err(format!("{feature} unavailable"));
                    }
                    if args.len() != if write { 3 } else { 2 } {
                        return Err(
                            "expected entity, attribute name/ID, and (for writes) scalar value"
                                .into(),
                        );
                    }
                    let entity = match &args[0] {
                        LuaValue::Integer(v) => {
                            u32::try_from(*v).map_err(|_| "entity outside u32")?
                        }
                        _ => return Err("expected entity integer".into()),
                    };
                    let catalog = state.attribute_catalog()?;
                    let (descriptor, component, mut bytes) =
                        entity_storage(catalog, catalog.get(&args[1])?, entity)?;
                    let name = component["qualified_name"].as_str().unwrap();
                    let range = storage_range(&descriptor, &component, &bytes)?;
                    let scalar = component["storage_scalar_type"]
                        .as_str()
                        .ok_or("unresolved attribute scalar type")?;
                    if write {
                        if descriptor["storage_writable"] != true {
                            return Err(descriptor["storage_reason"]
                                .as_str()
                                .unwrap_or("unverified attribute write layout")
                                .into());
                        }
                        let encoded = encode_attribute_scalar(&descriptor, scalar, &args[2])?;
                        let before = bytes.clone();
                        let mut mask = vec![0u8; bytes.len()];
                        mask[range.clone()].fill(1);
                        bytes[range].copy_from_slice(&encoded);
                        recalculate_component_update(&state, name, &before, &mut bytes, &mut mask)?;
                        // Include the initialized header and root identity in
                        // the expected snapshot. A delayed write must not use
                        // an association that changed after entity_storage.
                        super::runtime_ownership::check_write(&state, &r#mod,
                            &super::runtime_ownership::component_target(entity, name))?;
                        loader::runtime_provider::compare_exchange(
                            entity, name, &before, &mask, &bytes,
                        )?;
                        Ok(LuaValue::Boolean(true))
                    } else {
                        let data: [u8; 4] = bytes[range].try_into().unwrap();
                        match scalar {
                            "keen::sint32" => {
                                Ok(LuaValue::Integer(i32::from_le_bytes(data) as i64))
                            }
                            "keen::uint32" => {
                                Ok(LuaValue::Integer(u32::from_le_bytes(data) as i64))
                            }
                            "keen::float32" => {
                                Ok(LuaValue::Number(f32::from_le_bytes(data) as f64))
                            }
                            _ => Err("unsupported attribute scalar".into()),
                        }
                    }
                };
                match execute() {
                    Ok(value) => Ok((value, None)),
                    Err(e) => Ok((
                        if write {
                            LuaValue::Boolean(false)
                        } else {
                            LuaValue::Nil
                        },
                        Some(e),
                    )),
                }
            })?,
        )?;
    }
    Ok(())
}

fn root_result(
    entries: &[&Value],
    scalar: &str,
    values: &[u32],
    trace: Vec<Value>,
) -> Result<Value, String> {
    let _timing = runtime_metrics::span("attributes.root_result");
    let mut named = serde_json::Map::new();
    for (entry, bits) in entries.iter().zip(values) {
        let value = match scalar {
            "keen::sint32" => json!(*bits as i32),
            "keen::uint32" => json!(*bits),
            "keen::float32" => {
                let value = f32::from_bits(*bits);
                if !value.is_finite() {
                    return Err("non-finite attribute value".into());
                }
                json!(value)
            }
            _ => return Err("unresolved attribute scalar".into()),
        };
        named.insert(entry["name"].as_str().unwrap().to_owned(), value);
    }
    Ok(
        json!({"root_hash":entries[0]["root_hash"],"values":named,"trace":trace,
        "calculation_model":"attribute-command-v1","scalar_type":scalar}),
    )
}

fn encode_scalar(kind: &str, value: &LuaValue) -> Result<[u8; 4], String> {
    match (kind, value) {
        ("keen::sint32", LuaValue::Integer(v)) => Ok(i32::try_from(*v)
            .map_err(|_| "value outside sint32")?
            .to_le_bytes()),
        ("keen::uint32", LuaValue::Integer(v)) => Ok(u32::try_from(*v)
            .map_err(|_| "value outside uint32")?
            .to_le_bytes()),
        ("keen::float32", LuaValue::Number(v)) if v.is_finite() && (*v as f32).is_finite() => {
            Ok((*v as f32).to_le_bytes())
        }
        ("keen::float32", LuaValue::Integer(v)) => Ok((*v as f32).to_le_bytes()),
        _ => Err(format!("expected finite scalar of type {kind}")),
    }
}

fn encode_attribute_scalar(
    descriptor: &Value,
    kind: &str,
    value: &LuaValue,
) -> Result<[u8; 4], String> {
    if descriptor["write_value_domain"] == "integer-common-range-0-to-2147483647"
        && !matches!(value, LuaValue::Integer(v) if (0..=i32::MAX as i64).contains(v))
    {
        return Err(
            "conflicting signedness: only the common integer range 0..2147483647 is validated"
                .into(),
        );
    }
    encode_scalar(kind, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_resolution::attributes::decode_program;
    use crate::runtime_resolution::attributes::{header_layout, tests::header_fixture};
    use kfc::{
        container::{KFCFile, KFCReader},
        reflection::{TypeIndex, TypeRegistry},
    };

    fn raw_update_fixture(compact: bool, scalar: &str) -> (Catalog, Vec<u8>, usize) {
        let registry = header_fixture(compact);
        let header = header_layout(&registry, TypeIndex::new(8)).unwrap();
        let start = if compact { 8 } else { 36 };
        let root = 0x04b6aa8bu32;
        let mut catalog = Catalog::default();
        let names = ["Current", "Min", "Max", "Rampup", "Timeout", "Recharge", "Base", "Adder"];
        for (index, name) in names.into_iter().enumerate() {
            let words = if index == 0 {
                vec![0x20000, 0x20001, 0x20002, 0xcffff]
            } else if index == 2 && scalar == "keen::sint32" {
                vec![0x30000, 0x20001, 0x20002, 0x20006, 0x20007, 0x6ffff, 0xdffff]
            } else { Vec::new() };
            catalog.entries.insert(root + index as u32, json!({
                "name":name,"hash":root + index as u32,"root_hash":root,"index":index,
                "scalar_type":scalar,"calculation_scalar_type":scalar,
                "calculation_sign_independent":false,"calculation_encoding_verified":true,
                "calculation":{"decoded":true},"calculation_words":words
            }));
        }
        let mut component = json!({"qualified_name":"keen::ecs::Stamina","size":start+32,
            "header":header,"storage_offset":start,"storage_size":32,"storage_scalar_type":scalar});
        let mut bytes = vec![0; start + 32];
        bytes[..4].copy_from_slice(&root.to_le_bytes());
        if compact {
            bytes[4..6].copy_from_slice(&(start as u16).to_le_bytes());
            bytes[6] = 1;
            bytes[7] = 8;
        } else {
            bytes[8..12].copy_from_slice(&(start as u32).to_le_bytes());
            bytes[12..16].copy_from_slice(&8u32.to_le_bytes());
            bytes[16..18].copy_from_slice(&1u16.to_le_bytes());
            let guid = Guid::parse("f552f8bc-ada0-4a58-8739-4081ce0d3873").unwrap();
            bytes[20..36].copy_from_slice(guid.data());
            component["definition_offset"] = json!(20);
            component["definition_guid"] = json!(guid.to_string());
        }
        for (index, value) in [50u32, 0, 100, 0, 0, 0, 100, 0].into_iter().enumerate() {
            let bits = if scalar == "keen::float32" { (value as f32).to_bits() } else { value };
            bytes[start + index*4..start + (index+1)*4].copy_from_slice(&bits.to_le_bytes());
        }
        catalog.layouts.push(component);
        (catalog, bytes, start)
    }

    #[test]
    fn raw_attribute_write_recalculates_derived_values_and_refreshes_complete_scalar_masks() {
        for compact in [false, true] {
            let (catalog, before, start) = raw_update_fixture(compact, "keen::sint32");
            let mut after = before.clone();
            let mut mask = vec![0; before.len()];
            after[start+24..start+28].copy_from_slice(&200i32.to_le_bytes());
            mask[start+24] = 1;
            recalculate_bound_component_update(&catalog, "keen::ecs::Stamina", &before,
                &mut after, &mut mask, "attribute-command-v1").unwrap();
            assert_eq!(i32::from_le_bytes(after[start..start+4].try_into().unwrap()), 100);
            assert_eq!(i32::from_le_bytes(after[start+8..start+12].try_into().unwrap()), 200);
            assert_eq!(&after[..start], &before[..start]);
            for index in 0..8 {
                assert_eq!(&mask[start+index*4..start+(index+1)*4],
                    if [0, 2, 6].contains(&index) { &[1, 1, 1, 1] } else { &[0, 0, 0, 0] });
            }
            assert!(mask[..start].iter().all(|byte| *byte == 0));
        }
    }

    #[test]
    fn raw_attribute_write_rejects_headers_uninitialized_storage_and_unmasked_changes() {
        for compact in [false, true] {
            let (catalog, before, start) = raw_update_fixture(compact, "keen::sint32");
            let header_offsets = if compact { vec![0, 4, 6, 7] } else { vec![0, 8, 12, 16, 20] };
            for offset in header_offsets {
                let mut after = before.clone();
                let mut mask = vec![0; before.len()];
                after[offset] ^= 1;
                mask[offset] = 1;
                let candidate = after.clone();
                let selected = mask.clone();
                assert!(recalculate_bound_component_update(&catalog, "keen::ecs::Stamina", &before,
                    &mut after, &mut mask, "attribute-command-v1").unwrap_err().contains("read-only"));
                assert_eq!((after, mask), (candidate, selected));
            }
            let mut after = before.clone();
            let mut mask = vec![0; before.len()];
            mask[0] = 1;
            assert!(recalculate_bound_component_update(&catalog, "keen::ecs::Stamina", &before,
                &mut after, &mut mask, "attribute-command-v1").is_err());
            mask.fill(0);
            after[start] ^= 1;
            assert!(recalculate_bound_component_update(&catalog, "keen::ecs::Stamina", &before,
                &mut after, &mut mask, "attribute-command-v1").unwrap_err().contains("write mask"));
            let mut uninitialized = before.clone();
            uninitialized[if compact { 6 } else { 16 }] = 0;
            let mut after = uninitialized.clone();
            assert!(recalculate_bound_component_update(&catalog, "keen::ecs::Stamina", &uninitialized,
                &mut after, &mut mask, "attribute-command-v1").unwrap_err().contains("uninitialized"));
        }
    }

    #[test]
    fn raw_attribute_write_requires_verified_model_programs_and_scalar_domain() {
        let (mut catalog, before, start) = raw_update_fixture(true, "keen::sint32");
        let mut after = before.clone();
        let mut mask = vec![0; before.len()];
        assert!(recalculate_bound_component_update(&catalog, "keen::ecs::Stamina", &before,
            &mut after, &mut mask, "unknown-model").unwrap_err().contains("not validated"));
        catalog.entries.values_mut().next().unwrap()["calculation_words"] = json!([0xffff0000u32]);
        assert!(recalculate_bound_component_update(&catalog, "keen::ecs::Stamina", &before,
            &mut after, &mut mask, "attribute-command-v1").is_err());
        assert_eq!(after, before);
        assert!(mask.iter().all(|byte| *byte == 0));

        let (mut conflict, before, _) = raw_update_fixture(true, "keen::sint32");
        for entry in conflict.entries.values_mut() {
            entry["scalar_type"] = json!("keen::uint32");
        }
        let mut after = before.clone();
        assert!(recalculate_bound_component_update(&conflict, "keen::ecs::Stamina", &before,
            &mut after, &mut mask, "attribute-command-v1").unwrap_err().contains("signedness-dependent"));

        let (mut catalog, before, start2) = raw_update_fixture(true, "keen::sint32");
        assert_eq!(start, start2);
        for entry in catalog.entries.values_mut() {
            entry["scalar_type"] = json!("keen::uint32");
            entry["calculation_scalar_type"] = json!("keen::uint32");
            entry["calculation_sign_independent"] = json!(true);
            entry["calculation_words"] = json!([]);
        }
        let mut after = before.clone();
        after[start+28..start+32].copy_from_slice(&u32::MAX.to_le_bytes());
        mask[start+28..start+32].fill(1);
        assert!(recalculate_bound_component_update(&catalog, "keen::ecs::Stamina", &before,
            &mut after, &mut mask, "attribute-command-v1").unwrap_err().contains("common integer range"));

        let (catalog, before, start) = raw_update_fixture(true, "keen::float32");
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut after = before.clone();
            let mut mask = vec![0; before.len()];
            after[start+28..start+32].copy_from_slice(&invalid.to_bits().to_le_bytes());
            mask[start+28..start+32].fill(1);
            assert!(recalculate_bound_component_update(&catalog, "keen::ecs::Stamina", &before,
                &mut after, &mut mask, "attribute-command-v1").unwrap_err().contains("finite scalar"));
        }
    }

    #[test]
    fn named_attribute_updates_validate_unselected_programs_and_derived_result_domains() {
        let (mut catalog, before, start) = raw_update_fixture(true, "keen::sint32");
        let sibling = 0x04b6aa8bu32 + 7;
        catalog.entries.get_mut(&sibling).unwrap()["calculation"]["decoded"] = json!(false);
        let mut after = before.clone();
        let mut mask = vec![0; before.len()];
        after[start..start + 4].copy_from_slice(&75i32.to_le_bytes());
        mask[start..start + 4].fill(1);
        let input: Vec<_> = after[start..].chunks_exact(4)
            .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap())).collect();
        // Arithmetic alone accepts the root. A live write must also validate
        // the untouched sibling's program evidence before reaching native CAS.
        assert!(catalog.recalculate(0x04b6aa8b, &input).is_ok());
        let candidate = after.clone();
        let selected = mask.clone();
        assert!(recalculate_bound_component_update(&catalog, "keen::ecs::Stamina", &before,
            &mut after, &mut mask, "attribute-command-v1").unwrap_err().contains("unverified"));
        assert_eq!((&after, &mask), (&candidate, &selected));

        let (mut catalog, before, start) = raw_update_fixture(true, "keen::sint32");
        let sibling = catalog.entries.get_mut(&sibling).unwrap();
        sibling["scalar_type"] = json!("keen::uint32");
        sibling["calculation_sign_independent"] = json!(true);
        sibling["calculation_words"] = json!([0x5ffffu32, u32::MAX]);
        let mut after = before.clone();
        let mut mask = vec![0; before.len()];
        after[start..start + 4].copy_from_slice(&75i32.to_le_bytes());
        mask[start..start + 4].fill(1);
        let input: Vec<_> = after[start..].chunks_exact(4)
            .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap())).collect();
        assert_eq!(catalog.recalculate(0x04b6aa8b, &input).unwrap().0[7], u32::MAX);
        let candidate = after.clone();
        let selected = mask.clone();
        assert!(recalculate_bound_component_update(&catalog, "keen::ecs::Stamina", &before,
            &mut after, &mut mask, "attribute-command-v1").unwrap_err().contains("common integer range"));
        assert_eq!((after, mask), (candidate, selected));
    }

    #[test]
    fn raw_attribute_policy_follows_actual_reflection_inheritance_and_exact_storage() {
        let registry = header_fixture(true);
        assert!(is_attribute_component(&registry, "keen::ecs::Stamina").unwrap());
        assert!(!is_attribute_component(&registry, "keen::HashKey32").unwrap());
        let (mut catalog, before, _) = raw_update_fixture(true, "keen::sint32");
        catalog.layouts.clear();
        let mut after = before.clone();
        let mut mask = vec![0; before.len()];
        assert!(recalculate_bound_component_update(&catalog, "keen::ecs::Stamina", &before,
            &mut after, &mut mask, "attribute-command-v1").unwrap_err().contains("no verified"));
    }

    #[test]
    fn attribute_snapshot_accepts_compact_header_without_reading_storage_as_header() {
        for compact in [false, true] {
            let registry = header_fixture(compact);
            let header = header_layout(&registry, TypeIndex::new(8)).unwrap();
            let start = if compact { 8 } else { 36 };
            let mut component =
                json!({"size":start+32,"header":header,"storage_offset":start,"storage_size":32});
            let root = 0x04b6aa8bu32;
            let descriptor = json!({"root_hash":root,"index":0});
            let mut bytes = vec![0; start + 32];
            bytes[..4].copy_from_slice(&root.to_le_bytes());
            if compact {
                bytes[4..6].copy_from_slice(&(start as u16).to_le_bytes());
                bytes[6] = 1;
                bytes[7] = 8;
            } else {
                bytes[8..12].copy_from_slice(&(start as u32).to_le_bytes());
                bytes[12..16].copy_from_slice(&8u32.to_le_bytes());
                bytes[16..18].copy_from_slice(&1u16.to_le_bytes());
                let guid = Guid::parse("f552f8bc-ada0-4a58-8739-4081ce0d3873").unwrap();
                bytes[20..36].copy_from_slice(guid.data());
                component["definition_offset"] = json!(20);
                component["definition_guid"] = json!(guid.to_string());
            }
            bytes[start..start + 4].copy_from_slice(&50u32.to_le_bytes());
            assert_eq!(
                storage_range(&descriptor, &component, &bytes).unwrap(),
                start..start + 4
            );
            let mut corrupt = bytes.clone();
            corrupt[0] ^= 1;
            assert!(storage_range(&descriptor, &component, &corrupt).is_err());
            let flags = component["header"]["flags"]["offset"].as_u64().unwrap() as usize;
            bytes[flags] = 0;
            assert!(storage_range(&descriptor, &component, &bytes).is_err());
        }
    }

    #[test]
    fn attribute_entity_join_requires_one_initialized_root_match() {
        let registry = header_fixture(true);
        let header = header_layout(&registry, TypeIndex::new(8)).unwrap();
        let root = 0x04b6aa8bu32;
        let mut catalog = Catalog::default();
        for index in 0..8 {
            catalog.entries.insert(
                index,
                json!({"root_hash":root,"index":index,
                "scalar_type":"keen::uint32","calculation_sign_independent":true}),
            );
        }
        for name in ["keen::ecs::Stamina", "keen::ecs::DifferentName"] {
            catalog
                .layouts
                .push(json!({"qualified_name":name,"size":40,"header":header,
                "storage_offset":8,"storage_size":32,"storage_scalar_type":"keen::uint32"}));
        }
        let descriptor = catalog.entries[&0].clone();
        let mut valid = vec![0; 40];
        valid[..4].copy_from_slice(&root.to_le_bytes());
        valid[4..6].copy_from_slice(&8u16.to_le_bytes());
        valid[6] = 1;
        valid[7] = 8;
        let mut different_root = valid.clone();
        different_root[0] ^= 1;
        let (bound, component, _) = resolve_storage(&catalog, &descriptor, |component| {
            Some(if component["qualified_name"] == "keen::ecs::Stamina" {
                different_root.clone()
            } else {
                valid.clone()
            })
        })
        .unwrap();
        assert_eq!(component["qualified_name"], "keen::ecs::DifferentName");
        assert_eq!(bound["storage_writable"], true);
        assert!(
            resolve_storage(&catalog, &descriptor, |_| Some(valid.clone()))
                .unwrap_err()
                .contains("multiple")
        );
        valid[6] = 0;
        assert!(
            resolve_storage(&catalog, &descriptor, |_| Some(valid.clone()))
                .unwrap_err()
                .contains("no initialized component")
        );
    }

    #[test]
    #[ignore = "set SHROUDFORGE_TEST_EXE to inspect current installed attribute definitions"]
    fn installed_attribute_resolution() {
        let exe = std::path::PathBuf::from(std::env::var("SHROUDFORGE_TEST_EXE").unwrap());
        let registry = TypeRegistry::load_from_executable(&exe).unwrap();
        let file = KFCFile::from_path(exe.with_extension("kfc"), false).unwrap();
        let mut reader = KFCReader::new(
            exe.parent().unwrap(),
            exe.file_stem().unwrap().to_str().unwrap(),
        )
        .unwrap()
        .into_cursor()
        .unwrap();
        let catalog = Catalog::load(&registry, &file, &mut reader).unwrap();
        assert!(!catalog.layouts.is_empty());
        assert_eq!(catalog.entries[&0x04b6aa8b]["name"], "Stamina");
        assert_eq!(catalog.entries[&0x8eb84995]["name"], "Health");
        assert!(
            catalog
                .entries
                .values()
                .all(|entry| entry["calculation"]["decoded"] == true)
        );
        for layout in &catalog.layouts {
            let ty = registry
                .get_by_name(kfc::reflection::LookupKey::Qualified(
                    layout["qualified_name"].as_str().unwrap(),
                ))
                .unwrap();
            assert_eq!(
                layout["header"],
                header_layout(&registry, ty.index).unwrap()
            );
            assert_eq!(layout["header"]["flags"]["initialized_mask"], 1);
        }
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../target/enshrouded-attribute-resolution.json");
        std::fs::write(path, serde_json::to_vec_pretty(&json!({"version":registry.version,
            "entries":catalog.entries.values().collect::<Vec<_>>(),"component_layouts":catalog.layouts})).unwrap()).unwrap();
        println!(
            "{} original attributes and {} reflected component storage layouts inspected, no live storage association assumed",
            catalog.entries.len(),
            catalog.layouts.len()
        );
        if let Ok(path) = std::env::var("SHROUDFORGE_TEST_ECS_SNAPSHOTS") {
            let capture: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            let snapshots = capture["snapshots"].as_array().unwrap();
            let mut entities = std::collections::BTreeMap::<
                (u32, u32),
                std::collections::BTreeMap<String, Vec<u8>>,
            >::new();
            for snapshot in snapshots {
                let hex = snapshot["bytes"].as_str().unwrap();
                let bytes = hex
                    .as_bytes()
                    .chunks_exact(2)
                    .map(|pair| {
                        ((pair[0] as char).to_digit(16).unwrap() * 16
                            + (pair[1] as char).to_digit(16).unwrap()) as u8
                    })
                    .collect();
                entities
                    .entry((
                        uint(&snapshot["entityId"]).unwrap(),
                        uint(&snapshot["generation"]).unwrap(),
                    ))
                    .or_default()
                    .insert(snapshot["component"].as_str().unwrap().to_owned(), bytes);
            }
            let lua = Lua::new();
            let mut roots_checked = 0;
            let mut root_types = std::collections::BTreeSet::new();
            let mut scalar_values_checked = 0;
            for snapshot in snapshots
                .iter()
                .filter(|snapshot| !snapshot["rootId"].is_null())
            {
                let root = uint(&snapshot["rootId"]).unwrap();
                let source = &entities[&(
                    uint(&snapshot["entityId"]).unwrap(),
                    uint(&snapshot["generation"]).unwrap(),
                )];
                let descriptor = &catalog.entries[&root];
                let (_, component, bytes) = resolve_storage(&catalog, descriptor, |component| {
                    source
                        .get(component["qualified_name"].as_str().unwrap())
                        .cloned()
                })
                .unwrap();
                assert_eq!(component["qualified_name"], snapshot["component"]);
                let start = uint(&component["storage_offset"]).unwrap() as usize;
                let end = start + uint(&component["storage_size"]).unwrap() as usize;
                let values = bytes[start..end]
                    .chunks_exact(4)
                    .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
                    .collect::<Vec<_>>();
                let entries = catalog.root_entries(root).unwrap();
                let scalar = component["storage_scalar_type"].as_str().unwrap();
                let result = root_result(&entries, scalar, &values, Vec::new()).unwrap();
                assert_eq!(
                    result["values"].as_object().unwrap().len(),
                    snapshot["values"].as_object().unwrap().len()
                );
                let LuaValue::Table(result) = registry::json_to_lua(&lua, &result).unwrap() else {
                    panic!("root must become Lua table");
                };
                let values = result.raw_get::<Table>("values").unwrap();
                for (name, expected) in snapshot["values"].as_object().unwrap() {
                    let value = values.raw_get::<LuaValue>(name.as_str()).unwrap();
                    let actual = match value {
                        LuaValue::Integer(value) => value as f64,
                        LuaValue::Number(value) => value,
                        _ => panic!("attribute scalar did not become Lua number"),
                    };
                    if scalar == "keen::float32" {
                        assert_eq!(
                            (actual as f32).to_bits(),
                            (expected.as_f64().unwrap() as f32).to_bits(),
                            "{name}"
                        );
                    } else {
                        assert_eq!(Some(actual), expected.as_f64(), "{name}");
                    }
                    scalar_values_checked += 1;
                }
                roots_checked += 1;
                root_types.insert(root);
            }
            let report = json!({"source_capture":path,"capture_execution":"external-read-only-process-snapshot",
                "new_dll_or_live_lua_verified":false,"root_snapshots_checked":roots_checked,
                "distinct_root_types_checked":root_types.len(),"scalar_values_compared":scalar_values_checked,
                "resolution":"production-per-entity-root-join-and-Lua-root-result"});
            std::fs::write(
                format!("{path}.attribute-codec.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
            println!(
                "Production entity-root join and Lua results checked {roots_checked} external root snapshots, {} root types and {scalar_values_checked} scalar values",
                root_types.len()
            );
        }
    }

    #[test]
    #[ignore = "set SHROUDFORGE_TEST_EXE to the KFC3 client/server executable used by this regression"]
    fn installed_attributes() {
        let exe = std::path::PathBuf::from(std::env::var("SHROUDFORGE_TEST_EXE").unwrap());
        let types = TypeRegistry::load_from_executable(&exe).unwrap();
        let file = KFCFile::from_path(exe.with_extension("kfc"), false).unwrap();
        let mut reader = KFCReader::new(
            exe.parent().unwrap(),
            exe.file_stem().unwrap().to_str().unwrap(),
        )
        .unwrap()
        .into_cursor()
        .unwrap();
        let catalog = Catalog::load(&types, &file, &mut reader).unwrap();
        assert_eq!(catalog.entries[&0x04b6aa8b]["name"], "Stamina");
        assert_eq!(catalog.entries[&0xf443c410]["name"], "Stamina_Max");
        assert_eq!(catalog.entries[&0x8eb84995]["name"], "Health");
        assert_eq!(catalog.entries.len(), 304);
        assert!(
            catalog
                .entries
                .values()
                .all(|entry| entry["components"].as_array().unwrap().len() == 1)
        );
        let mana = catalog
            .entries
            .values()
            .find(|entry| entry["name"] == "ManaRechargeMod")
            .unwrap();
        assert!(mana["resource_ids"].as_array().unwrap().is_empty());
        assert_eq!(
            mana["components"][0]["qualified_name"],
            "keen::ecs::ManaRechargeMod"
        );
        let freeze = catalog
            .entries
            .values()
            .find(|entry| entry["name"] == "FreezingResistance")
            .unwrap();
        assert_eq!(freeze["calculation_writable"], true);
        assert!(encode_attribute_scalar(freeze, "keen::sint32", &LuaValue::Integer(-1)).is_err());
        let heat = catalog
            .entries
            .values()
            .find(|entry| entry["name"] == "BodyHeat")
            .unwrap();
        assert_eq!(heat["calculation_writable"], false);
        for entry in catalog.entries.values().filter(|e| e["index"] == 0) {
            let root = uint(&entry["root_hash"]).unwrap();
            let entries = catalog.root_entries(root).unwrap();
            catalog.recalculate(root, &vec![0; entries.len()]).unwrap();
        }
        // Derived maxima run before the root clamp and rescale the current
        // value through LoadRef, as the native descending recompute loop does.
        let (values, trace) = catalog
            .recalculate(0x04b6aa8b, &[50, 0, 100, 2000, 250, 95, 200, 0])
            .unwrap();
        assert_eq!((values[0], values[2]), (100, 200));
        assert!(!trace.is_empty());
        let mapped = catalog
            .entries
            .values()
            .filter(|v| v["storage_writable"] == true)
            .count();
        assert!(mapped >= 290);
        let output = exe.file_stem().unwrap().to_str().unwrap();
        let artifacts = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../target");
        let path = artifacts.join(format!("{output}-attribute-catalog.json"));
        std::fs::write(
            path,
            serde_json::to_vec_pretty(
                &json!({"entries":catalog.entries.values().collect::<Vec<_>>()}),
            )
            .unwrap(),
        )
        .unwrap();
        let target = if output.contains("server") {
            "server"
        } else {
            "client"
        };
        if let Ok(bytes) = std::fs::read(artifacts.join(format!("{target}-attribute-live.json"))) {
            let snapshots: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
            for snapshot in &snapshots {
                let lua = Lua::new();
                let selector = LuaValue::String(
                    lua.create_string(snapshot["component"].as_str().unwrap())
                        .unwrap(),
                );
                let descriptor = catalog.get(&selector).unwrap();
                let bytes =
                    super::super::buffer_transform::unhex(snapshot["bytes"].as_str().unwrap())
                        .unwrap();
                let component = &descriptor["components"][0];
                assert_eq!(
                    storage_range(descriptor, component, &bytes).unwrap(),
                    36..40
                );
                let mut corrupt = bytes.clone();
                corrupt[8] = 0xff;
                assert!(storage_range(descriptor, component, &corrupt).is_err());
                corrupt = bytes.clone();
                corrupt[0] ^= 1;
                assert!(storage_range(descriptor, component, &corrupt).is_err());
            }
            println!(
                "{} read-only live component snapshots accepted. Corrupt headers rejected",
                snapshots.len()
            );
        }
        println!(
            "{} original attribute IDs, {mapped} with unique reflected component storage",
            catalog.entries.len()
        );
    }
    #[test]
    fn expected_attribute_values_reject_stale_bounds_and_other_roots() {
        let lua = Lua::new();
        let mut catalog = Catalog::default();
        for (id, name, root, index) in [
            (1, "Stamina", 1, 0),
            (2, "Stamina_Min", 1, 1),
            (3, "Stamina_Max", 1, 2),
            (4, "Health", 4, 0),
        ] {
            catalog.entries.insert(id, json!({"name":name,"root_hash":root,"index":index}));
        }
        let expected: Table = lua.load("return {Stamina_Min=0,Stamina_Max=100}").eval().unwrap();
        assert!(expect_root_values(&catalog, 1, "keen::sint32", &[50, 0, 100], &expected).is_ok());
        for changed in [[50, 1, 100], [50, 0, 200]] {
            assert!(expect_root_values(&catalog, 1, "keen::sint32", &changed, &expected)
                .unwrap_err().contains("snapshot changed"));
        }
        let cross_root: Table = lua.load("return {Health=50}").eval().unwrap();
        assert!(expect_root_values(&catalog, 1, "keen::sint32", &[50, 0, 100], &cross_root)
            .unwrap_err().contains("different root"));
        let empty = lua.create_table().unwrap();
        assert!(expect_root_values(&catalog, 1, "keen::sint32", &[50, 0, 100], &empty).is_err());
        let unknown: Table = lua.load("return {unknown=50}").eval().unwrap();
        assert!(expect_root_values(&catalog, 1, "keen::sint32", &[50, 0, 100], &unknown).is_err());
    }

    #[test]
    fn scalar_range_checks() {
        assert!(encode_scalar("keen::sint32", &LuaValue::Integer(i32::MAX as i64 + 1)).is_err());
        assert!(encode_scalar("keen::uint32", &LuaValue::Integer(-1)).is_err());
        assert!(encode_scalar("keen::float32", &LuaValue::Number(f64::INFINITY)).is_err());
        assert!(encode_scalar("keen::sint32", &LuaValue::Number(1.5)).is_err());
    }
    #[test]
    fn program_literals_are_not_misreported_as_opcodes() {
        let operations = [(2, "Load".into()), (5, "Push".into()), (12, "Clamp".into())]
            .into_iter()
            .collect();
        let program = decode_program(
            &[
                json!(0x20000),
                json!(0x5ffff),
                json!(0x20001),
                json!(0xcffff),
            ],
            &operations,
            &[json!({"value":123})],
            &[json!("original_name")],
        );
        assert_eq!(program["instructions"].as_array().unwrap().len(), 3);
        assert_eq!(program["instructions"][0]["attribute_hash"], 123);
        assert_eq!(program["instructions"][1]["immediate_bits"], 0x20001);
        assert_eq!(program["decoded"], true);
        assert_eq!(
            decode_program(&[json!(0x5ffff)], &operations, &[], &[])["decoded"],
            false
        );
        assert_eq!(
            decode_program(&[json!(0x20002)], &operations, &[], &[])["decoded"],
            false
        );
    }
}
