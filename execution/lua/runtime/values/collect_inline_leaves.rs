use crate::env::registry;
use kfc::reflection::{PrimitiveType, TypeIndex, TypeRegistry};
use std::collections::HashSet;

pub(super) fn inline_leaves(
    registry: &TypeRegistry,
    index: TypeIndex,
    base: usize,
    limit: usize,
    stack: &mut HashSet<TypeIndex>,
    leaves: &mut Vec<(PrimitiveType, std::ops::Range<usize>)>,
) -> Result<(), String> {
    if stack.len() >= 64 || !stack.insert(index) {
        return Err("cyclic or excessively nested inline field layout".into());
    }
    let ty = registry.get(index).ok_or("unresolved inline field type")?;
    let end = base
        .checked_add(ty.size as usize)
        .filter(|end| *end <= limit)
        .ok_or_else(|| {
            format!(
                "inline field outside reflected storage: {}",
                ty.qualified_name
            )
        })?;
    match ty.primitive_type {
        PrimitiveType::Typedef => {
            let inner = ty.inner_type.ok_or("missing typedef value type")?;
            inline_leaves(registry, inner, base, end, stack, leaves)?;
        }
        PrimitiveType::Struct => {
            for (_, field) in registry::fields(registry, index)? {
                let offset =
                    usize::try_from(field.data_offset).map_err(|_| "field offset overflow")?;
                let start = base.checked_add(offset).ok_or("field offset overflow")?;
                inline_leaves(registry, field.r#type, start, end, stack, leaves)?;
            }
        }
        PrimitiveType::StaticArray => {
            let inner = ty
                .inner_type
                .and_then(|i| registry.get(i))
                .ok_or("missing static array element type")?;
            if (inner.size as usize).checked_mul(ty.field_count as usize) != Some(ty.size as usize)
            {
                return Err("static array size differs from reflected element count".into());
            }
            for position in 0..ty.field_count as usize {
                inline_leaves(
                    registry,
                    inner.index,
                    base + position * inner.size as usize,
                    end,
                    stack,
                    leaves,
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
                "{} has external storage or reference ownership. Select an inline sibling field",
                ty.qualified_name
            ));
        }
        kind => {
            let width = match kind {
                PrimitiveType::Bool
                | PrimitiveType::UInt8
                | PrimitiveType::SInt8
                | PrimitiveType::Bitmask8 => 1,
                PrimitiveType::UInt16 | PrimitiveType::SInt16 | PrimitiveType::Bitmask16 => 2,
                PrimitiveType::UInt32
                | PrimitiveType::SInt32
                | PrimitiveType::Float32
                | PrimitiveType::Bitmask32 => 4,
                PrimitiveType::UInt64
                | PrimitiveType::SInt64
                | PrimitiveType::Float64
                | PrimitiveType::Bitmask64 => 8,
                PrimitiveType::Guid => 16,
                PrimitiveType::Enum => {
                    let inner = ty
                        .inner_type
                        .and_then(|i| registry.get(i))
                        .ok_or("missing enum storage type")?;
                    if !matches!(
                        inner.primitive_type,
                        PrimitiveType::UInt8
                            | PrimitiveType::SInt8
                            | PrimitiveType::UInt16
                            | PrimitiveType::SInt16
                            | PrimitiveType::UInt32
                            | PrimitiveType::SInt32
                            | PrimitiveType::UInt64
                            | PrimitiveType::SInt64
                    ) {
                        return Err("enum has no integer storage type".into());
                    }
                    inner.size as usize
                }
                _ => {
                    return Err(format!(
                        "{} is not an inline value field",
                        ty.qualified_name
                    ));
                }
            };
            if ty.size as usize != width || leaves.len() >= 65_536 {
                return Err("invalid or excessively large inline field layout".into());
            }
            leaves.push((kind, base..end));
        }
    }
    stack.remove(&index);
    Ok(())
}
