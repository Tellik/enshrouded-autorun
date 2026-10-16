//! Attribute identities come from the active KFC container, never FNV(name).
//! Storage layouts come from reflection. Ownership is checked against each
//! entity's initialized root ID, with legacy definition-GUID joins retained.
use super::{
    calculation::{self, Scalar},
    types,
};
use kfc::{
    container::{KFCCursor, KFCFile, KFCReader},
    guid::Guid,
    reflection::{LookupKey, PrimitiveType, TypeIndex, TypeRegistry},
    resource::value::Value as KfcValue,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct Catalog {
    pub entries: BTreeMap<u32, Value>,
    pub layouts: Vec<Value>,
}

/// Header fields changed from wide KFC3 storage to the compact KFC4 layout.
/// Keep the reflected field name and width together instead of treating their
/// offsets as proof of a fixed C layout.
pub(crate) fn header_layout(registry: &TypeRegistry, index: TypeIndex) -> Result<Value, String> {
    fn width(registry: &TypeRegistry, index: TypeIndex, depth: usize) -> Result<u32, String> {
        if depth > 16 {
            return Err("cyclic or excessively nested attribute header type".into());
        }
        let ty = registry
            .get(index)
            .ok_or("unresolved attribute header type")?;
        let expected = match ty.primitive_type {
            PrimitiveType::UInt8 | PrimitiveType::Bitmask8 => 1,
            PrimitiveType::UInt16 | PrimitiveType::Bitmask16 => 2,
            PrimitiveType::UInt32 | PrimitiveType::Bitmask32 => 4,
            PrimitiveType::Typedef | PrimitiveType::Enum => width(
                registry,
                ty.inner_type
                    .ok_or("missing attribute header storage type")?,
                depth + 1,
            )?,
            PrimitiveType::Struct => {
                let fields = types::fields(registry, index)?;
                let [(_, field)] = fields.as_slice() else {
                    return Err("attribute header wrapper has multiple or missing fields".into());
                };
                if field.data_offset != 0 {
                    return Err("attribute header wrapper does not start at zero".into());
                }
                width(registry, field.r#type, depth + 1)?
            }
            _ => return Err("attribute header is not an unsigned inline integer".into()),
        };
        if ty.size != expected {
            return Err("attribute header type size differs from its integer storage".into());
        }
        Ok(expected)
    }

    let ty = registry
        .get(index)
        .ok_or("unresolved attribute component")?;
    let fields = types::fields(registry, index)?;
    let mut result = json!({});
    for (key, names) in [
        ("rootId", &["rootId"][..]),
        ("storageOffset", &["storageOffset"][..]),
        ("storageCount", &["storageCount", "storageSize"][..]),
        ("flags", &["flags"][..]),
    ] {
        let matches: Vec<_> = fields
            .iter()
            .filter(|(_, field)| names.contains(&field.name.as_str()))
            .collect();
        let [(_, field)] = matches.as_slice() else {
            return Err(format!("missing or ambiguous attribute header field {key}"));
        };
        let size = width(registry, field.r#type, 0)?;
        if field
            .data_offset
            .checked_add(u64::from(size))
            .is_none_or(|end| end > u64::from(ty.size))
        {
            return Err(format!(
                "attribute header field {key} exceeds its component"
            ));
        }
        result[key] = json!({"name":field.name,"offset":field.data_offset,"size":size});
        if key == "flags" {
            let flags = registry
                .get(field.r#type)
                .ok_or("unresolved attribute flags type")?;
            if !matches!(
                flags.primitive_type,
                PrimitiveType::Bitmask8 | PrimitiveType::Bitmask16 | PrimitiveType::Bitmask32
            ) {
                return Err("attribute flags have no reflected bitmask representation".into());
            }
            let initialized = flags
                .inner_type
                .and_then(|index| registry.get(index))
                .and_then(|ty| {
                    ty.enum_fields
                        .values()
                        .find(|field| field.name == "Initialized")
                })
                .ok_or("attribute flags have no reflected Initialized bit")?;
            if initialized.value >= u64::from(size) * 8 {
                return Err("attribute Initialized bit exceeds flag storage".into());
            }
            result[key]["initialized_mask"] = json!(1u32 << initialized.value);
        }
    }
    let mut ranges: Vec<_> = result
        .as_object()
        .unwrap()
        .values()
        .map(|field| {
            let offset = field["offset"].as_u64().unwrap();
            (offset, offset + field["size"].as_u64().unwrap())
        })
        .collect();
    ranges.sort_unstable();
    if ranges.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Err("attribute header fields overlap".into());
    }
    Ok(result)
}

pub(crate) fn read_header(header: &Value, name: &str, bytes: &[u8]) -> Result<u32, String> {
    let field = &header[name];
    let offset = uint(&field["offset"])? as usize;
    let size = uint(&field["size"])? as usize;
    let value = bytes
        .get(
            offset
                ..offset
                    .checked_add(size)
                    .ok_or("attribute header offset overflow")?,
        )
        .ok_or("attribute header outside component")?;
    match size {
        1 => Ok(u32::from(value[0])),
        2 => Ok(u32::from(u16::from_le_bytes(value.try_into().unwrap()))),
        4 => Ok(u32::from_le_bytes(value.try_into().unwrap())),
        _ => Err("unsupported reflected attribute header width".into()),
    }
}

fn component_layouts(registry: &TypeRegistry) -> Result<Vec<Value>, String> {
    let mut layouts = Vec::new();
    for ty in registry
        .iter()
        .filter(|ty| ty.qualified_name.starts_with("keen::ecs::"))
    {
        if !types::inheritance(registry, ty.index)?.iter().any(|index| {
            registry
                .get(*index)
                .is_some_and(|base| base.qualified_name == "keen::ecs::AttributeComponent")
        }) {
            continue;
        }
        let fields = types::fields(registry, ty.index)?;
        let Some((_, storage)) = fields.iter().find(|(_, field)| field.name == "dataStorage")
        else {
            continue;
        };
        let Some(array) = registry.get(storage.r#type) else {
            continue;
        };
        let Some(element) = array.inner_type.and_then(|index| registry.get(index)) else {
            continue;
        };
        if array.primitive_type != PrimitiveType::StaticArray
            || element.size != 4
            || !matches!(
                element.primitive_type,
                PrimitiveType::SInt32 | PrimitiveType::UInt32 | PrimitiveType::Float32
            )
            || array.field_count.checked_mul(4) != Some(array.size)
            || storage
                .data_offset
                .checked_add(u64::from(array.size))
                .is_none_or(|end| end > u64::from(ty.size))
        {
            continue;
        }
        let Ok(header) = header_layout(registry, ty.index) else {
            continue;
        };
        let storage_end = storage.data_offset + u64::from(array.size);
        if header.as_object().unwrap().values().any(|field| {
            let start = field["offset"].as_u64().unwrap();
            start < storage_end && start + field["size"].as_u64().unwrap() > storage.data_offset
        }) {
            continue;
        }
        let mut layout = json!({"qualified_name":ty.qualified_name,"qualified_hash":ty.qualified_hash,
            "size":ty.size,"storage_offset":storage.data_offset,"storage_size":array.size,"header":header,
            "storage_scalar_type":element.qualified_name,"engine_server_only":ty.attributes.contains_key("server_only"),
            "association":"reflected-attribute-storage"});
        if let Some((_, definition)) = fields.iter().find(|(_, field)| field.name == "definition") {
            if registry
                .get(definition.r#type)
                .is_some_and(|ty| ty.size == 16)
            {
                if let Some(bytes) = ty.default_value.as_ref().and_then(|bytes| {
                    bytes.get(definition.data_offset as usize..definition.data_offset as usize + 16)
                }) {
                    layout["definition_offset"] = json!(definition.data_offset);
                    layout["definition_guid"] =
                        json!(Guid::new(bytes.try_into().unwrap()).to_string());
                }
            }
        }
        layouts.push(layout);
    }
    Ok(layouts)
}

impl Catalog {
    pub(crate) fn bind_storage(
        &self,
        descriptor: &Value,
        component: &Value,
    ) -> Result<Value, String> {
        let count = self.root_entries(uint(&descriptor["root_hash"])?)?.len();
        if component["storage_size"].as_u64() != Some(count as u64 * 4) {
            return Err("attribute component storage count differs from KFC root".into());
        }
        let scalar = descriptor["scalar_type"].as_str();
        let storage = component["storage_scalar_type"].as_str();
        let exact = scalar.is_some() && scalar == storage;
        let integer = matches!(scalar, Some("keen::sint32" | "keen::uint32"))
            && matches!(storage, Some("keen::sint32" | "keen::uint32"));
        let calculation = exact || (integer && descriptor["calculation_sign_independent"] == true);
        let mut result = descriptor.clone();
        result["components"] = json!([component]);
        result["storage_writable"] = json!(exact || integer);
        result["storage_reason"] = if exact || integer {
            Value::Null
        } else {
            json!("KFC scalar type differs from reflected storage")
        };
        result["write_value_domain"] = json!(if !exact && integer {
            "integer-common-range-0-to-2147483647"
        } else {
            "declared-scalar"
        });
        result["calculation_writable"] = json!(calculation);
        result["calculation_reason"] = if calculation {
            Value::Null
        } else {
            json!("signedness-dependent program with conflicting resource/storage scalar types")
        };
        if calculation {
            result["calculation_scalar_type"] = component["storage_scalar_type"].clone();
        }
        Ok(result)
    }

    pub fn load(
        registry: &TypeRegistry,
        file: &KFCFile,
        reader: &mut KFCCursor<KFCReader>,
    ) -> Result<Self, String> {
        let mut resources = Vec::new();
        for name in [
            "keen::AttributeContainerResource",
            "keen::BaseAttributeResource",
        ] {
            let Some(ty) = registry.get_by_name(LookupKey::Qualified(name)) else {
                continue;
            };
            for id in shroudforge_parser::kfc_format::resources_by_type(file, registry, name) {
                let mut bytes = Vec::new();
                if !reader
                    .read_resource_into(&id, &mut bytes)
                    .map_err(|e| e.to_string())?
                {
                    return Err(format!("missing attribute resource {id}"));
                }
                let value =
                    KfcValue::from_bytes(registry, ty, &bytes).map_err(|e| e.to_string())?;
                resources.push(json!({"id":id.guid().to_string(),"type":name,"value":value}));
            }
        }
        Self::from_resources(registry, &resources)
    }

    fn from_resources(types: &TypeRegistry, resources: &[Value]) -> Result<Self, String> {
        let mut roots: BTreeMap<u32, (Value, Vec<String>, Vec<String>)> = BTreeMap::new();
        for resource in resources {
            let container = resource["type"] == "keen::AttributeContainerResource";
            let values: Vec<_> = if container {
                resource["value"]["attributes"]
                    .as_array()
                    .ok_or("missing container attributes")?
                    .iter()
                    .collect()
            } else {
                vec![&resource["value"]]
            };
            for value in values {
                let root = uint(&value["ids"][0]["value"])?;
                let record = roots
                    .entry(root)
                    .or_insert_with(|| (value.clone(), Vec::new(), Vec::new()));
                if record.0 != *value {
                    return Err(format!(
                        "conflicting container/base definition for {root:#010x}"
                    ));
                }
                let ids = if container {
                    &mut record.2
                } else {
                    &mut record.1
                };
                ids.push(
                    resource["id"]
                        .as_str()
                        .ok_or("missing resource GUID")?
                        .to_owned(),
                );
            }
        }
        if roots.is_empty() {
            return Err("no attribute definitions in this KFC container".into());
        }
        let operations: BTreeMap<u32, String> = types
            .get_by_name(LookupKey::Qualified("keen::AttributeOps"))
            .map(|ty| {
                ty.enum_fields
                    .values()
                    .map(|field| (field.value as u32, field.name.clone()))
                    .collect()
            })
            .unwrap_or_default();
        let mut catalog = Self {
            layouts: component_layouts(types)?,
            ..Self::default()
        };
        let encoding_verified = [
            (1, "Load0"),
            (2, "Load"),
            (3, "LoadRef"),
            (5, "Push"),
            (6, "Add"),
            (7, "Substract"),
            (8, "Multiply"),
            (12, "Clamp"),
            (13, "ScaleToNewMax"),
            (15, "Max"),
        ]
        .iter()
        .all(|(opcode, name)| operations.get(opcode).map(String::as_str) == Some(*name));
        for (root, (value, resource_ids, container_ids)) in roots {
            let ids = value["ids"].as_array().ok_or("missing attribute IDs")?;
            let names = value["debugNames"]
                .as_array()
                .ok_or("missing attribute names")?;
            let structure = value["structure"]
                .as_array()
                .ok_or("missing attribute structure")?;
            if ids.len() != names.len() || ids.len() != structure.len() || ids.len() > 65535 {
                return Err("attribute name/ID/structure lengths disagree".into());
            }
            let scalar_hash = uint(&value["type"]["value"])?;
            let scalar_types: Vec<_> = types
                .iter()
                .filter(|t| {
                    t.internal_hash == scalar_hash
                        && t.size == 4
                        && matches!(
                            t.primitive_type,
                            PrimitiveType::SInt32 | PrimitiveType::UInt32 | PrimitiveType::Float32
                        )
                })
                .collect();
            let scalar = match scalar_types.as_slice() {
                [t] => Some(*t),
                _ => None,
            };
            let mut components = Vec::new();
            for layout in &catalog.layouts {
                let Some(guid) = layout["definition_guid"].as_str() else {
                    continue;
                };
                let guid_bytes =
                    Guid::parse(guid).ok_or("invalid reflected attribute definition GUID")?;
                // The engine initializer hashes the GUID bytes, not a display
                // name. This also resolves roots only present in the container.
                if definition_root(guid_bytes.data()) != root {
                    continue;
                }
                if !resource_ids.is_empty() && !resource_ids.iter().any(|id| id == guid) {
                    return Err(format!(
                        "attribute definition GUID/hash collision for {root:#010x}"
                    ));
                }
                if layout["storage_size"].as_u64() != Some(ids.len() as u64 * 4) {
                    continue;
                }
                let mut component = layout.clone();
                component["scalar_type_matches"] =
                    json!(scalar.is_some_and(|s| Some(s.qualified_name.as_str())
                        == layout["storage_scalar_type"].as_str()));
                component["integer_representation_compatible"] = json!(
                    scalar.is_some_and(|s| matches!(
                        s.primitive_type,
                        PrimitiveType::SInt32 | PrimitiveType::UInt32
                    )) && matches!(
                        layout["storage_scalar_type"].as_str(),
                        Some("keen::sint32" | "keen::uint32")
                    )
                );
                component["association"] = json!("fnv1a-definition-guid-bytes-and-static-array");
                components.push(component);
            }
            let scalar_matches =
                components.len() == 1 && components[0]["scalar_type_matches"] == true;
            let common_integer =
                components.len() == 1 && components[0]["integer_representation_compatible"] == true;
            // These instructions operate on the same 32-bit representation
            // for signed/unsigned interpreters, including wrapping arithmetic.
            // Ordering, division and rescaling are intentionally excluded.
            let sign_independent = structure.iter().all(|node| {
                let Some(words) = node["calculation"].as_array() else {
                    return false;
                };
                let mut pc = 0;
                while pc < words.len() {
                    let Ok(word) = uint(&words[pc]) else {
                        return false;
                    };
                    pc += 1;
                    match word >> 16 {
                        1 | 2 | 6 | 7 | 8 => {}
                        5 if pc < words.len() => pc += 1,
                        _ => return false,
                    }
                }
                true
            });
            for (index, id) in ids.iter().enumerate() {
                let hash = uint(&id["value"])?;
                let name = names[index]
                    .as_str()
                    .filter(|n| !n.is_empty())
                    .ok_or("empty original attribute name")?;
                for link in ["parentIndex", "childIndex", "siblingIndex"] {
                    let target = uint(&structure[index][link])?;
                    if target != 65535 && target as usize >= ids.len() {
                        return Err(format!("invalid attribute {link}"));
                    }
                }
                let words = structure[index]["calculation"]
                    .as_array()
                    .ok_or("missing calculation words")?;
                for word in words {
                    uint(word)?;
                }
                let descriptor = json!({"name":name,"hash":hash,"hash_domain":"attribute-id",
                    "name_source":"kfc-debugNames","root_hash":root,"index":index,"resource_ids":resource_ids,
                    "container_ids":container_ids,"scalar_type_hash":scalar_hash,
                    "scalar_type":scalar.map(|t| t.qualified_name.as_str()),"components":components,
                    "calculation_scalar_type":if components.len() == 1 && (scalar_matches || (common_integer && sign_independent)) {
                        components[0]["storage_scalar_type"].clone()
                    } else {json!(scalar.map(|t| t.qualified_name.as_str()))},
                    "structure":structure[index],"calculation_words":words,
                    "calculation":decode_program(words, &operations, ids, names),
                    "calculation_encoding_verified":encoding_verified,
                    "calculation_sign_independent":sign_independent,
                    "storage_resolution":if components.is_empty() {"per-entity-initialized-root-id"} else {"reflected-definition-guid"},
                    "literal_encoding":if scalar.is_some_and(|s| s.primitive_type == PrimitiveType::Float32) {"uint32 converted to float32"} else {"32-bit integer word"},
                    "calculation_execution":"attribute-command-v1, descending root index, guarded updates require exact build validation",
                    "effect_scope":"local process, network authority and publication remain engine-managed",
                    "storage_writable":scalar_matches || common_integer,
                    "write_value_domain":if !scalar_matches && common_integer {"integer-common-range-0-to-2147483647"} else {"declared-scalar"},
                    "calculation_writable":scalar_matches || (common_integer && sign_independent),
                    "calculation_reason":if components.is_empty() {Some("calculation storage type requires an initialized entity root match")}
                        else if !scalar_matches && !sign_independent {Some("signedness-dependent program with conflicting resource/storage scalar types")} else {None},
                    "storage_reason":if components.is_empty() {Some("no proven component/root association. Compact headers without a reflected definition GUID require an independently observed initialized root ID")}
                        else if !scalar_matches && !common_integer {Some("KFC scalar type differs from reflected storage")}
                        else if components.len() != 1 {Some("ambiguous component ownership")} else {None}});
                if catalog.entries.insert(hash, descriptor).is_some() {
                    return Err(format!("duplicate attribute ID {hash:#010x}"));
                }
            }
        }
        Ok(catalog)
    }

    pub(crate) fn root_entries(&self, root: u32) -> Result<Vec<&Value>, String> {
        let mut entries: Vec<_> = self
            .entries
            .values()
            .filter(|e| e["root_hash"] == root)
            .collect();
        entries.sort_by_key(|e| e["index"].as_u64());
        if entries.is_empty() || entries.iter().enumerate().any(|(i, e)| e["index"] != i) {
            return Err("incomplete attribute root".into());
        }
        Ok(entries)
    }

    /// Engine recompute walks structure from last entry to first. ScaleToNewMax
    /// can update an earlier referenced value before that entry is evaluated.
    pub(crate) fn recalculate(
        &self,
        root: u32,
        values: &[u32],
    ) -> Result<(Vec<u32>, Vec<Value>), String> {
        let entries = self.root_entries(root)?;
        if entries[0]["calculation_encoding_verified"] != true {
            return Err("engine AttributeOps enum differs from the verified model".into());
        }
        if entries.len() != values.len() {
            return Err("attribute root value count mismatch".into());
        }
        let kind = Scalar::parse(
            entries[0]["calculation_scalar_type"]
                .as_str()
                .ok_or("unknown scalar")?,
        )?;
        let mut result = values.to_vec();
        let mut trace = Vec::new();
        for (i, entry) in entries.iter().enumerate().rev() {
            let words: Vec<u32> = serde_json::from_value(entry["calculation_words"].clone())
                .map_err(|e| e.to_string())?;
            if words.is_empty() {
                continue;
            }
            let before = result.clone();
            if let Some(value) = calculation::evaluate(kind, &mut result, &words)? {
                result[i] = value;
            }
            trace.push(json!({"name":entry["name"],"hash":entry["hash"],"index":i,
                "before_bits":before,"after_bits":result}));
        }
        Ok((result, trace))
    }
}

/// Engine Guid -> HashKey32 (client 0x1e4600, server 0x2f080).
/// Zero GUID is special; nonzero GUIDs use FNV-1a in on-disk byte order.
pub(crate) fn definition_root(bytes: &[u8; 16]) -> u32 {
    if *bytes == [0; 16] {
        return 0;
    }
    bytes.iter().fold(0x811c9dc5u32, |h, b| {
        (h ^ *b as u32).wrapping_mul(0x1000193)
    })
}

pub(crate) fn uint(value: &Value) -> Result<u32, String> {
    value
        .as_u64()
        .and_then(|v| v.try_into().ok())
        .ok_or_else(|| "expected u32 attribute data".into())
}

// Observed in both signed attribute interpreters (client 0x1e1b00, server
// 0x2c310): SHR opcode,16; low 16-bit operand; Push increments the word cursor
// once more before loading its 32-bit literal. This decodes, never executes.
pub(crate) fn decode_program(
    words: &[Value],
    operations: &BTreeMap<u32, String>,
    ids: &[Value],
    names: &[Value],
) -> Value {
    let mut result = Vec::new();
    let mut offset = 0;
    let mut complete = true;
    while offset < words.len() {
        let word = uint(&words[offset]).unwrap();
        let opcode = word >> 16;
        let operand = word & 0xffff;
        let name = operations.get(&opcode).map(String::as_str);
        let mut instruction = json!({"word_offset":offset,"word":word,"opcode":opcode,"operation":name,"operand":operand});
        offset += 1;
        if name == Some("Push") {
            if let Some(immediate) = words.get(offset) {
                instruction["immediate_bits"] = immediate.clone();
                offset += 1;
            } else {
                instruction["reason"] = json!("truncated Push literal");
                complete = false;
            }
        } else if matches!(name, Some("Load" | "LoadRef" | "Store")) {
            if let (Some(id), Some(name)) = (ids.get(operand as usize), names.get(operand as usize))
            {
                instruction["attribute_hash"] = id["value"].clone();
                instruction["attribute_name"] = name.clone();
            } else {
                instruction["reason"] = json!("attribute index outside root definition");
                complete = false;
            }
        }
        if name.is_none() {
            complete = false;
        }
        result.push(instruction);
    }
    json!({"encoding":"attribute-command-v1","decoded":complete,"native_callable":false,
        "evaluation_api":"runtime.ecs.evaluate_attributes","instructions":result})
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn header_fixture(compact: bool) -> TypeRegistry {
        let header_size = if compact { 8 } else { 36 };
        let mut definitions = Vec::new();
        for (index, (name, kind, size, inner)) in [
            ("uint8", "UINT8", 1, None),
            ("uint16", "UINT16", 2, None),
            ("uint32", "UINT32", 4, None),
            ("HashKey32", "STRUCT", 4, None),
            ("Guid", "GUID", 16, None),
            (
                "AttributeFlags",
                if compact { "BITMASK8" } else { "BITMASK16" },
                if compact { 1 } else { 2 },
                Some(9),
            ),
            ("ecs::AttributeComponent", "STRUCT", header_size, None),
            ("AttributeStorage", "STATIC_ARRAY", 32, Some(2)),
            ("ecs::Stamina", "STRUCT", header_size + 32, Some(6)),
            (
                "AttributeFlagsEnum",
                "ENUM",
                if compact { 1 } else { 2 },
                Some(if compact { 0 } else { 1 }),
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let qualified = format!("keen::{name}");
            definitions.push(json!({"index":index,"name":name,"qualifiedName":qualified,"impactName":name,
                "size":size,"alignment":1,"elementAlignment":1,"fieldCount":if index == 7 {8} else {0},
                "primitiveType":kind,"innerType":inner,"flags":"","nameHash":kfc::hash::fnv(name),
                "qualifiedHash":kfc::hash::fnv(&qualified),"impactHash":kfc::hash::fnv(name),"internalHash":index}));
        }
        definitions[3]["structFields"] = json!({"value":{"name":"value","type":2,"dataOffset":0}});
        definitions[6]["structFields"] = if compact {
            json!({"rootId":{"name":"rootId","type":3,"dataOffset":0},
                "storageOffset":{"name":"storageOffset","type":1,"dataOffset":4},
                "flags":{"name":"flags","type":5,"dataOffset":6},
                "storageCount":{"name":"storageCount","type":0,"dataOffset":7}})
        } else {
            json!({"rootId":{"name":"rootId","type":3,"dataOffset":0},
                "storageOffset":{"name":"storageOffset","type":2,"dataOffset":8},
                "storageSize":{"name":"storageSize","type":2,"dataOffset":12},
                "flags":{"name":"flags","type":5,"dataOffset":16},
                "definition":{"name":"definition","type":4,"dataOffset":20}})
        };
        definitions[8]["structFields"] =
            json!({"dataStorage":{"name":"dataStorage","type":7,"dataOffset":header_size}});
        definitions[9]["enumFields"] = json!({"Initialized":{"name":"Initialized","value":0}});
        serde_json::from_value(json!({"version":"attribute-header-fixture","types":definitions}))
            .unwrap()
    }

    #[test]
    fn attribute_header_uses_reflected_kfc3_and_kfc4_widths() {
        for compact in [false, true] {
            let registry = header_fixture(compact);
            let header = header_layout(&registry, TypeIndex::new(8)).unwrap();
            assert_eq!(header["rootId"]["size"], 4);
            assert_eq!(header["storageOffset"]["size"], if compact { 2 } else { 4 });
            assert_eq!(header["storageCount"]["size"], if compact { 1 } else { 4 });
            assert_eq!(header["flags"]["size"], if compact { 1 } else { 2 });
            assert_eq!(
                header["storageCount"]["name"],
                if compact {
                    "storageCount"
                } else {
                    "storageSize"
                }
            );
        }
        let mut invalid = serde_json::to_value(header_fixture(true)).unwrap();
        invalid["types"][6]["structFields"]["storageCount"]["dataOffset"] = json!(6);
        let registry = serde_json::from_value(invalid).unwrap();
        assert!(
            header_layout(&registry, TypeIndex::new(8))
                .unwrap_err()
                .contains("overlap")
        );
    }
}
