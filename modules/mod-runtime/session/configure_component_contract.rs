use super::*;
use crate::runtime_provider;
use kfc::reflection::TypeMetadata;
fn is_component_type(registry: &TypeRegistry, metadata: &TypeMetadata) -> bool {
    if !metadata.qualified_name.starts_with("keen::ecs::")
        || metadata.qualified_name == "keen::ecs::Component"
    {
        return false;
    }
    let mut current = Some(metadata);
    for _ in 0..=registry.len() {
        let Some(ty) = current else {
            return false;
        };
        if ty.qualified_name == "keen::ecs::Component" {
            return true;
        }
        current = registry.get_inner_type(ty);
    }
    false
}

pub(crate) fn configure_component_contract(app_state: &AppState) {
    if app_state.phase() == RuntimePhase::Ingame && !app_state.runtime_configured.get() {
        let registry = app_state.type_registry();
        let ecs_catalog_types = registry
            .iter()
            .filter(|metadata| metadata.qualified_name.starts_with("keen::ecs::"))
            .count();
        let component_candidates = registry
            .iter()
            .filter(|metadata| metadata.size > 0 && is_component_type(registry, metadata))
            .count();
        let dynamic_candidates = registry
            .iter()
            .filter(|metadata| {
                metadata.size > 0
                    && metadata.qualified_name.starts_with("keen::ecs::Dynamic")
                    && !is_component_type(registry, metadata)
            })
            .count();
        let contract: Vec<_> = registry
            .iter()
            .filter(|metadata| {
                metadata.size > 0 && metadata.qualified_name.starts_with("keen::ecs::")
            })
            .map(|metadata| (metadata.qualified_name.clone(), metadata.size))
            .collect();
        tracing::debug!(
            target: "shroudforge::runtime",
            reflection_types = registry.len(),
            ecs_catalog_types,
            component_candidates,
            dynamic_runtime_struct_candidates = dynamic_candidates,
            index_candidates = contract.len(),
            "Prepared KFC runtime component index candidates"
        );
        let configured = runtime_provider::configure(&contract);
        if configured {
            app_state.runtime_configured.set(true);
        }
        if !configured {
            tracing::error!("KFC Runtime rejected the component contract");
        }
        tracing::debug!(
            target: "shroudforge::runtime",
            configured,
            components = contract.len(),
            "KFC runtime component contract configuration finished"
        );
    }
}
