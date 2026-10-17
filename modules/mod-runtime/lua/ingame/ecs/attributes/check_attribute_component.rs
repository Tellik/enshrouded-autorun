use kfc::reflection::LookupKey;
use kfc::reflection::TypeRegistry;

pub(super) fn is_attribute_component(registry: &TypeRegistry, name: &str) -> Result<bool, String> {
    let metadata = registry
        .get_by_name(LookupKey::Qualified(name))
        .ok_or_else(|| format!("unresolved component type: {name}"))?;
    Ok(
        crate::runtime_resolution::types::inheritance(registry, metadata.index)?
            .iter()
            .any(|index| {
                registry
                    .get(*index)
                    .is_some_and(|base| base.qualified_name == "keen::ecs::AttributeComponent")
            }),
    )
}
