use super::{FieldStep, InlineField, field_path, inline_leaves, reject_external_overlap};
use crate::{env::registry, type_aliases::TypeHandle};
use kfc::reflection::{PrimitiveType, TypeIndex, TypeRegistry};
use std::{collections::HashSet, rc::Rc};

/// Resolve names and one-based static indices, such as `transform.position[1]`.
/// Resolution follows reflected inline containment only, never live pointers.
pub(crate) fn inline_field(
    registry: &Rc<TypeRegistry>,
    component: TypeIndex,
    path: &str,
) -> Result<InlineField, String> {
    let root = registry.get(component).ok_or("unresolved component type")?;
    let mut index = component;
    let mut offset = 0usize;
    for step in field_path(path)? {
        let chain = registry::inheritance(registry, index)?;
        for current in chain {
            let ty = registry.get(current).ok_or("unresolved field type")?;
            if ty.primitive_type != PrimitiveType::Typedef {
                index = current;
                break;
            }
        }
        let ty = registry.get(index).ok_or("unresolved field type")?;
        let (child, relative) = if let FieldStep::Index(position) = step {
            if ty.primitive_type != PrimitiveType::StaticArray {
                return Err(format!("{} is not a static array", ty.qualified_name));
            }
            let inner = ty
                .inner_type
                .and_then(|i| registry.get(i))
                .ok_or("missing static array element type")?;
            if position >= ty.field_count as usize
                || (inner.size as usize).checked_mul(ty.field_count as usize)
                    != Some(ty.size as usize)
            {
                return Err("static array index or reflected layout is invalid".into());
            }
            (inner.index, position * inner.size as usize)
        } else {
            if ty.primitive_type != PrimitiveType::Struct {
                return Err(format!(
                    "{} has no reflected inline fields",
                    ty.qualified_name
                ));
            }
            let FieldStep::Name(name) = step else {
                unreachable!()
            };
            let fields = registry::fields(registry, index)?;
            let (_, field) = fields
                .iter()
                .find(|(_, field)| field.name == name)
                .ok_or_else(|| {
                    format!("reflected field not found: {}.{name}", ty.qualified_name)
                })?;
            (
                field.r#type,
                usize::try_from(field.data_offset).map_err(|_| "field offset overflow")?,
            )
        };
        let child_type = registry.get(child).ok_or("unresolved field type")?;
        if relative
            .checked_add(child_type.size as usize)
            .is_none_or(|end| end > ty.size as usize)
        {
            return Err("field exceeds its reflected parent storage".into());
        }
        offset = offset
            .checked_add(relative)
            .ok_or("field offset overflow")?;
        index = child;
    }
    let ty = TypeHandle::new(registry.clone(), index);
    if offset
        .checked_add(ty.size as usize)
        .is_none_or(|end| end > root.size as usize)
    {
        return Err("field exceeds component storage".into());
    }
    let mut leaves = Vec::new();
    inline_leaves(
        registry,
        index,
        0,
        ty.size as usize,
        &mut HashSet::new(),
        &mut leaves,
    )?;
    leaves.sort_by_key(|(_, range)| range.start);
    if leaves.is_empty()
        || leaves
            .windows(2)
            .any(|pair| pair[0].1.end > pair[1].1.start)
    {
        return Err("inline field has empty or overlapping reflected storage".into());
    }
    reject_external_overlap(
        registry,
        component,
        0,
        &(offset..offset + ty.size as usize),
        &mut HashSet::new(),
    )?;
    Ok(InlineField { ty, offset, leaves })
}
