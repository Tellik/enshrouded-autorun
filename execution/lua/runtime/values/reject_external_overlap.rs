use crate::env::registry;
use kfc::reflection::{PrimitiveType, TypeIndex, TypeRegistry};
use std::collections::HashSet;

pub(super) fn reject_external_overlap(
    registry: &TypeRegistry,
    index: TypeIndex,
    base: usize,
    selected: &std::ops::Range<usize>,
    stack: &mut HashSet<TypeIndex>,
) -> Result<(), String> {
    let ty = registry
        .get(index)
        .ok_or("unresolved enclosing field type")?;
    let end = base
        .checked_add(ty.size as usize)
        .ok_or("enclosing field offset overflow")?;
    if end <= selected.start || base >= selected.end {
        return Ok(());
    }
    if stack.len() >= 64 || !stack.insert(index) {
        return Err("cyclic or excessively nested enclosing field layout".into());
    }
    match ty.primitive_type {
        PrimitiveType::Typedef => {
            reject_external_overlap(
                registry,
                ty.inner_type.ok_or("missing typedef value type")?,
                base,
                selected,
                stack,
            )?;
        }
        PrimitiveType::Struct => {
            for (_, field) in registry::fields(registry, index)? {
                let relative =
                    usize::try_from(field.data_offset).map_err(|_| "field offset overflow")?;
                let offset = base.checked_add(relative).ok_or("field offset overflow")?;
                reject_external_overlap(registry, field.r#type, offset, selected, stack)?;
            }
        }
        PrimitiveType::StaticArray => {
            let inner = ty
                .inner_type
                .and_then(|i| registry.get(i))
                .ok_or("missing static array element type")?;
            if inner.size == 0
                || (inner.size as usize).checked_mul(ty.field_count as usize)
                    != Some(ty.size as usize)
            {
                return Err("invalid enclosing static array layout".into());
            }
            let width = inner.size as usize;
            let first = selected.start.saturating_sub(base) / width;
            let last = (selected.end.min(end) - base - 1) / width;
            for position in first..=last {
                reject_external_overlap(
                    registry,
                    inner.index,
                    base + position * width,
                    selected,
                    stack,
                )?;
            }
        }
        PrimitiveType::DsArray
        | PrimitiveType::DsString
        | PrimitiveType::DsOptional
        | PrimitiveType::DsVariant
        | PrimitiveType::BlobArray
        | PrimitiveType::BlobString
        | PrimitiveType::BlobOptional
        | PrimitiveType::BlobVariant
        | PrimitiveType::ObjectReference => {
            return Err(format!(
                "selected field overlaps external storage or reference descriptor {}",
                ty.qualified_name
            ));
        }
        _ => {}
    }
    stack.remove(&index);
    Ok(())
}
