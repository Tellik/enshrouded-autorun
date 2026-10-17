use crate::env::registry;
use kfc::reflection::{PrimitiveType, TypeIndex, TypeRegistry};
use std::collections::HashSet;

fn supported(
    registry: &TypeRegistry,
    index: TypeIndex,
    seen: &mut HashSet<TypeIndex>,
) -> Result<(), String> {
    if !seen.insert(index) {
        return Ok(());
    }
    let ty = registry.get(index).ok_or("unresolved value type")?;
    if matches!(
        ty.primitive_type,
        PrimitiveType::DsArray
            | PrimitiveType::DsString
            | PrimitiveType::DsOptional
            | PrimitiveType::DsVariant
    ) {
        return Err(format!(
            "{} requires engine allocation and ownership. The KFC value codec cannot construct it",
            ty.qualified_name
        ));
    }
    registry::inheritance(registry, index)?;
    if let Some(inner) = ty.inner_type {
        supported(registry, inner, seen)?;
    }
    for field in ty.struct_fields.values() {
        supported(registry, field.r#type, seen)?;
    }
    Ok(())
}

pub(crate) fn validate_owned_type(registry: &TypeRegistry, index: TypeIndex) -> Result<(), String> {
    supported(registry, index, &mut HashSet::new())
}
