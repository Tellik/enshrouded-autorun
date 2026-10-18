use super::{
    FieldStep, InlineField, field_path, inline_leaves, reject_external_overlap,
    validate_runtime_scalar,
};
use crate::{
    env::registry,
    type_aliases::{MappedValue, TypeHandle},
};
use kfc::reflection::{PrimitiveType, TypeIndex, TypeRegistry};
use std::{collections::HashSet, rc::Rc};

/// Resolve one naturally aligned scalar leaf inside an owned resource snapshot. The KFC
/// mapper selects fields, indices and concrete variants. Source offsets follow
/// its exact BlobArray/BlobVariant format, because that mapper does not expose
/// source positions. See kfc-resource/src/mapped/mod.rs:498-514,563-603 and
/// src/value/write.rs:817-821 in the pinned, unchanged parser dependency.
/// This scope never follows Ds pointers or changes container descriptors.
pub(crate) fn resource_field(
    registry: &Rc<TypeRegistry>,
    root_type: TypeIndex,
    path: &str,
    bytes: &[u8],
) -> Result<InlineField, String> {
    fn span(bytes: &[u8], offset: usize, size: usize) -> Result<(), String> {
        offset
            .checked_add(size)
            .filter(|end| *end <= bytes.len())
            .map(|_| ())
            .ok_or_else(|| "field outside owned resource snapshot".into())
    }
    fn u32_at(bytes: &[u8], offset: usize) -> Result<usize, String> {
        span(bytes, offset, 4)?;
        Ok(u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize)
    }
    let steps = field_path(path)?;
    let root = registry.get(root_type).ok_or("unresolved resource type")?;
    if bytes.len() > u32::MAX as usize {
        return Err("resource snapshot exceeds 32-bit blob offset range".into());
    }
    span(bytes, 0, root.size as usize)?;
    let owned: Rc<[u8]> = bytes.into();
    let mut mapped = MappedValue::from_bytes(registry, root, &owned).map_err(|e| e.to_string())?;
    let mut index = root_type;
    let mut offset = 0usize;
    let mut containers = Vec::new();
    let mut descriptors = Vec::new();
    for step in steps {
        for current in registry::inheritance(registry, index)? {
            if registry
                .get(current)
                .ok_or("unresolved resource field type")?
                .primitive_type
                != PrimitiveType::Typedef
            {
                index = current;
                break;
            }
        }
        let ty = registry
            .get(index)
            .ok_or("unresolved resource field type")?;
        span(bytes, offset, ty.size as usize)?;
        match (step, ty.primitive_type, &mapped) {
            (FieldStep::Name(name), PrimitiveType::Struct, MappedValue::Struct(value)) => {
                if !Rc::ptr_eq(value.data(), &owned) || value.r#type().index != index {
                    return Err("mapped struct does not belong to this resource scope".into());
                }
                let fields = registry::fields(registry, index)?;
                let (_, field) = fields
                    .iter()
                    .find(|(_, field)| field.name == name)
                    .ok_or_else(|| {
                        format!("reflected field not found: {}.{name}", ty.qualified_name)
                    })?;
                let child = registry
                    .get(field.r#type)
                    .ok_or("unresolved resource child type")?;
                let relative =
                    usize::try_from(field.data_offset).map_err(|_| "field offset overflow")?;
                if relative
                    .checked_add(child.size as usize)
                    .is_none_or(|end| end > ty.size as usize)
                {
                    return Err("field exceeds its reflected parent storage".into());
                }
                containers.push((index, offset));
                offset = offset
                    .checked_add(relative)
                    .ok_or("field offset overflow")?;
                index = child.index;
                span(bytes, offset, child.size as usize)?;
                mapped = value
                    .get(name)
                    .map_err(|e| e.to_string())?
                    .ok_or("mapped field missing")?;
            }
            (
                FieldStep::Index(position),
                PrimitiveType::StaticArray | PrimitiveType::BlobArray,
                MappedValue::Array(value),
            ) => {
                if !Rc::ptr_eq(value.data(), &owned) || value.r#type().index != index {
                    return Err("mapped array does not belong to this resource scope".into());
                }
                let inner = value.element_type();
                if inner.size == 0 || position >= value.len() {
                    return Err("resource array index or element size is invalid".into());
                }
                let length = value
                    .len()
                    .checked_mul(inner.size as usize)
                    .ok_or("array size overflow")?;
                let start = if ty.primitive_type == PrimitiveType::BlobArray {
                    if ty.size != 8 || value.len() != u32_at(bytes, offset + 4)? {
                        return Err("invalid reflected blob array descriptor".into());
                    }
                    descriptors.push(offset..offset + 8);
                    let start = offset
                        .checked_add(u32_at(bytes, offset)?)
                        .ok_or("blob offset overflow")?;
                    if start < offset + 8 || start < root.size as usize {
                        return Err("blob array payload overlaps its resource descriptors".into());
                    }
                    start
                } else {
                    if value.len() != ty.field_count as usize || length != ty.size as usize {
                        return Err("invalid reflected static array layout".into());
                    }
                    offset
                };
                span(bytes, start, length)?;
                offset = start
                    .checked_add(
                        position
                            .checked_mul(inner.size as usize)
                            .ok_or("array element offset overflow")?,
                    )
                    .ok_or("array element offset overflow")?;
                index = inner.index;
                mapped = value
                    .get(position)
                    .map_err(|e| e.to_string())?
                    .ok_or("mapped array element missing")?;
            }
            (FieldStep::Name("value"), PrimitiveType::BlobVariant, MappedValue::Variant(value)) => {
                if ty.size != 12
                    || value.base_type().index != index
                    || !Rc::ptr_eq(value.value().data(), &owned)
                {
                    return Err("mapped variant does not belong to this resource scope".into());
                }
                let concrete = value.variant_type();
                let relative = u32_at(bytes, offset + 4)?;
                if relative == 0
                    || u32_at(bytes, offset)? != concrete.qualified_hash as usize
                    || u32_at(bytes, offset + 8)? != concrete.size as usize
                {
                    return Err("invalid, absent or mismatched resource variant descriptor".into());
                }
                let base = registry
                    .get(ty.inner_type.ok_or("missing variant base type")?)
                    .ok_or("unresolved variant base type")?;
                if concrete.primitive_type != PrimitiveType::Struct
                    || !registry.is_sub_type(base, concrete)
                {
                    return Err("resource variant is not a reflected subtype".into());
                }
                descriptors.push(offset..offset + 12);
                let start = offset
                    .checked_add(4)
                    .and_then(|n| n.checked_add(relative))
                    .ok_or("variant payload offset overflow")?;
                if start < offset + 12 || start < root.size as usize {
                    return Err("variant payload overlaps its resource descriptors".into());
                }
                span(bytes, start, concrete.size as usize)?;
                offset = start;
                index = concrete.index;
                mapped = MappedValue::Struct(value.value().clone());
            }
            _ => {
                return Err(format!(
                    "{} does not support this owned resource field step",
                    ty.qualified_name
                ));
            }
        }
    }
    let ty = TypeHandle::new(registry.clone(), index);
    let mut leaves = Vec::new();
    inline_leaves(
        registry,
        index,
        0,
        ty.size as usize,
        &mut HashSet::new(),
        &mut leaves,
    )?;
    let width = ty.size as usize;
    if !matches!(width, 1 | 2 | 4 | 8)
        || offset % width != 0
        || leaves.len() != 1
        || leaves[0].1 != (0..width)
    {
        return Err(
            "resource writes require one naturally aligned, contiguous 1/2/4/8-byte value leaf"
                .into(),
        );
    }
    span(bytes, offset, width)?;
    let selected = offset..offset + width;
    if descriptors
        .iter()
        .any(|range| range.start < selected.end && selected.start < range.end)
    {
        return Err("resource field overlaps a traversed container descriptor".into());
    }
    for (container, base) in containers {
        reject_external_overlap(registry, container, base, &selected, &mut HashSet::new())?;
    }
    validate_runtime_scalar(leaves[0].0, &bytes[selected])?;
    Ok(InlineField { ty, offset, leaves })
}
